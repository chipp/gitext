use crate::Error;
use bitbucket::{get_current_repo_id, Client, PullRequest};
use git2::Repository;
use prettytable::{cell, row, Cell, Table};

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

        let mut table = Table::new();
        table.set_titles(row![
            "ID",
            "Author",
            "Title",
            "CI",
            "Approvals",
            "Target",
            "Last updated"
        ]);

        for pr in prs.values {
            let mut row = row![
                pr.id,
                pr.author.user.display_name,
                Self::title_for_pr(&pr, 50)
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

            table.add_row(row);
        }

        table.printstd();

        Ok(())
    }

    fn title_for_pr(pr: &PullRequest, max_width: usize) -> String {
        use textwrap::{NoHyphenation, Wrapper};

        let wrapper = Wrapper::with_splitter(max_width, NoHyphenation);
        wrapper.fill(&pr.title)
    }
}
