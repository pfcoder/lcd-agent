use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    #[error("Command error: {0}")]
    CommandError(String),
    //Utf8Error
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    //IoError
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
