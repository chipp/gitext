use git2::Error as GitError;
use http_client::Error as HttpError;
use std::error::Error as StdError;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    AuthorizationError,

    UnknownCommand(String),
    UnknownSubCommand(String, &'static [&'static str]),
    InvalidRepo,
    Detached,

    Git(GitError),
    Http(HttpError),

    OpenUrl(IoError, url::Url),
    NoJiraTicket(String),

    NoPrsForBranch(String, HttpError),
    NoOpenPrsForBranch(String),
    NoPrWithId(u16, HttpError),
    InvalidPrId(String),
}

impl From<GitError> for Error {
    fn from(err: GitError) -> Error {
        Error::Git(err)
    }
}

impl From<HttpError> for Error {
    fn from(err: HttpError) -> Error {
        Error::Http(err)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use Error::*;

        match self {
            Git(err) => Some(err),
            Http(err) => Some(err),
            OpenUrl(err, _) => Some(err),
            NoPrsForBranch(_, err) => Some(err),
            NoPrWithId(_, err) => Some(err),
            _ => None,
        }
    }
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            AuthorizationError => write!(f, "token is invalid"),
            UnknownCommand(cmd) => write!(f, "unknown command {}", cmd),
            UnknownSubCommand(sub, supported) => write!(
                f,
                "unknown sub-command {}. supported sub-commands: {}",
                sub,
                supported.join(", ")
            ),
            InvalidRepo => write!(f, "this is not a bitbucket repository"),
            Detached => write!(f, "can't find the current branch"),

            Git(err) => write!(f, "{}", err),
            Http(err) => write!(f, "{}", err),

            OpenUrl(err, url) => write!(f, "can't open URL {}: {}", url, err),
            NoJiraTicket(branch) => {
                write!(f, "can't find JIRA ticket in branch name \"{}\"", branch)
            }

            NoPrsForBranch(branch, err) => {
                write!(f, "can't find prs for branch {}: {}", branch, err)
            }
            NoOpenPrsForBranch(branch) => {
                write!(f, "there are no any open prs for branch {}", branch)
            }

            NoPrWithId(id, err) => write!(f, "can't find pr with id {}: {}", id, err),
            InvalidPrId(id) => write!(f, "invalid PR id \"{}\"\nusage: git pr #42", id),
        }
    }
}
