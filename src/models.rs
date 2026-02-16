use std::fmt::Display;

use octocrab::models::workflows::{Conclusion, Job, Run, Status};
use octocrab::models::{JobId, RunId};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub owner: String,
    pub name: String,
    pub branch: Option<String>,
    pub count: Option<u8>,
    pub actor: Option<String>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(any(test, feature = "mocks"), derive(fake::Dummy))]
pub enum WorkflowRunStatus {
    #[default]
    InProgress,
    Completed,
    Other(String),
}

impl Display for WorkflowRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl From<&str> for WorkflowRunStatus {
    fn from(c: &str) -> Self {
        match c {
            "in_progress" => Self::InProgress,
            "completed" => Self::Completed,
            _ => Self::Other(c.to_string()),
        }
    }
}

impl From<&WorkflowRunStatus> for String {
    fn from(v: &WorkflowRunStatus) -> Self {
        match v {
            WorkflowRunStatus::InProgress => "In Progress".to_string(),
            WorkflowRunStatus::Completed => "Completed".to_string(),
            WorkflowRunStatus::Other(c) => c.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(any(test, feature = "mocks"), derive(fake::Dummy))]
pub enum WorkflowRunConclusion {
    #[default]
    Pending,
    Success,
    Failure,
    Other(String),
}

impl Display for WorkflowRunConclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl From<&str> for WorkflowRunConclusion {
    fn from(c: &str) -> Self {
        match c {
            "success" => Self::Success,
            "failure" => Self::Failure,
            _ => Self::Other(c.to_string()),
        }
    }
}

impl From<&WorkflowRunConclusion> for String {
    fn from(v: &WorkflowRunConclusion) -> Self {
        match v {
            WorkflowRunConclusion::Pending => "⌛ Pending".to_string(),
            WorkflowRunConclusion::Success => "✅ Success".to_string(),
            WorkflowRunConclusion::Failure => "❌ Failure".to_string(),
            WorkflowRunConclusion::Other(c) => c.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(any(test, feature = "mocks"), derive(fake::Dummy))]
pub enum WorkflowJobStatus {
    #[default]
    Pending,
    Queued,
    InProgress,
    Completed,
    Failed,
    Other(String),
}

impl Display for WorkflowJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl From<&Status> for WorkflowJobStatus {
    fn from(c: &Status) -> Self {
        match c {
            Status::Pending => Self::Pending,
            Status::Queued => Self::Queued,
            Status::InProgress => Self::InProgress,
            Status::Completed => Self::Completed,
            Status::Failed => Self::Failed,
            _ => Self::Other(format!("{:?}", c)),
        }
    }
}

impl From<&WorkflowJobStatus> for String {
    fn from(v: &WorkflowJobStatus) -> Self {
        match v {
            WorkflowJobStatus::Pending => "Pending".to_string(),
            WorkflowJobStatus::Queued => "Queued".to_string(),
            WorkflowJobStatus::InProgress => "In Progress".to_string(),
            WorkflowJobStatus::Completed => "Completed".to_string(),
            WorkflowJobStatus::Failed => "Failed".to_string(),
            WorkflowJobStatus::Other(c) => c.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(any(test, feature = "mocks"), derive(fake::Dummy))]
pub enum WorkflowJobConclusion {
    ActionRequired,
    Cancelled,
    Failure,
    #[default]
    Neutral,
    Skipped,
    Success,
    TimedOut,
    Other(String),
}

impl Display for WorkflowJobConclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl From<&Conclusion> for WorkflowJobConclusion {
    fn from(c: &Conclusion) -> Self {
        match c {
            Conclusion::ActionRequired => Self::ActionRequired,
            Conclusion::Cancelled => Self::Cancelled,
            Conclusion::Failure => Self::Failure,
            Conclusion::Neutral => Self::Neutral,
            Conclusion::Skipped => Self::Skipped,
            Conclusion::Success => Self::Success,
            Conclusion::TimedOut => Self::TimedOut,
            _ => Self::Other(format!("{:?}", c)),
        }
    }
}

impl From<&WorkflowJobConclusion> for String {
    fn from(v: &WorkflowJobConclusion) -> Self {
        match v {
            WorkflowJobConclusion::ActionRequired => "Action Required".to_string(),
            WorkflowJobConclusion::Cancelled => "🛑 Cancelled".to_string(),
            WorkflowJobConclusion::Failure => "❌ Failure".to_string(),
            WorkflowJobConclusion::Neutral => "Neutral".to_string(),
            WorkflowJobConclusion::Skipped => "⏩ Skipped".to_string(),
            WorkflowJobConclusion::Success => "✅ Success".to_string(),
            WorkflowJobConclusion::TimedOut => "⏱️ Timed Out".to_string(),
            WorkflowJobConclusion::Other(c) => c.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowRun {
    pub id: RunId,
    pub owner: String,
    pub repo: String,
    pub name: String,
    pub commit_message: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub status: WorkflowRunStatus,
    pub conclusion: WorkflowRunConclusion,
    pub html_url: url::Url,
}

impl Display for WorkflowRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WorkflowRun<id={}, repo={}/{} name={}, status={}, conclusion={}, url={}>",
            self.id, self.owner, self.repo, self.name, self.status, self.conclusion, self.html_url
        )
    }
}

impl From<&Run> for WorkflowRun {
    fn from(r: &Run) -> Self {
        let conclusion = r
            .conclusion
            .as_ref()
            .map_or(WorkflowRunConclusion::default(), |c| {
                WorkflowRunConclusion::from(c.as_str())
            });

        let owner = match &r.repository.owner {
            Some(owner) => owner.login.clone(),
            None => String::new(),
        };

        Self {
            id: r.id,
            owner,
            repo: r.repository.name.clone(),
            name: r.name.clone(),
            commit_message: r.head_commit.message.clone(),
            start_time: r.created_at,
            status: WorkflowRunStatus::from(r.status.as_str()),
            conclusion,
            html_url: r.html_url.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowJob {
    pub id: JobId,
    pub name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: WorkflowJobStatus,
    pub conclusion: WorkflowJobConclusion,
    pub html_url: url::Url,
}

impl Display for WorkflowJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WorkflowJob<id={}, name={}, status={}, conclusion={}, url={}>",
            self.id, self.name, self.status, self.conclusion, self.html_url
        )
    }
}

impl From<Job> for WorkflowJob {
    fn from(j: Job) -> Self {
        let conclusion = j.conclusion.as_ref().map_or(
            WorkflowJobConclusion::default(),
            WorkflowJobConclusion::from,
        );

        Self {
            id: j.id,
            name: j.name.clone(),
            started_at: j.started_at,
            completed_at: j.completed_at,
            status: (&j.status).into(),
            conclusion,
            html_url: j.html_url.clone(),
        }
    }
}
