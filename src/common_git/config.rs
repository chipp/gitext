use std::{collections::HashMap, error::Error as StdError};

use git2::{Config as GitConfig, Repository};
use url::Url;

#[derive(Debug)]
pub struct Config {
    pub provider: Provider,

    pub base_url: Url,
    pub auth_domain: String,

    pub jira_url: Option<Url>,
    pub jira_auth_domain: Option<String>,
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

pub trait JiraAuthDomainConfig {
    fn jira_auth_domain(&self) -> Option<&str>;
}

impl JiraAuthDomainConfig for Config {
    fn jira_auth_domain(&self) -> Option<&str> {
        self.jira_auth_domain.as_ref().map(String::as_str)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Provider {
    BitBucket,
    GitLab,
    GitHub,
}

impl Provider {
    fn parse_from_str(raw: &str) -> Option<Self> {
        match raw.to_lowercase().as_str() {
            "bitbucket" => Some(Provider::BitBucket),
            "gitlab" => Some(Provider::GitLab),
            "github" => Some(Provider::GitHub),
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
        .get_string("gitext.provider")
        .map_err(|_| GetConfigError::ProviderNotSpecified)?;
    let provider =
        Provider::parse_from_str(&provider).ok_or(GetConfigError::UnknownProvider(provider))?;

    let base_url = config.get_string("gitext.baseurl").or_else(|_| {
        if let Provider::GitHub = provider {
            Ok("https://github.com".to_string())
        } else {
            Err(GetConfigError::BaseUrlNotSpecified)
        }
    })?;

    let base_url = Url::parse(&base_url).map_err(|_| GetConfigError::InvalidBaseUrl(base_url))?;

    if base_url.host().is_none() {
        return Err(GetConfigError::InvalidBaseUrl(base_url.into()));
    }

    let auth_domain = config
        .get_string("gitext.authdomain")
        .unwrap_or(String::from(base_url.host_str().unwrap()));

    let jira_url = config.get_string("gitext.jiraurl").ok();
    let jira_url = jira_url.and_then(|string| Url::parse(&string).ok());

    let jira_auth_domain = config.get_string("gitext.jiraauthdomain").ok().or(jira_url
        .as_ref()
        .map(|url| String::from(url.host_str().unwrap())));

    Ok(Config {
        provider,
        base_url,
        auth_domain,
        jira_url,
        jira_auth_domain,
    })
}

pub fn get_aliases_from_config(config: &GitConfig) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    let mut entries = config.entries(Some("alias.*")).unwrap();

    while let Some(Ok(entry)) = entries.next() {
        if let (Some(name), Some(value)) = (entry.name(), entry.value()) {
            if let Some(name) = name.strip_prefix("alias.") {
                aliases.insert(name.to_string(), value.to_string());
            }
        }
    }

    aliases
}
