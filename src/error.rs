/// Store error type.
#[derive(Debug)]
pub enum StoreError {
    OpenFailed(String),
    QueryFailed(String),
    NotFound(String),
    InvalidData(String),
    IoError(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenFailed(msg) => write!(f, "open failed: {msg}"),
            Self::QueryFailed(msg) => write!(f, "query failed: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::InvalidData(msg) => write!(f, "invalid data: {msg}"),
            Self::IoError(msg) => write!(f, "io error: {msg}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        Self::QueryFailed(e.to_string())
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::InvalidData(e.to_string())
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e.to_string())
    }
}
