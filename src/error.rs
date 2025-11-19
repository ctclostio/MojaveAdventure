use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum GameError {
    #[error("Save file error: {0}")]
    #[diagnostic(
        code(fallout_dnd::save_file_error),
        help("Check that the save directory exists and has write permissions")
    )]
    SaveFileError(#[from] std::io::Error),

    #[error("AI connection failed: {0}")]
    #[diagnostic(
        code(fallout_dnd::ai_connection),
        help("Make sure llama.cpp server is running at the configured URL.\nYou can start it with: llama-server -m <model-path> -c 4096")
    )]
    AIConnectionError(String),

    #[error("Invalid input: {0}")]
    #[diagnostic(
        code(fallout_dnd::invalid_input),
        help("Check the command syntax and try again")
    )]
    InvalidInput(String),

    #[error("Serialization error: {0}")]
    #[diagnostic(
        code(fallout_dnd::serialization),
        help("The save file may be corrupted. Consider loading an earlier save.")
    )]
    SerializationError(#[from] serde_json::Error),

    #[error("Deserialization error: {0}")]
    #[diagnostic(
        code(fallout_dnd::deserialization),
        help("The configuration file may be invalid. Check config.toml for syntax errors.")
    )]
    DeserializationError(#[from] toml::de::Error),

    #[error("Path traversal detected: {0}")]
    #[diagnostic(
        code(fallout_dnd::security::path_traversal),
        severity(Error),
        help("File paths must stay within the game directory for security")
    )]
    PathTraversalError(String),

    #[error("Combat error: {0}")]
    #[diagnostic(transparent)]
    Combat(#[from] CombatError),

    #[error("Character creation error: {0}")]
    #[diagnostic(transparent)]
    Character(#[from] CharacterError),

    #[error("Configuration error: {0}")]
    #[diagnostic(transparent)]
    Config(#[from] ConfigError),

    #[error("Network error: {0}")]
    #[diagnostic(
        code(fallout_dnd::network),
        help("Check your internet connection or AI server configuration")
    )]
    NetworkError(#[from] reqwest::Error),

    #[error("{0}")]
    #[diagnostic(code(fallout_dnd::other))]
    Other(String),
}

#[derive(Error, Debug, Clone, PartialEq, Eq, Diagnostic)]
pub enum CombatError {
    #[error("Not enough Action Points to perform that action.")]
    #[diagnostic(
        code(fallout_dnd::combat::insufficient_ap),
        help("Wait for next round to regenerate AP, or use an action that costs less AP")
    )]
    #[allow(dead_code)]
    InsufficientAP,

    #[error("Target with ID {0} not found in the current combat.")]
    #[diagnostic(
        code(fallout_dnd::combat::target_not_found),
        help("Check the list of active enemies with the 'enemies' command")
    )]
    #[allow(dead_code)]
    TargetNotFound(String),

    #[error("Cannot perform action: combat is not active.")]
    #[diagnostic(
        code(fallout_dnd::combat::not_active),
        help("You need to be in combat to perform this action")
    )]
    #[allow(dead_code)]
    CombatNotActive,
}

#[derive(Error, Debug, Clone, PartialEq, Eq, Diagnostic)]
pub enum CharacterError {
    #[error("Invalid SPECIAL stat allocation: {0}.")]
    #[diagnostic(
        code(fallout_dnd::character::invalid_special),
        help("SPECIAL stats must total 40 points maximum, with each stat between 1-10")
    )]
    #[allow(dead_code)]
    InvalidSpecialAllocation(String),

    #[error("Character name '{0}' is invalid.")]
    #[diagnostic(
        code(fallout_dnd::character::invalid_name),
        help("Character name must be 1-50 characters long and contain only letters, numbers, spaces, and basic punctuation")
    )]
    InvalidName(String),
}

