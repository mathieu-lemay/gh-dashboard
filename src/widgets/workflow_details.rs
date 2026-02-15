use std::sync::{Arc, RwLock};
use std::time::Duration;

use exn::Exn;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, HighlightSpacing, Row, StatefulWidget, Table, TableState, Widget};
use tokio::time;

use crate::error::ServiceError;
use crate::models::{WorkflowJob, WorkflowRun};
use crate::service::workflows::{GitHubService, Service};
use crate::widgets::state::LoadingState;

#[derive(Debug, Default)]
struct WorkflowDetailsState {
    workflow_jobs: Vec<WorkflowJob>,
    loading_state: LoadingState,
    table_state: TableState,
}

#[derive(Debug, Clone)]
pub struct WorkflowDetailsWidget {
    github_service: Arc<dyn GitHubService + Sync + Send>,
    state: Arc<RwLock<WorkflowDetailsState>>,
    visible: bool,
}

impl Default for WorkflowDetailsWidget {
    fn default() -> Self {
        Self {
            github_service: Arc::new(Service {}),
            state: Arc::new(RwLock::new(WorkflowDetailsState::default())),
            visible: false,
        }
    }
}

impl WorkflowDetailsWidget {
    pub fn new(github_service: Arc<dyn GitHubService + Sync + Send>) -> Self {
        Self {
            github_service,
            ..Default::default()
        }
    }

    pub fn run(&mut self, workflow: WorkflowRun) {
        let this = self.clone();
        tokio::spawn(this.sync_data(workflow));
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        let mut state = self.state.write().unwrap();
        state.workflow_jobs.clear();

        self.visible = false;
    }

    async fn sync_data(self, workflow: WorkflowRun) {
        let period = Duration::from_secs(60);
        let mut interval = time::interval(period);

        loop {
            interval.tick().await;
            if !self.visible {
                return;
            }
            self.fetch_workflow_jobs(&workflow).await;
        }
    }

    async fn fetch_workflow_jobs(&self, workflow: &WorkflowRun) {
        self.set_loading_state(LoadingState::Loading);

        let jobs = self.github_service.list_jobs(workflow).await;

        match jobs {
            Ok(j) => self.on_load(j),
            Err(err) => self.on_err(&err),
        }
    }

    fn on_load(&self, jobs: Vec<WorkflowJob>) {
        let mut state = self.state.write().unwrap();

        state.workflow_jobs = jobs;

        state.loading_state = LoadingState::Loaded(chrono::Local::now());
    }

    fn on_err(&self, err: &Exn<ServiceError>) {
        self.set_loading_state(LoadingState::Error(err.to_string()));
    }

    fn set_loading_state(&self, state: LoadingState) {
        self.state.write().unwrap().loading_state = state;
    }
}

impl Widget for &WorkflowDetailsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let loading_state = Line::from(format!("{}", state.loading_state)).right_aligned();
        let block = Block::bordered()
            .title("Workflow Jobs")
            .title(loading_state)
            .title_bottom("esc to close");

        let widths = [
            Constraint::Max(120),   // Job Name
            Constraint::Max(32),    // Started At
            Constraint::Max(32),    // Completed At
            Constraint::Length(16), // Status
            Constraint::Length(16), // Completion
        ];

        let header = Row::new(vec![
            "Job Name",
            "Started At",
            "Completed At",
            "Status",
            "Conclusion",
        ])
        .style(Style::new().bold());

        let rows = state.workflow_jobs.iter();

        let table = Table::new(rows, widths)
            .header(header)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>")
            .row_highlight_style(Style::new().on_blue());

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}

impl From<&WorkflowJob> for Row<'_> {
    fn from(r: &WorkflowJob) -> Self {
        let j = r.clone();
        Row::new(vec![
            j.name,
            j.started_at
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            j.completed_at.map_or("".to_string(), |t| {
                t.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            }),
            j.status.to_string(),
            j.conclusion.to_string(),
        ])
        .height(2)
    }
}
