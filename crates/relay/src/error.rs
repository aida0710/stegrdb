use thiserror::Error;

#[derive(Debug, Error)]
#[error("{message}")]
pub struct RelayError {
    message: String,
    retryable: bool,
}

impl RelayError {
    pub fn retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: true,
        }
    }

    pub fn permanent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: false,
        }
    }

    pub fn is_retryable(&self) -> bool {
        self.retryable
    }
}
