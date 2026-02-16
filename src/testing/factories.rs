use fake::faker::chrono::en::*;
use fake::faker::company::en::*;
use fake::faker::lorem::en::*;
use fake::rand::random;
use fake::{Fake, Faker, Rng};
use url::Url;

use crate::models::{WorkflowJob, WorkflowRun};

#[cfg(feature = "mocks")]
impl fake::Dummy<Faker> for WorkflowRun {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, _rng: &mut R) -> Self {
        let run_id = random::<u64>();
        let owner = CompanyName()
            .fake::<String>()
            .replace(' ', "-")
            .to_lowercase();
        let repo = Buzzword().fake::<String>().replace(' ', "-").to_lowercase();
        let url = format!(
            "https://example.org/{}/{}/actions/runs/{}",
            owner, repo, run_id
        );

        Self {
            id: run_id.into(),
            owner,
            repo,
            name: Sentence(2..4).fake(),
            commit_message: format!("fake: {}", Bs().fake::<String>()),
            start_time: DateTime().fake(),
            status: Faker.fake(),
            conclusion: Faker.fake(),
            html_url: Url::parse(&url).unwrap(),
        }
    }
}

impl fake::Dummy<Faker> for WorkflowJob {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, _rng: &mut R) -> Self {
        let job_id = random::<u64>();
        let run_id = random::<u64>();
        let owner = CompanyName()
            .fake::<String>()
            .replace(' ', "-")
            .to_lowercase();
        let repo = Buzzword().fake::<String>().replace(' ', "-").to_lowercase();
        let url = format!(
            "https://example.org/{}/{}/actions/runs/{}/job/{}",
            owner, repo, run_id, job_id
        );

        Self {
            id: job_id.into(),
            name: Sentence(2..4).fake(),
            started_at: DateTime().fake(),
            completed_at: DateTime().fake(),
            status: Faker.fake(),
            conclusion: Faker.fake(),
            html_url: Url::parse(&url).unwrap(),
        }
    }
}
