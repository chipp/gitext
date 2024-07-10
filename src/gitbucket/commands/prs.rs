use std::collections::HashMap;

use crate::bitbucket::{get_current_repo_id, Client, MergedBuildStatus, PullRequest};
use crate::error::Error;
use crate::git::{extract_ticket, AuthDomainConfig, BaseUrlConfig, JiraUrlConfig};

use clap::ArgMatches;
use git2::Repository;
use jira_api::JiraClient;
use prettytable::{cell, row, Cell, Table};

pub struct Prs;

impl Prs {
    pub async fn handle<Conf>(
        args: &ArgMatches,
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, config).ok_or(Error::InvalidRepo)?;

        // TODO: handle `assigned` and username
        let author = match args.get_one::<String>("filter").map(String::as_str) {
            Some("my") => {
                let (username, _) = chipp_auth::user_and_password(config.auth_domain());
                Some(username)
            }
            Some("assigned") => unimplemented!(),
            Some(_username) => unimplemented!(),
            None => None,
        };

        let client = Client::new(config);
        let prs = client.find_open_prs(&repo_id, author).await?;

        let shas = prs
            .values
            .iter()
            .map(|pr| pr.from_ref.latest_commit.as_str())
            .collect::<Vec<_>>();

        let build_statuses = client
            .get_commits_build_stats(&shas)
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect::<HashMap<_, _>>();

        Self::print_table_for_prs(&prs.values, build_statuses, config).await;

        Ok(())
    }

    pub async fn print_table_for_prs<Conf>(
        prs: &[PullRequest],
        build_statuses: HashMap<String, MergedBuildStatus>,
        config: &Conf,
    ) where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraUrlConfig,
    {
        if prs.is_empty() {
            println!("No PRs for that branch");
            return;
        }

        let mut table = Table::new();
        table.set_titles(row![
            "ID",
            "Author",
            "Title",
            "CI",
            "Approvals",
            "Target",
            "Last updated",
            "Jira status"
        ]);

        let tickets = Self::get_tickets_statuses_for_prs(&prs, config)
            .await
            .unwrap_or_default();
        let na = String::from("N/A");

        for pr in prs {
            let mut row = row![
                pr.id,
                pr.author.user.display_name,
                Self::title_for_pr(&pr, 35)
            ];

            let updated = pr.updated - chrono::Utc::now();
            let updated = chrono_humanize::HumanTime::from(updated);

            match build_statuses.get(&pr.from_ref.latest_commit) {
                Some(MergedBuildStatus::Success) => row.add_cell(cell!(Fg->"A")),
                Some(MergedBuildStatus::InProgress) => row.add_cell(cell!(Fy->"I")),
                Some(MergedBuildStatus::Failed) | None => row.add_cell(cell!(Fr->"X")),
            };

            let approvals = pr
                .reviewers
                .iter()
                .filter(|reviewer| {
                    &reviewer.user.name != "devops"
                        && &reviewer.user.name != "ci"
                        && reviewer.approved
                })
                .collect::<Vec<_>>()
                .len();

            let reviewers = pr
                .reviewers
                .iter()
                .filter(|reviewer| &reviewer.user.name != "devops" && &reviewer.user.name != "ci")
                .collect::<Vec<_>>()
                .len();

            let approvals_cell = Cell::new(&format!("{}/{}", approvals, reviewers));

            if approvals >= 2 {
                row.add_cell(approvals_cell.style_spec("Fg"));
            } else {
                row.add_cell(approvals_cell.style_spec("Fr"));
            }

            row.add_cell(cell!(pr.to_ref.display_id));
            row.add_cell(cell!(updated));

            let status = extract_ticket(&pr.from_ref.id)
                .and_then(|ticket| tickets.get(ticket))
                .unwrap_or(&na);

            row.add_cell(cell!(status));

            table.add_row(row);
        }

        table.printstd();
    }

    fn title_for_pr(pr: &PullRequest, max_width: usize) -> String {
        use hyphenation::{Language, Load, Standard};
        use textwrap::{fill, Options, WordSplitter::Hyphenation};

        let hyphenator = Standard::from_embedded(Language::EnglishUS).unwrap();
        let options = Options::new(max_width).word_splitter(Hyphenation(hyphenator));

        fill(&pr.title, options)
    }

    async fn get_tickets_statuses_for_prs<Conf>(
        prs: &[PullRequest],
        config: &Conf,
    ) -> Option<HashMap<String, String>>
    where
        Conf: JiraUrlConfig,
    {
        let jira_url = config.jira_url()?;
        let jira_client = JiraClient::new(jira_url, jira_api::client::AuthType::AccessToken)?;

        let mut tickets = prs
            .iter()
            .filter_map(|pr| extract_ticket(&pr.from_ref.id))
            .collect::<Vec<_>>();
        tickets.dedup();

        let jql = format!("key in ({})", tickets.join(","));
        let tickets = crate::jira::pull(&jira_client, &jql).await;

        Some(
            tickets
                .into_iter()
                .map(|issue| (issue.key, issue.fields.status.name))
                .collect::<HashMap<_, _>>(),
        )
    }
}
