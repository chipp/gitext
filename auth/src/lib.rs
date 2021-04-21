use security_framework::os::macos::keychain::SecKeychain;
use security_framework::os::macos::passwords::*;

pub fn credentials() -> (String, String) {
    let username = std::env::var("GB_USERNAME")
        .expect("set your BitBucket username in env variable GB_USERNAME (e.g. in ~/.zshrc)");
    let (password, _) = SecKeychain::default()
        .unwrap()
        .find_internet_password(
            "bitbucket.company.com",
            None,
            &username,
            "",
            None,
            SecProtocolType::Any,
            SecAuthenticationType::Any,
        )
        .unwrap();

    (
        username,
        String::from_utf8(Vec::from(password.as_ref())).unwrap(),
    )
}
