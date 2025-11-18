use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Save file error: {0}")]
    SaveFileError(#[from] std::io::Error),

    #[error("AI connection failed: {0}")]
    AIConnectionError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Path traversal detected: {0}")]
    PathTraversalError(String),

    #[error("Combat error: {0}")]
    CombatError(String),

    #[error("Character creation error: {0}")]
    CharacterError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("{0}")]
    Other(String),
}

// Convenience conversion from anyhow::Error
impl From<anyhow::Error> for GameError {
    fn from(err: anyhow::Error) -> Self {
        GameError::Other(err.to_string())
    }
}

// Allow String to be converted to GameError
impl From<String> for GameError {
    fn from(s: String) -> Self {
        GameError::Other(s)
    }
}

impl From<&str> for GameError {
    fn from(s: &str) -> Self {
        GameError::Other(s.to_string())
    }
}
