use crate::error::Error;
use crate::git::{AuthDomainConfig, BaseUrlConfig};
use crate::github::Client;

pub struct Auth;

impl Auth {
    pub async fn handle<Conf>(config: &Conf) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        match Self::print_whoami(config).await {
            Ok(_) => Ok(()),
            Err(_) => {
                chipp_auth::reset_user_and_pass(config.auth_domain());
                Self::print_whoami(config)
                    .await
                    .map_err(|_| Error::AuthorizationError)
            }
        }
    }

    async fn print_whoami<Conf>(config: &Conf) -> Result<(), chipp_http::Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let client = Client::new(config);

        match client.whoami().await {
            Ok(user) => {
                println!("You're logged in as {}", user.login);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
