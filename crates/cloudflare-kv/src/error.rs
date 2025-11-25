use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, KvError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let test_cases = vec![
            (
                KvError::InvalidConfig("test config error".to_string()),
                "Invalid configuration: test config error",
            ),
            (
                KvError::AuthError("invalid token".to_string()),
                "Authentication failed: invalid token",
            ),
            (
                KvError::KeyNotFound("my-key".to_string()),
                "Key not found: my-key",
            ),
            (
                KvError::RequestFailed("500 server error".to_string()),
                "Request failed: 500 server error",
            ),
            (
                KvError::SerializationError("invalid json".to_string()),
                "Serialization error: invalid json",
            ),
        ];

        for (error, expected) in test_cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = KvError::InvalidConfig("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidConfig"));
    }
}
