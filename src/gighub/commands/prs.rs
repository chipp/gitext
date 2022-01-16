use crate::common_git::{AuthDomainConfig, BaseUrlConfig};
use crate::github::{get_current_repo_id, Client, Conclusion, PullRequest, RepoId, State, Status};

use crate::Error;

use futures::{stream, StreamExt};
use git2::Repository;
use prettytable::{cell, row, Table};

pub struct Prs;

impl Prs {
    pub async fn handle<Conf>(repo: &Repository, config: &Conf) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, config).ok_or(Error::InvalidRepo)?;
        let client = Client::new(config);

        let prs = client.find_open_prs(&repo_id).await?;
        if prs.is_empty() {
            println!("No open PRs for that repo");
            return Ok(());
        }

        Self::print_table_for_prs(&prs, false, &repo_id, config).await;

        Ok(())
    }

    pub async fn print_table_for_prs<Conf>(
        prs: &[PullRequest],
        show_status: bool,
        repo_id: &RepoId,
        config: &Conf,
    ) where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let mut table = Table::new();

        {
            let mut row = row![];
            row.add_cell(cell!("ID"));
            row.add_cell(cell!("Author"));
            row.add_cell(cell!("Title"));
            row.add_cell(cell!("CI"));
            row.add_cell(cell!("Target"));
            row.add_cell(cell!("Last updated"));

            if show_status {
                row.add_cell(cell!("Status"));
            }

            table.set_titles(row);
        }

        let client = Client::new(config);

        let statuses = stream::iter(
            prs.iter()
                .map(|pr| Self::pr_checks_status(pr, repo_id, &client)),
        )
        .buffered(10)
        .collect::<Vec<_>>()
        .await;

        for (pr, status) in prs.into_iter().zip(statuses.into_iter()) {
            let mut row = row![pr.number, pr.user.login, title_for_pr(&pr, 35)];

            let updated = pr.updated_at - chrono::Utc::now();
            let updated = chrono_humanize::HumanTime::from(updated);

            match status {
                Some(ChecksStatus::Passed) => row.add_cell(cell!(Fg->"A")),
                Some(ChecksStatus::InProgress) => row.add_cell(cell!(Fy->"I")),
                Some(ChecksStatus::Failed) => row.add_cell(cell!(Fr->"X")),
                None => row.add_cell(cell!(" ")),
            }

            row.add_cell(cell!(pr.base.label));
            row.add_cell(cell!(updated));

            if show_status {
                match status_for_pr(&pr) {
                    PullRequestStatus::Open => row.add_cell(cell!(Fy->"Open")),
                    PullRequestStatus::Merged => row.add_cell(cell!(Fg->"Merged")),
                    PullRequestStatus::Closed => row.add_cell(cell!(Fr->"Closed")),
                }
            }

            table.add_row(row);
        }

        table.printstd();
    }

    async fn pr_checks_status(
        pr: &PullRequest,
        repo_id: &RepoId,
        client: &Client<'_>,
    ) -> Option<ChecksStatus> {
        let response = client
            .get_commit_check_suites(repo_id, pr.head.sha.as_str())
            .await
            .ok()?;

        let mut checks = response.check_suites;
        checks.sort_unstable_by_key(|c| c.created_at);

        let check = checks.last()?;

        match (&check.status, check.conclusion.as_ref()) {
            (Status::Queued, _) | (Status::InProgress, _) => Some(ChecksStatus::InProgress),
            (Status::Completed, Some(Conclusion::Success)) => Some(ChecksStatus::Passed),
            (Status::Completed, _) => Some(ChecksStatus::Failed),
        }
    }
}

enum ChecksStatus {
    Passed,
    Failed,
    InProgress,
}

enum PullRequestStatus {
    Open,
    Merged,
    Closed,
}

fn title_for_pr(pr: &PullRequest, max_width: usize) -> String {
    use hyphenation::{Language, Load, Standard};
    use textwrap::{fill, Options, WordSplitter::Hyphenation};

    let hyphenator = Standard::from_embedded(Language::EnglishUS).unwrap();
    let options = Options::new(max_width).word_splitter(Hyphenation(hyphenator));

    fill(&pr.title, options)
}

fn status_for_pr(pr: &PullRequest) -> PullRequestStatus {
    match pr.state {
        State::Open => PullRequestStatus::Open,
        State::Closed if pr.merged_at.is_some() => PullRequestStatus::Merged,
        State::Closed => PullRequestStatus::Closed,
    }
}
