use std::collections::HashMap;

use crate::common_git::{
    extract_ticket, AuthDomainConfig, BaseUrlConfig, JiraAuthDomainConfig, JiraUrlConfig,
};
use crate::gitlab::{get_current_repo_id, Client, Pipeline, PipelineStatus, PullRequest, RepoId};
use crate::jira::JiraClient;
use crate::Error;

use futures_util::{stream, StreamExt};
use git2::Repository;
use prettytable::{cell, row, Cell, Table};

enum FilterMode {
    ByAuthor(u16),
    ByAssignee(u16),
}

pub struct Prs;

impl Prs {
    pub async fn handle<Arg: AsRef<str>, Conf>(
        args: &[Arg],
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
        Conf: JiraUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, config).ok_or(Error::InvalidRepo)?;
        let client = Client::new(config);

        let filter_mode = Self::filter_mode(args, &client).await;

        let prs = Self::find_all_open_prs(&client, &repo_id, filter_mode).await?;
        if prs.is_empty() {
            println!("No open PRs in that repo");
            return Ok(());
        }

        Self::print_table_for_prs(&prs, &repo_id, config).await;

        Ok(())
    }

    pub async fn print_table_for_prs<Conf>(prs: &[PullRequest], repo_id: &RepoId, config: &Conf)
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
    {
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
        let pipelines = Self::get_last_pipelines_for_prs(&prs, &repo_id, config).await;

        let na = String::from("N/A");

        for pr in prs {
            let mut row = row![pr.id, pr.author.display_name, Self::title_for_pr(&pr, 35)];

            let updated = pr.updated - chrono::Utc::now();
            let updated = chrono_humanize::HumanTime::from(updated);

            match pipelines.get(&pr.id) {
                Some(pipeline) => match pipeline.status {
                    PipelineStatus::Pending => row.add_cell(cell!(Fy->"P")),
                    PipelineStatus::Running => row.add_cell(cell!(Fy->"R")),
                    PipelineStatus::Success => row.add_cell(cell!(Fg->"S")),
                    PipelineStatus::Failed => row.add_cell(cell!(Fr->"F")),
                },
                None => row.add_cell(cell!("")),
            };

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

    async fn filter_mode<Arg: AsRef<str>>(args: &[Arg], client: &Client<'_>) -> Option<FilterMode> {
        match args.first().map(AsRef::<_>::as_ref) {
            Some("my") => {
                let user = client.whoami().await.ok()?;
                Some(FilterMode::ByAuthor(user.id))
            }
            Some("assigned") => {
                let user = client.whoami().await.ok()?;
                Some(FilterMode::ByAssignee(user.id))
            }
            Some(username) => {
                let users = client.get_user_by_name(username).await.ok()?;
                let user = users.first()?;
                Some(FilterMode::ByAuthor(user.id))
            }
            None => None,
        }
    }

    fn title_for_pr(pr: &PullRequest, max_width: usize) -> String {
        use hyphenation::{Language, Load, Standard};
        use textwrap::{fill, Options, WordSplitter::Hyphenation};

        let hyphenator = Standard::from_embedded(Language::EnglishUS).unwrap();
        let options = Options::new(max_width).word_splitter(Hyphenation(hyphenator));

        fill(&pr.title, options)
    }

    async fn get_last_pipelines_for_prs<Conf>(
        prs: &[PullRequest],
        repo_id: &RepoId,
        config: &Conf,
    ) -> HashMap<u16, Pipeline>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let client = Client::new(config);

        let pipelines = stream::iter(
            prs.iter()
                .map(|pr| client.get_last_pipeline_for_branch(&pr.source_branch, &repo_id)),
        )
        .buffered(10)
        .collect::<Vec<_>>()
        .await;

        let mut result = HashMap::new();

        for (pr, pipeline) in prs.iter().zip(pipelines.into_iter()) {
            if let Ok(pipeline) = pipeline {
                result.insert(pr.id, pipeline);
            }
        }

        result
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

    async fn find_all_open_prs(
        client: &Client<'_>,
        repo_id: &RepoId,
        filter_mode: Option<FilterMode>,
    ) -> Result<Vec<PullRequest>, Error> {
        let mut result = vec![];
        let mut page = 1;

        let (author, assignee) = match filter_mode {
            Some(FilterMode::ByAuthor(id)) => (Some(id), None),
            Some(FilterMode::ByAssignee(id)) => (None, Some(id)),
            None => (None, None),
        };

        loop {
            let prs = client
                .find_open_prs(&repo_id, author, assignee, page)
                .await?;
            if prs.is_empty() {
                return Ok(result);
            } else {
                result.extend(prs);
                page += 1;
            }
        }
    }
}
