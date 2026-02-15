use std::panic;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};
use exn::{Result, ResultExt};
use log::error;
use octocrab::Octocrab;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::{DefaultTerminal, Frame};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::error::AppError;
use crate::service::workflows;
use crate::service::workflows::GitHubService;
use crate::widgets::workflow_run::WorkflowRunListWidget;

mod configuration;
mod error;
mod models;
mod service;
#[cfg(any(test, feature = "mocks"))]
mod testing;
mod widgets;

fn make_error() -> AppError {
    AppError::from("failed to run app")
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    log_rs::from_env().expect("Unable to initialize log from env");

    let cfg = configuration::get_configuration().expect("Unable to read configuration");
    if cfg.repos.is_empty() {
        error!("No repositories configured, exiting");
        return Ok(());
    }

    init_github_client(&cfg).await?;

    color_eyre::install()
        .map_err(AppError::from_color_eyre)
        .or_raise(make_error)?;
    let terminal = ratatui::init();
    let app_result = App::new(cfg).run(terminal).await;
    ratatui::restore();

    app_result
        .map_err(AppError::from_color_eyre)
        .or_raise(make_error)
}

async fn init_github_client(cfg: &configuration::Settings) -> Result<(), AppError> {
    let token = cfg.token().or_raise(make_error)?;

    let crab = Octocrab::builder()
        .base_uri(format!("https://api.{}", cfg.host))
        .or_raise(make_error)?
        .user_access_token(token)
        .build()
        .unwrap();

    // Validate the token
    crab.current().user().await.or_raise(make_error)?;

    octocrab::initialise(crab);

    Ok(())
}

#[derive(Debug, Default)]
struct App {
    should_quit: bool,
    workflow_run_widgets: WorkflowRunListWidget,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    fn new(config: configuration::Settings) -> Self {
        let github_service = get_github_service();

        Self {
            workflow_run_widgets: WorkflowRunListWidget::new(github_service, config.repos),
            ..Default::default()
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let tx = self.workflow_run_widgets.run();

        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.render(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event, &tx).await,
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let [title_area, body_area] = frame.area().layout(&layout);
        let title = Line::from("GitHub Workflow Dashboard").centered().bold();
        frame.render_widget(title, title_area);
        frame.render_widget(&self.workflow_run_widgets, body_area);
    }

    async fn handle_event(&mut self, event: &Event, tx: &mpsc::Sender<Event>) {
        if let Some(key) = event.as_key_press_event() {
            #[allow(clippy::collapsible_if)]
            if let KeyCode::Char('q') = key.code {
                self.should_quit = true
            }
        }

        if let Err(e) = tx.send(event.clone()).await {
            error!("Failed to send event to workflow run widget: {}", e);
        }
    }
}

#[cfg(feature = "mocks")]
fn get_github_service() -> Arc<dyn GitHubService> {
    let mut svc = workflows::MockGitHubService::new();

    svc.expect_list_runs().returning(|_| {
        use fake::Fake;
        use fake::rand::random;

        let n = random::<u8>() % 16;

        let workflow_runs = (0..=n).map(|_| fake::Faker.fake()).collect();

        Ok(workflow_runs)
    });

    svc.expect_list_jobs().returning(|_| {
        use fake::Fake;
        use fake::rand::random;

        let n = random::<u8>() % 16;

        let workflow_jobs = (0..=n).map(|_| fake::Faker.fake()).collect();

        Ok(workflow_jobs)
    });

    Arc::new(svc)
}

#[cfg(not(feature = "mocks"))]
fn get_github_service() -> Arc<dyn GitHubService> {
    Arc::new(workflows::Service {})
}
