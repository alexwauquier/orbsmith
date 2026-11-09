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
    #[error("{app} has no executable named {executable}")]
    UnknownExecutable { app: String, executable: String },
    #[error("{0} is already running")]
    AlreadyRunning(String),
    #[error(
        "{app} cannot start as {executable}: {other} is running as that executable. Stop it first, or pick another executable."
    )]
    ExecutableInUse {
        app: String,
        executable: String,
        other: String,
    },
    #[error("no simulation is running for {0}")]
    NotRunning(String),
    #[error("dummy executable not found at {0}")]
    MissingDummy(String),
    #[error("invalid settings file: {0}")]
    InvalidSettings(#[source] serde_json::Error),
    #[error("invalid queue file: {0}")]
    InvalidQueueFile(#[source] serde_json::Error),
    #[error("{0} is already in the queue")]
    AlreadyQueued(String),
    #[error("the auto-stop timer cannot exceed 7 days")]
    TimerTooLong,
    #[error("refusing to create {0}: it resolves outside the executables directory")]
    OutsideExecutablesDir(String),
    #[error("could not launch {path}: {source}")]
    Spawn {
        path: String,
        #[source]
        source: io::Error,
    },
}
