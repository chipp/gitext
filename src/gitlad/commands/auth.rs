use crate::common_git::BaseUrlConfig;
use crate::{common_git::AuthDomainConfig, gitlab::Client};

use crate::Error;
use http_client::{Error as HttpError, ErrorKind as HttpErrorKind};

pub struct Auth;

impl Auth {
    pub async fn handle<Conf>(config: &Conf) -> Result<(), Error>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig + Send + Sync,
    {
        match Self::print_whoami(config).await {
            Ok(_) => Ok(()),
            Err(HttpError {
                request: _,
                kind: HttpErrorKind::JsonParseError(_),
            }) => Ok(()),
            Err(err) => {
                println!("{:#?}", err);

                auth::reset_token(config.auth_domain(), "access_token");

                Self::print_whoami(config)
                    .await
                    .map_err(|_| Error::AuthorizationError)
            }
        }
    }

    async fn print_whoami<Conf>(config: &Conf) -> Result<(), http_client::Error>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig + Send + Sync,
    {
        let client = Client::new(config);
        match client.whoami().await {
            Ok(user) => {
                println!("You're logged in as {}", user.display_name);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
