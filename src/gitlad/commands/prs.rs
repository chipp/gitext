use std::collections::HashMap;

use crate::common_git::{
    extract_ticket, AuthDomainConfig, BaseUrlConfig, JiraAuthDomainConfig, JiraUrlConfig,
};
use crate::gitlab::{get_current_repo_id, Client, PullRequest};
use crate::jira::JiraClient;
use crate::Error;

use git2::Repository;
use prettytable::{cell, row, Cell, Table};

pub struct Prs;

impl Prs {
    pub async fn handle<Conf>(
        args: std::env::Args,
        repo: Repository,
        config: Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
        Conf: JiraUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, &config).ok_or(Error::InvalidRepo)?;
        let client = Client::new(&config);

        let mut args = args;
        let author = if let Some("my") = args.next().as_ref().map(AsRef::<str>::as_ref) {
            let user = client.whoami().await?;
            Some(user.name)
        } else {
            None
        };

        let prs = client.find_open_prs(&repo_id, author).await?;
        Self::print_table_for_prs(&prs, &config).await;

        Ok(())
    }

    pub async fn print_table_for_prs<Conf>(prs: &[PullRequest], config: &Conf)
    where
        Conf: JiraUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
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
            "Likes",
            "Target",
            "Last updated",
            "Jira status"
        ]);

        let tickets = Self::get_tickets_statuses_for_prs(&prs, config)
            .await
            .unwrap_or_default();
        let na = String::from("N/A");

        for pr in prs {
            let mut row = row![pr.id, pr.author.display_name, Self::title_for_pr(&pr, 35)];

            let updated = pr.updated - chrono::Utc::now();
            let updated = chrono_humanize::HumanTime::from(updated);

            if pr.labels.iter().any(|label| label == "CI OK") {
                row.add_cell(cell!(Fg->"A"));
            } else if pr.labels.iter().any(|label| label == "CI FAILED") {
                row.add_cell(cell!(Fr->"X"));
            } else {
                row.add_cell(cell!(Fy->"I"));
            }

            let upvotes = pr.upvotes;
            let upvotes_cell = Cell::new(&format!("{}", upvotes));

            if upvotes >= 2 {
                row.add_cell(upvotes_cell.style_spec("Fg"));
            } else {
                row.add_cell(upvotes_cell.style_spec("Fr"));
            }

            row.add_cell(cell!(pr.target_branch));
            row.add_cell(cell!(updated));

            let status = extract_ticket(&pr.source_branch)
                .and_then(|ticket| tickets.get(ticket))
                .unwrap_or(&na);

            row.add_cell(cell!(status));

            table.add_row(row);
        }

        table.printstd();
    }

    fn title_for_pr(pr: &PullRequest, max_width: usize) -> String {
        use textwrap::{NoHyphenation, Wrapper};

        let wrapper = Wrapper::with_splitter(max_width, NoHyphenation);
        wrapper.fill(&pr.title)
    }

    async fn get_tickets_statuses_for_prs<Conf>(
        prs: &[PullRequest],
        config: &Conf,
    ) -> Option<HashMap<String, String>>
    where
        Conf: JiraUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
    {
        let jira_client = JiraClient::new(config).ok()?;

        let mut tickets = prs
            .iter()
            .filter_map(|pr| extract_ticket(&pr.source_branch))
            .collect::<Vec<_>>();
        tickets.dedup();

        let tickets = jira_client
            .search_issues(
                format!("key in ({})", tickets.join(",")),
                0,
                100,
                Some(&["status"]),
            )
            .await
            .ok()?;

        Some(
            tickets
                .issues
                .into_iter()
                .map(|issue| (issue.key, issue.fields.status.name))
                .collect::<HashMap<_, _>>(),
        )
    }
}
