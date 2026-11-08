use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("failed to download the app list: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid app list: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown app {0}")]
    UnknownApp(String),
    #[error("{0} has no executable that can be simulated on this platform")]
    NoExecutable(String),
}
