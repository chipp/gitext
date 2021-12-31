use crate::AuthDomainConfig;
use git2::{Cred, CredentialType, Error};
use rpassword::prompt_password_stdout;

pub struct CredentialHelper {
    state: CredentialHelperState,
}

enum CredentialHelperState {
    Initialized,
    Default,
    SshAgent,
    SshKey,
    UserPass,
}

use CredentialHelperState::*;

impl CredentialHelper {
    pub fn new() -> CredentialHelper {
        CredentialHelper {
            state: CredentialHelperState::Initialized,
        }
    }

    pub fn credentials(
        &mut self,
        url: &str,
        username_from_url: Option<&str>,
        allowed_types: CredentialType,
        config: &dyn AuthDomainConfig,
    ) -> Result<Cred, Error> {
        match self.state {
            Initialized => println!("requested authorization for url {}", url),
            _ => (),
        }

        if allowed_types.is_ssh_key() {
            let username = username_from_url.unwrap_or("git");

            match self.state {
                SshAgent => {
                    self.state = SshKey;
                    Self::load_id_rsa(username, false)
                }
                SshKey => Self::load_id_rsa(username, true),
                _ => {
                    self.state = SshAgent;
                    Cred::ssh_key_from_agent(username)
                }
            }
        } else if allowed_types.is_user_pass_plaintext() {
            let (username, password) = auth::user_and_password(&config.auth_domain());
            self.state = UserPass;

            Cred::userpass_plaintext(&username, &password)
        } else {
            self.state = Default;
            Cred::default()
        }
    }

    fn load_id_rsa(username: &str, request_passphrase: bool) -> Result<Cred, Error> {
        let mut id_rsa = dirs::home_dir().unwrap();
        id_rsa.push(".ssh");
        id_rsa.push("id_rsa");

        let mut id_rsa_pub = std::path::PathBuf::from(&id_rsa);
        id_rsa_pub.set_extension("pub");

        let passphrase = if request_passphrase {
            println!();
            println!("you can add your ssh key to ssh-agent to authorize with it automatically `ssh-add ~/.ssh/id_rsa`");

            prompt_password_stdout("Enter passphrase for ssh key id_rsa:").ok()
        } else {
            None
        };

        Cred::ssh_key(username, Some(&id_rsa_pub), &id_rsa, passphrase.as_deref())
    }
}
