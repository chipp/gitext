use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use git2::{Config as GitConfig, Repository};
use url::Url;

#[derive(Debug)]
pub struct Config {
    pub provider: Provider,

    pub base_url: Url,
    pub auth_domain: String,

    pub jira_url: Option<Url>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: Provider::GitHub,
            base_url: Url::parse("https://github.com").unwrap(),
            auth_domain: "github.com".to_string(),
            jira_url: None,
        }
    }
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

#[derive(Copy, Clone, Debug)]
pub enum Provider {
    BitBucket,
    GitLab,
    GitHub,
}

impl FromStr for Provider {
    type Err = ConfigError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.to_lowercase().as_str() {
            "bitbucket" => Ok(Provider::BitBucket),
            "gitlab" => Ok(Provider::GitLab),
            "github" => Ok(Provider::GitHub),
            _ => Err(ConfigError::UnknownProvider(raw.to_string())),
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::BitBucket => write!(f, "bitbucket"),
            Provider::GitLab => write!(f, "gitlab"),
            Provider::GitHub => write!(f, "github"),
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    ProviderNotSpecified,
    UnknownProvider(String),
    BaseUrlNotSpecified,
    InvalidBaseUrl(String),
    UnableToUpdateConfig(String),
}

impl StdError for ConfigError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ProviderNotSpecified => {
                write!(f, "provider is not specified in .git/config")
            }
            ConfigError::UnknownProvider(value) => {
                write!(
                    f,
                    "unknown provider \"{value}\" is specified in .git/config",
                )
            }
            ConfigError::BaseUrlNotSpecified => {
                write!(f, "host is not specified in .git/config")
            }
            ConfigError::InvalidBaseUrl(value) => {
                write!(
                    f,
                    "invalid base_url \"{value}\" is specified in .git/config",
                )
            }
            ConfigError::UnableToUpdateConfig(value) => {
                write!(f, "unable to update config at .git/config: {value}")
            }
        }
    }
}

pub fn get_config(repo: &Repository) -> Result<Config, ConfigError> {
    let config = repo.config().unwrap();

    let provider = config
        .get_string("gitext.provider")
        .map_err(|_| ConfigError::ProviderNotSpecified)?;

    let provider = Provider::from_str(&provider)?;

    let base_url = config.get_string("gitext.baseurl").or_else(|_| {
        if let Provider::GitHub = provider {
            Ok("https://github.com".to_string())
        } else {
            Err(ConfigError::BaseUrlNotSpecified)
        }
    })?;

    let base_url = Url::parse(&base_url).map_err(|_| ConfigError::InvalidBaseUrl(base_url))?;

    if base_url.host().is_none() {
        return Err(ConfigError::InvalidBaseUrl(base_url.into()));
    }

    let auth_domain = config
        .get_string("gitext.authdomain")
        .unwrap_or(String::from(base_url.host_str().unwrap()));

    let jira_url = config.get_string("gitext.jiraurl").ok();
    let jira_url = jira_url.and_then(|string| Url::parse(&string).ok());

    Ok(Config {
        provider,
        base_url,
        auth_domain,
        jira_url,
    })
}

pub fn set_config(repo: &Repository, config: &Config) -> Result<(), ConfigError> {
    let mut repo_config = repo.config().unwrap();

    repo_config
        .set_str("gitext.provider", &config.provider.to_string())
        .map_err(|err| ConfigError::UnableToUpdateConfig(err.message().to_string()))?;

    repo_config
        .set_str("gitext.baseurl", config.base_url.as_str())
        .map_err(|err| ConfigError::UnableToUpdateConfig(err.message().to_string()))?;

    repo_config
        .set_str("gitext.authdomain", &config.auth_domain)
        .map_err(|err| ConfigError::UnableToUpdateConfig(err.message().to_string()))?;

    if let Some(jira_url) = &config.jira_url {
        repo_config
            .set_str("gitext.jiraurl", jira_url.as_str())
            .map_err(|err| ConfigError::UnableToUpdateConfig(err.message().to_string()))?;
    } else {
        repo_config.remove("gitext.jiraurl").ok();
    }

    Ok(())
}

pub fn set_provider(repo: &Repository, provider: Provider) -> Result<(), ConfigError> {
    let mut config = repo.config().unwrap();
    let key = "gitext.provider";
    let value = match provider {
        Provider::BitBucket => "bitbucket",
        Provider::GitLab => "gitlab",
        Provider::GitHub => "github",
    };

    config
        .set_str(key, value)
        .map_err(|err| ConfigError::UnableToUpdateConfig(err.message().to_string()))
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
