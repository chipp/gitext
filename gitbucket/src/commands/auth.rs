use bitbucket::Client;

use crate::Error;

pub struct Auth;

impl Auth {
    pub async fn handle(_args: std::env::Args) -> Result<(), Error> {
        match Self::print_whoami().await {
            Ok(_) => Ok(()),
            Err(_) => {
                auth::reset_token();
                Self::print_whoami()
                    .await
                    .map_err(|_| Error::AuthorizationError)
            }
        }
    }

    async fn print_whoami() -> Result<(), http_client::Error> {
        let client = Client::new();
        match client.whoami().await {
            Ok(user) => {
                println!("You're logged in as {}", user.display_name);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
