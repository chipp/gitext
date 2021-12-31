use std::error::Error as StdError;

use git2::Repository;
use url::Url;

#[derive(Debug)]
pub struct Config {
    pub provider: Provider,
    pub base_url: Url,
    pub jira_url: Option<Url>,
    pub auth_domain: String,
}

pub trait BaseUrlConfig {
    fn base_url(&self) -> &Url;
}

impl BaseUrlConfig for Config {
    fn base_url(&self) -> &Url {
        &self.base_url
    }
}

pub trait AuthDomainConfig {
    fn auth_domain(&self) -> &str;
}

impl AuthDomainConfig for Config {
    fn auth_domain(&self) -> &str {
        &self.auth_domain
    }
}

pub trait JiraUrlConfig {
    fn jira_url(&self) -> Option<&Url>;
}

impl JiraUrlConfig for Config {
    fn jira_url(&self) -> Option<&Url> {
        self.jira_url.as_ref()
    }
}

#[derive(Debug)]
pub enum Provider {
    BitBucket,
}

impl Provider {
    fn parse_from_str(raw: &str) -> Option<Self> {
        match raw.to_lowercase().as_str() {
            "bitbucket" => Some(Provider::BitBucket),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum GetConfigError {
    ProviderNotSpecified,
    UnknownProvider(String),
    BaseUrlNotSpecified,
    InvalidBaseUrl(String),
}

impl StdError for GetConfigError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

use std::fmt;
impl fmt::Display for GetConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetConfigError::ProviderNotSpecified => {
                write!(f, "provider is not specified in .git/config")
            }
            GetConfigError::UnknownProvider(value) => {
                write!(
                    f,
                    "unknown provider \"{}\" is specified in .git/config",
                    value
                )
            }
            GetConfigError::BaseUrlNotSpecified => {
                write!(f, "host is not specified in .git/config")
            }
            GetConfigError::InvalidBaseUrl(value) => {
                write!(
                    f,
                    "invalid base_url \"{}\" is specified in .git/config",
                    value
                )
            }
        }
    }
}

pub fn get_config(repo: &Repository) -> Result<Config, GetConfigError> {
    let config = repo.config().unwrap();

    let provider = config
        .get_string("gitbucket.provider")
        .map_err(|_| GetConfigError::ProviderNotSpecified)?;
    let provider =
        Provider::parse_from_str(&provider).ok_or(GetConfigError::UnknownProvider(provider))?;

    let base_url = config
        .get_string("gitbucket.baseurl")
        .map_err(|_| GetConfigError::BaseUrlNotSpecified)?;
    let base_url = Url::parse(&base_url).map_err(|_| GetConfigError::InvalidBaseUrl(base_url))?;

    let jira_url = config.get_string("gitbucket.jiraurl").ok();
    let jira_url = jira_url.and_then(|string| Url::parse(&string).ok());

    if base_url.host().is_none() {
        return Err(GetConfigError::InvalidBaseUrl(base_url.into()));
    }

    let auth_domain = config
        .get_string("gitbucket.authdomain")
        .unwrap_or(String::from(base_url.host_str().unwrap()));

    Ok(Config {
        provider,
        base_url,
        jira_url,
        auth_domain,
    })
}
