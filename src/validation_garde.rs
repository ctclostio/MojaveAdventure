//! Declarative Validation Module using garde
//!
//! This module provides type-safe, declarative validation using the garde framework,
//! replacing manual validation logic with cleaner, more maintainable validation rules.

use crate::error::{CharacterError, GameError};
use garde::Validate;

/// Character name with built-in validation
#[derive(Debug, Clone, Validate)]
pub struct CharacterName {
    #[garde(length(min = 1, max = 50), custom(validate_character_name_chars))]
    pub value: String,
}

impl CharacterName {
    /// Create a new validated character name
    pub fn new(name: impl Into<String>) -> Result<Self, GameError> {
        let mut value: String = name.into();

        // Trim leading/trailing whitespace first
        let trimmed = value.trim();
        if trimmed != value {
            return Err::<Self, GameError>(
                CharacterError::InvalidName(
                    "Character name cannot start or end with whitespace".to_string(),
                )
                .into(),
            );
        }

        value = trimmed.to_string();
        let char_name = Self { value };
        char_name.validate().map_err(|e| {
            GameError::from(CharacterError::InvalidName(format!(
                "Validation failed: {}",
                e
            )))
        })?;
        Ok(char_name)
    }

    /// Get the validated name as a string
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Convert into owned String
    pub fn into_string(self) -> String {
        self.value
    }
}

/// Custom validator for character name characters
fn validate_character_name_chars(value: &str, _context: &()) -> garde::Result {
    if value
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '\'')
    {
        Ok(())
    } else {
        Err(garde::Error::new(
            "Character name can only contain letters, numbers, spaces, hyphens, and apostrophes",
        ))
    }
}

/// Save file name with built-in validation
#[derive(Debug, Clone, Validate)]
pub struct SaveName {
    #[garde(
        length(min = 1, max = 50),
        custom(validate_save_name_chars),
        custom(validate_no_path_traversal)
    )]
    pub value: String,
}

impl SaveName {
    /// Create a new validated save name
    pub fn new(name: impl Into<String>) -> Result<Self, GameError> {
        let value: String = name.into();
        let save_name = Self { value };
        save_name
            .validate()
            .map_err(|e| GameError::InvalidInput(format!("Validation failed: {}", e)))?;
        Ok(save_name)
    }

    /// Get the validated name as a string
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Convert into owned String
    pub fn into_string(self) -> String {
        self.value
    }
}

/// Custom validator for save name characters (alphanumeric, hyphen, underscore)
fn validate_save_name_chars(value: &str, _context: &()) -> garde::Result {
    if value
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        Ok(())
    } else {
        Err(garde::Error::new(
            "Save name can only contain letters, numbers, hyphens, and underscores",
        ))
    }
}

/// Custom validator to prevent path traversal
fn validate_no_path_traversal(value: &str, _context: &()) -> garde::Result {
    if value.contains("..") || value.contains('/') || value.contains('\\') {
        return Err(garde::Error::new("Path traversal attempt detected"));
    }

    if value == "." || value == ".." {
        return Err(garde::Error::new("Save name cannot be '.' or '..'"));
    }

    Ok(())
}

/// SPECIAL stat value with validation
#[derive(Debug, Clone, Copy, Validate)]
pub struct SpecialStat {
    #[garde(range(min = 1, max = 10))]
    pub value: u8,
}

impl SpecialStat {
    /// Create a new validated SPECIAL stat
    pub fn new(value: u8) -> Result<Self, GameError> {
        let stat = Self { value };
        stat.validate().map_err(|e| -> GameError {
            GameError::from(CharacterError::InvalidSpecialAllocation(format!(
                "SPECIAL stat must be 1-10: {}",
                e
            )))
        })?;
        Ok(stat)
    }

    /// Get the stat value
    pub fn get(&self) -> u8 {
        self.value
    }
}

