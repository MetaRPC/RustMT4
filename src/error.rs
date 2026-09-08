use thiserror::Error;

#[derive(Error, Debug)]
pub enum MT4Error {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Authentication failed for user: {0}")]
    AuthError(u64),
    #[error("Trade failed with code {0}: {1}")]
    TradeError(u32, String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
