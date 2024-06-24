use chipp_http::{curl::easy, Interceptor, Request};

pub struct Authenticator<'a> {
    auth_domain: &'a str,
    mode: Mode,
}

pub enum Mode {
    BasicAuth,
    Token(&'static str),
}

impl<'a> Authenticator<'a> {
    pub fn basic_auth(auth_domain: &'a str) -> Self {
        Authenticator {
            auth_domain,
            mode: Mode::BasicAuth,
        }
    }

    pub fn token(auth_domain: &'a str, token_name: &'static str) -> Self {
        Authenticator {
            auth_domain,
            mode: Mode::Token(token_name),
        }
    }
}

impl Interceptor for Authenticator<'_> {
    fn modify(&self, easy: &mut easy::Easy, _: &Request) {
        if let Mode::BasicAuth = self.mode {
            let mut auth = easy::Auth::new();
            auth.basic(true);
            easy.http_auth(&auth).unwrap();

            let (username, password) = chipp_auth::user_and_password(self.auth_domain);

            easy.username(username.as_ref()).unwrap();
            easy.password(password.as_ref()).unwrap();
        }
    }

    fn add_headers(&self, headers: &mut easy::List, _: &Request) {
        if let Mode::Token(token_name) = self.mode {
            let header = format!(
                "Authorization: Bearer {}",
                chipp_auth::token(self.auth_domain, token_name)
            );

            headers.append(&header).unwrap();
        }
    }
}