#[derive(Error, Debug, Clone, PartialEq, Diagnostic)]
pub enum ConfigError {
    #[error("Invalid temperature: {0}. Must be between 0.0 and 2.0.")]
    #[diagnostic(
        code(fallout_dnd::config::invalid_temperature),
        help("Temperature controls randomness. Try values like 0.7 (focused) or 1.2 (creative)")
    )]
    InvalidTemperature(f32),

    #[error("Invalid top_p: {0}. Must be between 0.0 and 1.0.")]
    #[diagnostic(
        code(fallout_dnd::config::invalid_top_p),
        help("Top_p controls diversity. Typical values are 0.9-0.95")
    )]
    InvalidTopP(f32),

    #[error("Invalid top_k: {0}. Must be at least 1.")]
    #[diagnostic(
        code(fallout_dnd::config::invalid_top_k),
        help("Top_k limits token selection. Typical values are 40-100")
    )]
    InvalidTopK(i32),

    #[error("Invalid max_tokens: {0}. Must be between 1 and 32000.")]
    #[diagnostic(
        code(fallout_dnd::config::invalid_max_tokens),
        help("Max_tokens limits response length. For narrative, try 512-2048")
    )]
    InvalidMaxTokens(i32),

    #[error("Invalid context_window: {0}. Must be between 512 and 128000.")]
    #[diagnostic(
        code(fallout_dnd::config::invalid_context_window),
        help(
            "Context_window must match your model's capabilities. Common values: 4096, 8192, 32768"
        )
    )]
    InvalidContextWindow(i32),

    #[error("Invalid repeat_penalty: {0}. Must be between 1.0 and 2.0.")]
    #[diagnostic(
        code(fallout_dnd::config::invalid_repeat_penalty),
        help("Repeat_penalty reduces repetition. Try 1.1-1.3")
    )]
    InvalidRepeatPenalty(f32),

    #[error("Invalid starting_level: {0}. Must be between 1 and 50.")]
    #[diagnostic(
        code(fallout_dnd::config::invalid_starting_level),
        help("Starting level determines initial character power. Level 1 is recommended for new characters")
    )]
    InvalidStartingLevel(u32),

    #[error("Invalid starting_caps: {0}. Must be less than 1,000,000.")]
    #[diagnostic(
        code(fallout_dnd::config::invalid_starting_caps),
        help("Starting caps is the initial currency. Typical values: 100-1000")
    )]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_game_error_display_messages() {
        // Test that error messages format correctly
        let error = GameError::AIConnectionError("Connection refused".to_string());
        assert_eq!(
            error.to_string(),
            "AI connection failed: Connection refused"
        );

        let error = GameError::InvalidInput("Bad command".to_string());
        assert_eq!(error.to_string(), "Invalid input: Bad command");

        let error = GameError::PathTraversalError("Path escapes directory".to_string());
        assert_eq!(
            error.to_string(),
            "Path traversal detected: Path escapes directory"
        );

        let error = GameError::Other("Custom error".to_string());
        assert_eq!(error.to_string(), "Custom error");
    }

    #[test]
    fn test_combat_error_display_messages() {
        let error = CombatError::InsufficientAP;
        assert_eq!(
            error.to_string(),
            "Not enough Action Points to perform that action."
        );

        let error = CombatError::TargetNotFound("enemy_123".to_string());
        assert_eq!(
            error.to_string(),
            "Target with ID enemy_123 not found in the current combat."
        );

        let error = CombatError::CombatNotActive;
        assert_eq!(
            error.to_string(),
            "Cannot perform action: combat is not active."
        );
    }

    #[test]
    fn test_character_error_display_messages() {
        let error = CharacterError::InvalidSpecialAllocation("Total exceeds 40".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid SPECIAL stat allocation: Total exceeds 40."
        );

        let error = CharacterError::InvalidName("".to_string());
        assert_eq!(error.to_string(), "Character name '' is invalid.");
    }

    #[test]
    fn test_config_error_display_messages() {
        let error = ConfigError::InvalidTemperature(2.5);
        assert_eq!(
            error.to_string(),
            "Invalid temperature: 2.5. Must be between 0.0 and 2.0."
        );

        let error = ConfigError::InvalidTopP(1.5);
        assert_eq!(
            error.to_string(),
            "Invalid top_p: 1.5. Must be between 0.0 and 1.0."
        );

        let error = ConfigError::InvalidTopK(0);
        assert_eq!(error.to_string(), "Invalid top_k: 0. Must be at least 1.");

        let error = ConfigError::InvalidMaxTokens(50000);
        assert_eq!(
            error.to_string(),
            "Invalid max_tokens: 50000. Must be between 1 and 32000."
        );

        let error = ConfigError::InvalidRepeatPenalty(3.0);
        assert_eq!(
            error.to_string(),
            "Invalid repeat_penalty: 3. Must be between 1.0 and 2.0."
        );

        let error = ConfigError::InvalidStartingLevel(100);
        assert_eq!(
            error.to_string(),
            "Invalid starting_level: 100. Must be between 1 and 50."
        );

        let error = ConfigError::InvalidStartingCaps(2000000);
        assert_eq!(
            error.to_string(),
            "Invalid starting_caps: 2000000. Must be less than 1,000,000."
        );
    }

    #[test]
    fn test_game_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let game_error: GameError = io_error.into();

        match game_error {
            GameError::SaveFileError(_) => {} // Expected
            _ => panic!("Expected SaveFileError variant"),
        }
    }

    #[test]
    fn test_game_error_from_serde_json_error() {
        let invalid_json = "{invalid json";
        let json_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let game_error: GameError = json_error.into();

        match game_error {
            GameError::SerializationError(_) => {} // Expected
            _ => panic!("Expected SerializationError variant"),
        }
    }

    #[test]
    fn test_game_error_from_combat_error() {
        let combat_error = CombatError::InsufficientAP;
        let game_error: GameError = combat_error.into();

        match game_error {
            GameError::Combat(CombatError::InsufficientAP) => {} // Expected
            _ => panic!("Expected Combat(InsufficientAP) variant"),
        }
    }

    #[test]
    fn test_game_error_from_character_error() {
        let char_error = CharacterError::InvalidName("".to_string());
        let game_error: GameError = char_error.into();

        match game_error {
            GameError::Character(CharacterError::InvalidName(_)) => {} // Expected
            _ => panic!("Expected Character(InvalidName) variant"),
        }
    }

    #[test]
    fn test_game_error_from_config_error() {
        let config_error = ConfigError::InvalidTemperature(2.5);
        let game_error: GameError = config_error.into();

        match game_error {
            GameError::Config(ConfigError::InvalidTemperature(_)) => {} // Expected
            _ => panic!("Expected Config(InvalidTemperature) variant"),
        }
    }

    #[test]
    fn test_game_error_from_string() {
        let error_msg = "Something went wrong".to_string();
        let game_error: GameError = error_msg.clone().into();

        match game_error {
            GameError::Other(msg) => assert_eq!(msg, error_msg),
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_game_error_from_str() {
        let error_msg = "Error message";
        let game_error: GameError = error_msg.into();

        match game_error {
            GameError::Other(msg) => assert_eq!(msg, error_msg),
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_game_error_from_anyhow() {
        let anyhow_error = anyhow::anyhow!("Anyhow error");
        let game_error: GameError = anyhow_error.into();

        match game_error {
            GameError::Other(msg) => assert!(msg.contains("Anyhow error")),
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_combat_error_clone() {
        let error1 = CombatError::InsufficientAP;
        let error2 = error1.clone();
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_combat_error_equality() {
        let error1 = CombatError::TargetNotFound("enemy_1".to_string());
        let error2 = CombatError::TargetNotFound("enemy_1".to_string());
        let error3 = CombatError::TargetNotFound("enemy_2".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
        assert_ne!(error1, CombatError::InsufficientAP);
    }

    #[test]
    fn test_character_error_clone() {
        let error1 = CharacterError::InvalidName("Test".to_string());
        let error2 = error1.clone();
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_character_error_equality() {
        let error1 = CharacterError::InvalidName("".to_string());
        let error2 = CharacterError::InvalidName("".to_string());
        let error3 = CharacterError::InvalidName("Bad".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_config_error_clone() {
        let error1 = ConfigError::InvalidTemperature(2.5);
        let error2 = error1.clone();
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_config_error_partial_eq() {
        let error1 = ConfigError::InvalidTopP(1.5);
        let error2 = ConfigError::InvalidTopP(1.5);
        let error3 = ConfigError::InvalidTopP(1.2);

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_all_combat_error_variants() {
        // Test all combat error variants can be created
        let errors = vec![
            CombatError::InsufficientAP,
            CombatError::TargetNotFound("test".to_string()),
            CombatError::CombatNotActive,
        ];

        for error in errors {
            // Verify each can be formatted
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn test_all_character_error_variants() {
        let errors = vec![
            CharacterError::InvalidSpecialAllocation("test".to_string()),
            CharacterError::InvalidName("test".to_string()),
        ];

        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn test_all_config_error_variants() {
        let errors = vec![
            ConfigError::InvalidTemperature(3.0),
            ConfigError::InvalidTopP(2.0),
            ConfigError::InvalidTopK(-1),
            ConfigError::InvalidMaxTokens(100000),
            ConfigError::InvalidRepeatPenalty(5.0),
            ConfigError::InvalidStartingLevel(200),
            ConfigError::InvalidStartingCaps(10000000),
        ];

        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn test_combat_error_debug_format() {
        let error = CombatError::InsufficientAP;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InsufficientAP"));
    }

    #[test]
    fn test_character_error_debug_format() {
        let error = CharacterError::InvalidName("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidName"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_config_error_debug_format() {
        let error = ConfigError::InvalidTemperature(2.5);
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidTemperature"));
    }

    #[test]
    fn test_game_error_chain_from_combat() {
        let combat_err = CombatError::CombatNotActive;
        let game_err: GameError = combat_err.into();

        assert!(game_err.to_string().contains("combat is not active"));
    }

    #[test]
    fn test_game_error_ai_connection() {
        let error = GameError::AIConnectionError("localhost:8080".to_string());
        let msg = error.to_string();

        assert!(msg.contains("AI connection failed"));
        assert!(msg.contains("localhost:8080"));
    }

    #[test]
    fn test_error_source_chain() {
        // Create a nested error with source
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let game_error: GameError = io_error.into();

        // Verify the error can be displayed
        assert!(game_error.to_string().contains("Save file error"));
    }
}
