#[derive(Debug, Clone)]
pub enum Error {
    Serde(String),
    Io(String),
    Disconnected,
}

impl From<ws::Error> for Error {
    fn from(e: ws::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e.to_string())
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(e: std::sync::mpsc::RecvError) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<std::sync::mpsc::TryRecvError> for Error {
    fn from(_: std::sync::mpsc::TryRecvError) -> Self {
        Error::Disconnected
    }
}

pub type Result<T> = std::result::Result<T, Error>;
