use thiserror::Error;

#[derive(Error, Debug)]
pub enum RdtError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Reddit API error: {0}")]
    RedditApi(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OAuth error: {0}")]
    OAuth(String),

    #[error("AWS Bedrock error: {0}")]
    Bedrock(String),

    #[error("Pattern matching error: {0}")]
    Pattern(String),

    #[error("Not authenticated. Run 'rdt auth login' first.")]
    NotAuthenticated,

    #[error("Rate limited. Please wait before making more requests.")]
    RateLimited,

    #[error("TUI error: {0}")]
    Tui(String),
}

pub type Result<T> = std::result::Result<T, RdtError>;
