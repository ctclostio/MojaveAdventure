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

    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] toml::de::Error),

    #[error("Path traversal detected: {0}")]
    PathTraversalError(String),

    #[error("Combat error: {0}")]
    Combat(#[from] CombatError),

    #[error("Character creation error: {0}")]
    Character(#[from] CharacterError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("{0}")]
    Other(String),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CombatError {
    #[error("Not enough Action Points to perform that action.")]
    InsufficientAP,
    #[error("Target with ID {0} not found in the current combat.")]
    TargetNotFound(String),
    #[error("Cannot perform action: combat is not active.")]
    CombatNotActive,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CharacterError {
    #[error("Invalid SPECIAL stat allocation: {0}.")]
    InvalidSpecialAllocation(String),
    #[error("Character name '{0}' is invalid.")]
    InvalidName(String),
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConfigError {
    #[error("Invalid temperature: {0}. Must be between 0.0 and 2.0.")]
    InvalidTemperature(f32),
    #[error("Invalid top_p: {0}. Must be between 0.0 and 1.0.")]
    InvalidTopP(f32),
    #[error("Invalid top_k: {0}. Must be at least 1.")]
    InvalidTopK(i32),
    #[error("Invalid max_tokens: {0}. Must be between 1 and 32000.")]
    InvalidMaxTokens(i32),
    #[error("Invalid repeat_penalty: {0}. Must be between 1.0 and 2.0.")]
    InvalidRepeatPenalty(f32),
    #[error("Invalid starting_level: {0}. Must be between 1 and 50.")]
    InvalidStartingLevel(u32),
    #[error("Invalid starting_caps: {0}. Must be less than 1,000,000.")]
    InvalidStartingCaps(u32),
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
