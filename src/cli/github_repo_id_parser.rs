use std::ffi::OsStr;

use clap::{
    builder::{TypedValueParser, ValueParserFactory},
    error::{ContextKind, ContextValue, ErrorKind},
    Arg, Command, Error,
};
use url::Url;

use crate::github::RepoId;

impl ValueParserFactory for RepoId {
    type Parser = GithubRepoIdParser;
    fn value_parser() -> Self::Parser {
        GithubRepoIdParser
    }
}

#[derive(Clone, Debug)]
pub struct GithubRepoIdParser;
impl TypedValueParser for GithubRepoIdParser {
    type Value = RepoId;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let value = value
            .to_str()
            .ok_or(Error::new(ErrorKind::InvalidUtf8).with_cmd(cmd))?;

        let base_url = Url::parse("https://github.com").unwrap();
        if RepoId::from_str_with_host(value, &base_url).is_ok() {
            return Err(Error::new(ErrorKind::ValueValidation));
        }

        let components = value.split("/").collect::<Vec<_>>();

        if components.len() == 2 {
            let owner = components[0].to_owned();
            let repo = components[1].to_owned();

            // Usernames for user accounts on GitHub.com can only contain alphanumeric characters and dashes ( - ).
            if !owner.chars().all(|c| c.is_alphanumeric() || c == '-') {
                return Err(Error::new(ErrorKind::ValueValidation));
            }

            Ok(RepoId { owner, repo })
        } else {
            let mut error = Error::new(ErrorKind::ValueValidation);

            error.insert(
                ContextKind::InvalidArg,
                ContextValue::String(arg.map(ToString::to_string).unwrap_or("...".to_string())),
            );

            error.insert(
                ContextKind::InvalidValue,
                ContextValue::String(value.to_owned()),
            );

            Err(error)
        }
    }
}
