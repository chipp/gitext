use crate::git::{extract_ticket, get_current_branch, BaseUrlConfig, JiraUrlConfig};
use crate::{Error, Result};

use git2::Repository;
use std::process::{Command, Stdio};

pub struct Ticket;

impl Ticket {
    pub fn handle<Conf>(repo: Repository, config: Conf) -> Result<()>
    where
        Conf: BaseUrlConfig,
        Conf: JiraUrlConfig,
    {
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let ticket = extract_ticket(&branch).ok_or(Error::NoJiraTicket(branch.to_string()))?;

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
            .map_err(|err| Error::OpenUrl(err, url))?;

        Ok(())
    }
}