/// Complete SPECIAL allocation with validation
#[derive(Debug, Clone, Validate)]
pub struct SpecialAllocation {
    #[garde(range(min = 1, max = 10))]
    pub strength: u8,
    #[garde(range(min = 1, max = 10))]
    pub perception: u8,
    #[garde(range(min = 1, max = 10))]
    pub endurance: u8,
    #[garde(range(min = 1, max = 10))]
    pub charisma: u8,
    #[garde(range(min = 1, max = 10))]
    pub intelligence: u8,
    #[garde(range(min = 1, max = 10))]
    pub agility: u8,
    #[garde(range(min = 1, max = 10))]
    pub luck: u8,
    #[garde(skip)]
    _phantom: (),
}

impl SpecialAllocation {
    /// Create a new validated SPECIAL allocation
    pub fn new(
        strength: u8,
        perception: u8,
        endurance: u8,
        charisma: u8,
        intelligence: u8,
        agility: u8,
        luck: u8,
        target_total: u8,
    ) -> Result<Self, GameError> {
        let allocation = Self {
            strength,
            perception,
            endurance,
            charisma,
            intelligence,
            agility,
            luck,
            _phantom: (),
        };

        // Validate individual stats
        allocation.validate().map_err(|e| -> GameError {
            GameError::from(CharacterError::InvalidSpecialAllocation(format!(
                "Invalid SPECIAL stats: {}",
                e
            )))
        })?;

        // Validate total
        let total = strength + perception + endurance + charisma + intelligence + agility + luck;
        if total != target_total {
            return Err::<Self, GameError>(GameError::from(
                CharacterError::InvalidSpecialAllocation(format!(
                    "SPECIAL stats must total {}, got {}",
                    target_total, total
                )),
            ));
        }

        Ok(allocation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_name_valid() {
        assert!(CharacterName::new("John Smith").is_ok());
        assert!(CharacterName::new("O'Brien").is_ok());
        assert!(CharacterName::new("Jean-Luc").is_ok());
        assert!(CharacterName::new("A").is_ok());
    }

    #[test]
    fn test_character_name_invalid() {
        assert!(CharacterName::new("").is_err());
        assert!(CharacterName::new(" ").is_err());
        assert!(CharacterName::new("  Leading space").is_err());
        assert!(CharacterName::new("Trailing space  ").is_err());
        assert!(CharacterName::new("x".repeat(51)).is_err());
        assert!(CharacterName::new("Invalid@Name").is_err());
    }

    #[test]
    fn test_save_name_valid() {
        assert!(SaveName::new("my_save").is_ok());
        assert!(SaveName::new("save-01").is_ok());
        assert!(SaveName::new("quick_save_2024").is_ok());
    }

    #[test]
    fn test_save_name_invalid() {
        assert!(SaveName::new("").is_err());
        assert!(SaveName::new("../etc/passwd").is_err());
        assert!(SaveName::new("..").is_err());
        assert!(SaveName::new(".").is_err());
        assert!(SaveName::new("path/to/save").is_err());
        assert!(SaveName::new("windows\\path").is_err());
        assert!(SaveName::new("x".repeat(51)).is_err());
        assert!(SaveName::new("invalid name").is_err()); // spaces not allowed
    }

    #[test]
    fn test_special_stat_valid() {
        assert!(SpecialStat::new(1).is_ok());
        assert!(SpecialStat::new(5).is_ok());
        assert!(SpecialStat::new(10).is_ok());
    }

    #[test]
    fn test_special_stat_invalid() {
        assert!(SpecialStat::new(0).is_err());
        assert!(SpecialStat::new(11).is_err());
    }

    #[test]
    fn test_special_allocation_valid() {
        // Target total of 28 (common for Fallout character creation)
        assert!(SpecialAllocation::new(4, 4, 4, 4, 4, 4, 4, 28).is_ok());
        assert!(SpecialAllocation::new(10, 1, 5, 3, 4, 3, 2, 28).is_ok());
    }

    #[test]
    fn test_special_allocation_invalid_total() {
        // Wrong total
        assert!(SpecialAllocation::new(5, 5, 5, 5, 5, 5, 5, 28).is_err());
    }

    #[test]
    fn test_special_allocation_invalid_stat() {
        // Stat out of range (11 is invalid)
        assert!(SpecialAllocation::new(11, 4, 4, 4, 4, 4, 4, 28).is_err());
    }
}
