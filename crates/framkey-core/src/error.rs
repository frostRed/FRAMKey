use thiserror::Error;

pub type Result<T> = std::result::Result<T, FramkeyError>;

#[derive(Debug, Error)]
pub enum FramkeyError {
    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl FramkeyError {
    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::InvalidData(message.into())
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }
}
