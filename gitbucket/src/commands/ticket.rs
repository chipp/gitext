use crate::Error;
use bitbucket::{get_current_branch, get_current_repo_id};
use git2::Repository;
use regex::Regex;
use std::process::{Command, Stdio};
use url::Url;

pub struct Ticket;

const JIRA_URL: &str = "https://jira.company.com";

impl Ticket {
    pub async fn handle(_args: std::env::Args, repo: Repository) -> Result<(), Error> {
        let _ = get_current_repo_id(&repo).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let ticket = Ticket::extract_ticket(&branch)?;

        let mut url = Url::parse(JIRA_URL).unwrap();

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

    fn extract_ticket<'b>(branch: &'b str) -> Result<&'b str, Error> {
        let re = Regex::new(r"\w{2,}-\d+").unwrap();
        re.captures(&branch)
            .map(|caps| caps.get(0).unwrap().as_str())
            .ok_or(Error::NoJiraTicket(branch.to_string()))
    }
}
