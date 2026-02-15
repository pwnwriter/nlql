// error types with pretty diagnostics

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error("couldn't connect to database")]
    #[diagnostic(
        code(nlql::db::connection),
        help("check your connection string and make sure the database is running")
    )]
    Database(#[from] sqlx::Error),

    #[error("claude api failed")]
    #[diagnostic(
        code(nlql::ai::claude),
        help("check your api key and network connection")
    )]
    Claude(String),

    #[error("no api key found")]
    #[diagnostic(
        code(nlql::ai::no_key),
        help("set ANTHROPIC_API_KEY or CLAUDE_API_KEY environment variable")
    )]
    MissingApiKey,

    #[error("http request failed")]
    #[diagnostic(code(nlql::http))]
    Http(#[from] reqwest::Error),

    #[error("json error")]
    #[diagnostic(code(nlql::json))]
    Json(#[from] serde_json::Error),

    #[error("server error: {0}")]
    #[diagnostic(code(nlql::server))]
    Server(String),
}
