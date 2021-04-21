use crate::Error;
use bitbucket::{get_current_repo_id, Client, PullRequest};
use git2::Repository;
use prettytable::{cell, row, Cell, Table};
use std::collections::HashMap;

mod jira_client;
use jira_client::JiraClient;

pub struct Prs;

impl Prs {
    pub async fn handle(args: std::env::Args, repo: Repository) -> Result<(), Error> {
        let repo_id = get_current_repo_id(&repo).ok_or(Error::InvalidRepo)?;

        let mut args = args;
        let author = if let Some(arg) = args.next() {
            if &arg == "my" {
                let (username, _) = auth::credentials();
                Some(username)
            } else {
                None
            }
        } else {
            None
        };

        let client = Client::new();
        let prs = client.find_open_prs(&repo_id, author).await?;

        Self::print_table_for_prs(&prs.values).await;

        Ok(())
    }

    pub async fn print_table_for_prs(prs: &[PullRequest]) {
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

        let tickets = Self::get_tickets_statuses_for_prs(&prs)
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

            if pr
                .reviewers
                .iter()
                .any(|reviewer| &reviewer.user.name == "devops" && reviewer.approved)
            {
                row.add_cell(cell!(Fg->"A"));
            } else {
                row.add_cell(cell!(Fr->"X"));
            }

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

            let status = super::Ticket::extract_ticket(&pr.from_ref.id)
                .ok()
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

    async fn get_tickets_statuses_for_prs(prs: &[PullRequest]) -> Option<HashMap<String, String>> {
        let jira_client = JiraClient::new();

        let mut tickets = prs
            .iter()
            .filter_map(|pr| super::Ticket::extract_ticket(&pr.from_ref.id).ok())
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
