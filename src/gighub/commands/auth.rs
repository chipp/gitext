use crate::common_git::{AuthDomainConfig, BaseUrlConfig};
use crate::error::Error;
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
                auth::reset_user_and_pass(config.auth_domain());
                Self::print_whoami(config)
                    .await
                    .map_err(|_| Error::AuthorizationError)
            }
        }
    }

    async fn print_whoami<Conf>(config: &Conf) -> Result<(), http_client::Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let client = Client::new(config);

        match client.whoami().await {
            Ok(user) => {
                println!("You're logged in as {}", user.name.unwrap_or(user.login));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
