mod commands {
    pub mod auth;
    pub mod browse;
}

pub use commands::auth::Auth;
pub use commands::browse::Browse;
