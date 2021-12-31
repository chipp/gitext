use crate::Error;
use bitbucket::get_current_repo_id;
use common_git::{get_current_branch, BaseUrlConfig, JiraUrlConfig};
use git2::Repository;
use regex::Regex;
use std::process::{Command, Stdio};

pub struct Ticket;

impl Ticket {
    pub async fn handle<Conf>(
        _args: std::env::Args,
        repo: Repository,
        config: Conf,
    ) -> Result<(), Error>
    where
        Conf: BaseUrlConfig,
        Conf: JiraUrlConfig,
    {
        let _ = get_current_repo_id(&repo, &config).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let ticket = Ticket::extract_ticket(&branch)?;

        let mut url = config
            .jira_url()
            .cloned()
            .ok_or(Error::JiraUrlNotConfigured)?;

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("browse");
            segments.push(ticket);
        }

        Command::new("open")
            .arg(url.as_str())
            .stdout(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|err| Error::OpenUrl(err, url))
    }

    pub fn extract_ticket<'b>(branch: &'b str) -> Result<&'b str, Error> {
        let re = Regex::new(r"\w{2,}-\d+").unwrap();
        re.captures(&branch)
            .map(|caps| caps.get(0).unwrap().as_str())
            .ok_or(Error::NoJiraTicket(branch.to_string()))
    }
}
