use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Claude API error: {0}")]
    Claude(String),

    #[error("Missing API key. Set one of: ANTHROPIC_API_KEY, CLAUDE_API_KEY, or CLAUDE_KEY")]
    MissingApiKey,

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Server error: {0}")]
    Server(String),
}
