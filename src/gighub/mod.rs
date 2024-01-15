mod commands {
    pub mod auth;
    pub mod browse;
    pub mod prs;
}

pub use commands::auth::Auth;
pub use commands::browse::Browse;
pub use commands::prs::Prs;
