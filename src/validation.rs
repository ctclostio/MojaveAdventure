//! Input Validation Module
//!
//! Centralizes all input validation logic to ensure data integrity and security.

use crate::error::{CharacterError, GameError};

/// Validate character name
///
/// # Rules
/// - Must be between 1 and 50 characters
/// - Can only contain alphanumeric characters, spaces, hyphens, and apostrophes
/// - Cannot start or end with whitespace
///
/// # Examples
/// ```
/// use fallout_dnd::validation;
///
/// assert!(validation::validate_character_name("John Smith").is_ok());
/// assert!(validation::validate_character_name("O'Brien").is_ok());
/// assert!(validation::validate_character_name("").is_err());
/// ```
pub fn validate_character_name(name: &str) -> Result<(), GameError> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(
            CharacterError::InvalidName("Character name cannot be empty".to_string()).into(),
        );
    }

    if trimmed.len() > 50 {
        return Err(CharacterError::InvalidName(
            "Character name must be 50 characters or less".to_string(),
        )
        .into());
    }

    // Check for valid characters (alphanumeric, space, hyphen, apostrophe)
    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '\'')
    {
        return Err(CharacterError::InvalidName(
            "Character name can only contain letters, numbers, spaces, hyphens, and apostrophes"
                .to_string(),
        )
        .into());
    }

    // Check for leading/trailing whitespace (different from trimmed)
    if name != trimmed {
        return Err(CharacterError::InvalidName(
            "Character name cannot start or end with whitespace".to_string(),
        )
        .into());
    }

    tracing::debug!("Character name validation passed: {}", name);
    Ok(())
}

/// Validate save file name
///
/// # Rules
/// - Must be between 1 and 50 characters
/// - Can only contain alphanumeric characters, hyphens, and underscores
/// - Cannot contain path separators (., /, \)
/// - Cannot be "." or ".."
///
/// # Examples
/// ```
/// use fallout_dnd::validation;
///
/// assert!(validation::validate_save_name("my_save").is_ok());
/// assert!(validation::validate_save_name("save-01").is_ok());
/// assert!(validation::validate_save_name("../etc/passwd").is_err());
/// ```
pub fn validate_save_name(name: &str) -> Result<(), GameError> {
    if name.is_empty() {
        return Err(GameError::InvalidInput(
            "Save name cannot be empty".to_string(),
        ));
    }

    if name.len() > 50 {
        return Err(GameError::InvalidInput(
            "Save name must be 50 characters or less".to_string(),
        ));
    }

    // Check for path traversal attempts
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(GameError::PathTraversalError(format!(
            "Invalid save name: {}",
            name
        )));
    }

    // Prevent "." and ".." as save names
    if name == "." || name == ".." {
        return Err(GameError::PathTraversalError(
            "Save name cannot be '.' or '..'".to_string(),
        ));
    }

    // Only allow alphanumeric, hyphen, and underscore
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(GameError::InvalidInput(
            "Save name can only contain letters, numbers, hyphens, and underscores".to_string(),
        ));
    }

    tracing::debug!("Save name validation passed: {}", name);
    Ok(())
}

/// Validate SPECIAL stat value
///
/// # Rules
/// - Must be between 1 and 10 (inclusive)
///
/// # Examples
/// ```
/// use fallout_dnd::validation;
///
/// assert!(validation::validate_special_stat("strength", 5).is_ok());
/// assert!(validation::validate_special_stat("intelligence", 0).is_err());
/// assert!(validation::validate_special_stat("luck", 11).is_err());
/// ```
pub fn validate_special_stat(stat_name: &str, value: u8) -> Result<(), GameError> {
    if !(1..=10).contains(&value) {
        return Err(CharacterError::InvalidSpecialAllocation(format!(
            "Invalid {} value: {}. Must be between 1 and 10",
            stat_name, value
        ))
        .into());
    }

    Ok(())
}

/// Validate total SPECIAL points allocation
///
/// # Rules
/// - Total points must equal the target (typically 28 for new characters)
/// - Each stat must be between 1 and 10
///
pub fn validate_special_total(points: &[u8], target: u8) -> Result<(), GameError> {
    let total: u32 = points.iter().map(|&p| p as u32).sum();

    if total != target as u32 {
        return Err(CharacterError::InvalidSpecialAllocation(format!(
            "Total SPECIAL points must be {}. Current total: {}",
            target, total
        ))
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_character_name() {
        // Valid names
        assert!(validate_character_name("John Smith").is_ok());
        assert!(validate_character_name("O'Brien").is_ok());
        assert!(validate_character_name("Jean-Luc").is_ok());
        assert!(validate_character_name("A").is_ok());

        // Invalid names
        assert!(validate_character_name("").is_err());
        assert!(validate_character_name("  ").is_err());
        assert!(validate_character_name(" Leading space").is_err());
        assert!(validate_character_name("Trailing space ").is_err());
        assert!(validate_character_name(&"a".repeat(51)).is_err());
        assert!(validate_character_name("John@Smith").is_err());
    }

    #[test]
    fn test_validate_save_name() {
        // Valid save names
        assert!(validate_save_name("my_save").is_ok());
        assert!(validate_save_name("save-01").is_ok());
        assert!(validate_save_name("QuickSave").is_ok());

        // Invalid save names
        assert!(validate_save_name("").is_err());
        assert!(validate_save_name("../etc/passwd").is_err());
        assert!(validate_save_name("..").is_err());
        assert!(validate_save_name(".").is_err());
        assert!(validate_save_name("save/file").is_err());
        assert!(validate_save_name("save\\file").is_err());
        assert!(validate_save_name("save file").is_err()); // No spaces
        assert!(validate_save_name(&"a".repeat(51)).is_err());
    }

    #[test]
    fn test_validate_special_stat() {
        // Valid values
        for val in 1..=10 {
            assert!(validate_special_stat("strength", val).is_ok());
        }

        // Invalid values
        assert!(validate_special_stat("strength", 0).is_err());
        assert!(validate_special_stat("strength", 11).is_err());
        assert!(validate_special_stat("strength", 255).is_err());
    }

    #[test]
    fn test_validate_special_total() {
        // Valid total (7 stats with average of 4 = 28)
        let valid = vec![4, 4, 4, 4, 4, 4, 4];
        assert!(validate_special_total(&valid, 28).is_ok());

        // Different valid total
        let valid2 = vec![5, 5, 5, 3, 3, 3, 4];
        assert!(validate_special_total(&valid2, 28).is_ok());

        // Invalid total
        let invalid = vec![10, 10, 10, 10, 10, 10, 10]; // Too many points
        assert!(validate_special_total(&invalid, 28).is_err());

        let invalid2 = vec![1, 1, 1, 1, 1, 1, 1]; // Too few points
        assert!(validate_special_total(&invalid2, 28).is_err());
    }
}
