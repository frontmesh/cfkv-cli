use thiserror::Error;

/// Blog module errors
#[derive(Error, Debug)]
pub enum BlogError {
    #[error("Frontmatter error: {0}")]
    FrontmatterError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("KV error: {0}")]
    KvError(String),
}

pub type Result<T> = std::result::Result<T, BlogError>;
