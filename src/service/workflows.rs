use std::fmt::{Debug, Formatter};

use async_trait::async_trait;
use exn::{Result, ResultExt};
use log::error;
#[cfg(any(test, feature = "mocks"))]
use mockall::automock;
use tokio::task::JoinSet;

use crate::error::ServiceError;
use crate::models::{Repository, WorkflowJob, WorkflowRun};

#[cfg_attr(any(test, feature = "mocks"), automock)]
#[async_trait]
pub trait GitHubService: Debug + Send + Sync {
    async fn list_runs(&self, repos: &[Repository]) -> Result<Vec<WorkflowRun>, ServiceError>;

    async fn list_jobs(&self, workflow: &WorkflowRun) -> Result<Vec<WorkflowJob>, ServiceError>;
}

pub struct Service {}

impl Debug for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Service{{}}")
    }
}

#[async_trait]
impl GitHubService for Service {
    async fn list_runs(&self, repos: &[Repository]) -> Result<Vec<WorkflowRun>, ServiceError> {
        let make_error = || ServiceError::from("Error getting workflow runs");

        let mut set = JoinSet::new();

        repos.iter().for_each(|repo| {
            set.spawn(get_last_run_for_repo(repo.clone()));
        });

        let mut workflows = vec![];

        while let Some(res) = set.join_next().await {
            let octo_res = res.or_raise(make_error)?;

            let workflow = match octo_res {
                Ok(wf) => wf,
                Err(e) => {
                    error!("Failed to get workflow runs for repo: {:?}", e);
                    continue;
                }
            };

            if let Some(w) = workflow {
                workflows.push(w);
            }
        }

        workflows.sort_by(|a, b| Ord::cmp(&a.start_time, &b.start_time).reverse());

        Ok(workflows)
    }

    async fn list_jobs(&self, workflow: &WorkflowRun) -> Result<Vec<WorkflowJob>, ServiceError> {
        let make_error = || ServiceError::from("Error getting workflow job");

        let jobs = octocrab::instance()
            .workflows(&workflow.owner, &workflow.repo)
            .list_jobs(workflow.id)
            .send()
            .await
            .or_raise(make_error)?;

        Ok(jobs.into_iter().map(Into::into).collect())
    }
}

async fn get_last_run_for_repo(repo: Repository) -> octocrab::Result<Option<WorkflowRun>> {
    let workflows = octocrab::instance()
        .workflows(repo.owner, repo.name)
        .list_all_runs()
        .branch(repo.branch.unwrap_or("main".into()))
        .per_page(1)
        .send()
        .await?;

    Ok(workflows.items.first().map(Into::into))
}
