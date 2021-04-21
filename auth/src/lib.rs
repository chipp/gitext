use rpassword::prompt_password_stdout;
use security_framework::os::macos::keychain::SecKeychain;
use security_framework::os::macos::passwords::*;

mod config;
use config::*;

const SERVER: &str = "bitbucket.company.com";

pub fn credentials() -> (String, String) {
    let username = match load_config() {
        Some(config) => config.username,
        None => {
            let username = match std::env::var("GB_USERNAME").ok() {
                Some(username) => username,
                None => request_username(),
            };

            let config = Config {
                username: username.clone(),
            };
            save_config(&config).unwrap();

            username
        }
    };

    let keychain = SecKeychain::default().unwrap();

    let token = match keychain.find_internet_password(
        SERVER,
        None,
        &username,
        "",
        None,
        SecProtocolType::Any,
        SecAuthenticationType::Any,
    ) {
        Ok((token, _)) => String::from_utf8(Vec::from(token.as_ref())).unwrap(),
        Err(_) => {
            let token = request_password();

            keychain
                .add_internet_password(
                    SERVER,
                    None,
                    &username,
                    "/",
                    None,
                    SecProtocolType::HTTPS,
                    SecAuthenticationType::Any,
                    &token.as_bytes(),
                )
                .unwrap();

            token
        }
    };

    (username, token)
}

pub fn reset_token() {
    let username = std::env::var("GB_USERNAME").expect(
        "set your BitBucket username in env variable GB_USERNAME (e.g. in ~/.zshrc or ~/.bashrc)",
    );

    let keychain = SecKeychain::default().unwrap();

    if let Ok((_, item)) = keychain.find_internet_password(
        SERVER,
        None,
        &username,
        "",
        None,
        SecProtocolType::Any,
        SecAuthenticationType::Any,
    ) {
        item.delete()
    }
}

fn request_username() -> String {
    use std::io;
    use std::io::prelude::*;
    print!("Username: ");
    io::stdout().flush().unwrap();
    let mut username = String::default();
    io::stdin().read_line(&mut username).unwrap();
    username.trim().to_owned()
}

fn request_password() -> String {
    println!("=============================================================");
    println!("Please generate a personal access token with Read permission.");
    println!("https://bitbucket.company.com/plugins/servlet/access-tokens/manage");
    println!("It will be stored securely in Keychain.");

    prompt_password_stdout("Enter the token: ").unwrap()
}
