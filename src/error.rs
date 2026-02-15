// error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("claude api error: {0}")]
    Claude(String),

    #[error("missing api key - set ANTHROPIC_API_KEY or CLAUDE_API_KEY")]
    MissingApiKey,

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("server error: {0}")]
    Server(String),
}
