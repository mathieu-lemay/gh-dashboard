use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossterm::event::{Event, KeyCode};
use exn::Exn;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{
    Block,
    Clear,
    HighlightSpacing,
    Row,
    StatefulWidget,
    Table,
    TableState,
    Widget,
};
use tokio::sync::mpsc;
use tokio::time;

use crate::error::ServiceError;
use crate::models::{Repository, WorkflowRun};
use crate::service::workflows::{GitHubService, Service};
use crate::widgets::state::LoadingState;
use crate::widgets::workflow_details::WorkflowDetailsWidget;

/// A widget that displays a list of workflow runs.
///
/// This is an async widget that fetches the list of workflow runs from the
/// GitHub API. It contains an inner `Arc<RwLock<WorkflowListState>>` that holds
/// the state of the widget. Cloning the widget will clone the Arc, so you can
/// pass it around to other threads, and this is used to spawn a background task
/// to fetch the workflow runs.
#[derive(Debug, Clone)]
pub struct WorkflowRunListWidget {
    github_service: Arc<dyn GitHubService>,
    repos: Vec<Repository>,
    state: Arc<RwLock<WorkflowListState>>,
    details_widget: Arc<RwLock<WorkflowDetailsWidget>>,
}

#[derive(Debug, Default)]
struct WorkflowListState {
    workflow_runs: Vec<WorkflowRun>,
    loading_state: LoadingState,
    table_state: TableState,
}

impl Default for WorkflowRunListWidget {
    fn default() -> Self {
        Self {
            github_service: Arc::new(Service {}),
            repos: vec![],
            state: Arc::new(RwLock::new(WorkflowListState::default())),
            details_widget: Arc::new(RwLock::new(WorkflowDetailsWidget::default())),
        }
    }
}

impl WorkflowRunListWidget {
    pub fn new(github_service: Arc<dyn GitHubService>, repos: Vec<Repository>) -> Self {
        let details_widget = Arc::new(RwLock::new(WorkflowDetailsWidget::new(
            github_service.clone(),
        )));

        Self {
            github_service,
            repos,
            details_widget,
            ..Default::default()
        }
    }

    /// Start fetching the pull requests in the background.
    ///
    /// This method spawns a background task that fetches the pull requests from
    /// the GitHub API. The result of the fetch is then passed to the
    /// `on_load` or `on_err` methods.
    pub fn run(&self) -> mpsc::Sender<Event> {
        let this = self.clone(); // clone the widget to pass to the background task
        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(this.sync_data(rx));

        tx
    }

    async fn sync_data(mut self, mut rx: mpsc::Receiver<Event>) {
        let period = Duration::from_secs(60);
        let mut interval = time::interval(period);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.fetch_workflow_runs().await;
                },
                Some(event) = rx.recv() => {
                    self.handle_event(&event).await
                },
            }
        }
    }

    async fn handle_event(&mut self, event: &Event) {
        if let Some(key) = event.as_key_press_event() {
            match key.code {
                KeyCode::Enter => self.open_url(),
                KeyCode::Char('d') => self.show_details(),
                KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
                KeyCode::Char('r') => self.fetch_workflow_runs().await,
                KeyCode::Esc => self.hide_details().await,
                _ => {}
            }
        }
    }

    async fn fetch_workflow_runs(&self) {
        self.set_loading_state(LoadingState::Loading);

        let workflows = self.github_service.list_runs(&self.repos).await;

        match workflows {
            Ok(wfs) => self.on_load(wfs),
            Err(err) => self.on_err(&err),
        }
    }

    fn on_load(&self, runs: Vec<WorkflowRun>) {
        let mut state = self.state.write().unwrap();

        state.workflow_runs = runs;

        if !state.workflow_runs.is_empty() && state.table_state.selected().is_none() {
            state.table_state.select(Some(0));
        }

        state.loading_state = LoadingState::Loaded(chrono::Local::now());
    }

    fn on_err(&self, err: &Exn<ServiceError>) {
        self.set_loading_state(LoadingState::Error(err.to_string()));
    }

    fn set_loading_state(&self, state: LoadingState) {
        self.state.write().unwrap().loading_state = state;
    }

    fn scroll_down(&self) {
        self.state.write().unwrap().table_state.scroll_down_by(1);
    }

    fn scroll_up(&self) {
        self.state.write().unwrap().table_state.scroll_up_by(1);
    }

    fn show_details(&self) {
        let state = self.state.read().unwrap();

        let idx = match state.table_state.selected() {
            Some(idx) => idx,
            None => return,
        };

        let workflow = state.workflow_runs[idx].clone();

        let mut w = self.details_widget.write().unwrap();

        w.hide(); // Hide / stop any previous details widget

        w.show();
        w.run(workflow);
    }

    async fn hide_details(&self) {
        self.details_widget.write().unwrap().hide();
    }

    fn open_url(&self) {
        let state = self.state.read().unwrap();
        let idx = match state.table_state.selected() {
            Some(idx) => idx,
            None => return,
        };

        let url = &state.workflow_runs[idx].html_url;
        open::that(url.as_str()).unwrap();
    }
}

impl Widget for &WorkflowRunListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        // a block with a right aligned title with the loading state on the right
        let loading_state = Line::from(format!("{}", state.loading_state)).right_aligned();
        let block = Block::bordered()
            .title("Workflow Runs")
            .title(loading_state)
            .title_bottom("j/k to scroll, q to quit");

        // a table with the list of workflow runs
        let widths = [
            Constraint::Max(50),    // Project
            Constraint::Max(32),    // Branch Name
            Constraint::Max(32),    // Workflow Name
            Constraint::Max(128),   // Commit Title
            Constraint::Max(32),    // Start Time
            Constraint::Length(16), // Status
            Constraint::Length(16), // Completion
        ];

        let header = Row::new(vec![
            "Project",
            "Branch",
            "Workflow Name",
            "Commit Title",
            "Start Time",
            "Status",
            "Completion",
        ])
        .style(Style::new().bold());

        let rows = state.workflow_runs.iter();

        let table = Table::new(rows, widths)
            .header(header)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>")
            .row_highlight_style(Style::new().on_blue());

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        let details_widget = self.details_widget.read().unwrap();
        if details_widget.is_visible() {
            let centered_area =
                area.centered(Constraint::Percentage(75), Constraint::Percentage(75));

            Widget::render(Clear, centered_area, buf);
            Widget::render(details_widget.deref(), centered_area, buf);
        }
    }
}

impl From<&WorkflowRun> for Row<'_> {
    fn from(r: &WorkflowRun) -> Self {
        let r = r.clone();
        Row::new(vec![
            format!("{}/{}", r.owner, r.repo),
            r.branch,
            r.name,
            r.commit_message.split('\n').next().unwrap().to_string(),
            r.start_time
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            r.status.to_string(),
            r.conclusion.to_string(),
        ])
    }
}
