//! # Persistence Module
//!
//! Handles saving and loading game state to/from disk.
//!
//! This module provides functions for:
//! - Saving game state to JSON files
//! - Loading game state from JSON files
//! - Listing available save files
//! - Interactive save/load with user prompts

use super::GameState;
use crate::error::GameError;
use crate::ui::UI;
use anyhow::Result;
use colored::*;
use std::fs;
use std::path::Path;

/// Save a game state to a file
///
/// Validates the filename, creates the saves directory if needed,
/// and writes the game state as JSON.
///
/// # Security
/// - Validates filename to prevent path traversal attacks
/// - Ensures saves are contained within the saves/ directory
/// - Limits filename length to prevent abuse
pub fn save_to_file(game_state: &GameState, filename: &str) -> Result<()> {
    crate::validation::validate_save_name(filename)?;

    // Ensure saves directory exists
    let saves_dir = Path::new("saves");
    if !saves_dir.exists() {
        fs::create_dir_all(saves_dir)?;
    }

    let json = serde_json::to_string_pretty(game_state)?;
    let save_path = saves_dir.join(format!("{}.json", filename));

    // Final safety check: ensure the path is within saves/ directory
    // We can't canonicalize a file that doesn't exist yet, so check the parent
    if let Some(parent) = save_path.parent() {
        let canonical_parent = parent.canonicalize().unwrap_or(parent.to_path_buf());
        let canonical_saves = saves_dir.canonicalize().unwrap_or(saves_dir.to_path_buf());

        if canonical_parent != canonical_saves {
            return Err(
                GameError::PathTraversalError("Path escapes saves directory".to_string()).into(),
            );
        }
    }

    fs::write(save_path, json)?;
    Ok(())
}

/// Load a game state from a file
///
/// Validates the filename and loads the game state from JSON.
///
/// # Security
/// - Validates filename to prevent path traversal attacks
/// - Ensures loads are contained within the saves/ directory
pub fn load_from_file(filename: &str) -> Result<GameState> {
    crate::validation::validate_save_name(filename)?;

    let saves_dir = Path::new("saves");
    let save_path = saves_dir.join(format!("{}.json", filename));

    // Final safety check: ensure the canonical path is within saves/
    let canonical_save = save_path.canonicalize().unwrap_or(save_path.clone());
    let canonical_saves = saves_dir.canonicalize().unwrap_or(saves_dir.to_path_buf());

    if !canonical_save.starts_with(&canonical_saves) {
        return Err(
            GameError::PathTraversalError("Path escapes saves directory".to_string()).into(),
        );
    }

    let json = fs::read_to_string(canonical_save)?;
    let mut game_state: GameState = serde_json::from_str(&json)?;

    // Migrate legacy story context to new conversation system if needed
    game_state.migrate_story_to_conversation();

    Ok(game_state)
}

/// List all available save files
///
/// Returns a vector of save file names (without .json extension).
pub fn list_save_files() -> Vec<String> {
    let mut saves = Vec::new();
    if let Ok(entries) = fs::read_dir("saves") {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    saves.push(filename.trim_end_matches(".json").to_string());
                }
            }
        }
    }
    saves
}

/// Interactive save game with user prompt
///
/// Prompts the user for a filename and saves the game state.
pub fn save_game(game_state: &GameState) {
    let filename = UI::prompt("Save name:");
    match save_to_file(game_state, &filename) {
        Ok(_) => UI::print_success("Game saved!"),
        Err(e) => UI::print_error(&format!("Save failed: {}", e)),
    }
}

/// Interactive load game with save file selection
///
/// Shows available saves and prompts the user to select one.
/// Returns Some(GameState) if successful, None if cancelled or failed.
pub fn load_game() -> Option<GameState> {
    let saves = list_save_files();

    if saves.is_empty() {
        UI::print_error("No saved games found.");
        return None;
    }

    println!("{}", "SAVED GAMES:".bold());
    for (i, save) in saves.iter().enumerate() {
        println!("  {}. {}", i + 1, save);
    }
    println!();

    let choice = UI::prompt("Enter save number (or 'cancel'):");

    if choice == "cancel" {
        return None;
    }

    if let Ok(index) = choice.parse::<usize>() {
        if index > 0 && index <= saves.len() {
            match load_from_file(&saves[index - 1]) {
                Ok(game_state) => {
                    UI::print_success("Game loaded!");
                    return Some(game_state);
                }
                Err(e) => UI::print_error(&format!("Failed to load game: {}", e)),
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::character::{Character, Special};
    use std::fs;

    fn create_test_game_state() -> GameState {
        let special = Special {
            strength: 6,
            perception: 7,
            endurance: 5,
            charisma: 4,
            intelligence: 8,
            agility: 6,
            luck: 5,
        };
        let character = Character::new("TestHero".to_string(), special);
        GameState::new(character)
    }

    fn cleanup_test_saves() {
        let _ = fs::remove_dir_all("saves");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        cleanup_test_saves();

        let game_state = create_test_game_state();

        // Save the game
        save_to_file(&game_state, "test_roundtrip").unwrap();

        // Load it back
        let loaded_state = load_from_file("test_roundtrip").unwrap();

        // Verify key fields match
        assert_eq!(loaded_state.character.name, game_state.character.name);
        assert_eq!(
            loaded_state.character.special.strength,
            game_state.character.special.strength
        );
        assert_eq!(loaded_state.day, game_state.day);

        cleanup_test_saves();
    }

    #[test]
    fn test_save_creates_directory() {
        cleanup_test_saves();

        let game_state = create_test_game_state();

        // Saves directory should not exist yet
        assert!(!Path::new("saves").exists());

        // Save should create it
        save_to_file(&game_state, "test_creates_dir").unwrap();

        assert!(Path::new("saves").exists());
        assert!(Path::new("saves/test_creates_dir.json").exists());

        cleanup_test_saves();
    }

    #[test]
    fn test_save_rejects_path_traversal() {
        let game_state = create_test_game_state();

        // Try various path traversal attacks
        assert!(save_to_file(&game_state, "../escape").is_err());
        assert!(save_to_file(&game_state, "..\\escape").is_err());
        assert!(save_to_file(&game_state, "../../etc/passwd").is_err());
        assert!(save_to_file(&game_state, "/etc/passwd").is_err());
    }

    #[test]
    fn test_load_rejects_path_traversal() {
        // Try various path traversal attacks
        assert!(load_from_file("../escape").is_err());
        assert!(load_from_file("..\\escape").is_err());
        assert!(load_from_file("../../etc/passwd").is_err());
    }

    #[test]
    fn test_load_nonexistent_file() {
        fs::create_dir_all("saves").unwrap();

        let result = load_from_file("nonexistent_file_12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_save_files_empty() {
        cleanup_test_saves();

        // No saves directory
        let saves = list_save_files();
        assert_eq!(saves.len(), 0);
    }

    #[test]
    fn test_list_save_files_multiple() {
        cleanup_test_saves();

        let game_state = create_test_game_state();

        // Create multiple saves
        save_to_file(&game_state, "test_save1").unwrap();
        save_to_file(&game_state, "test_save2").unwrap();
        save_to_file(&game_state, "test_save3").unwrap();

        let saves = list_save_files();
        assert_eq!(saves.len(), 3);
        assert!(saves.contains(&"test_save1".to_string()));
        assert!(saves.contains(&"test_save2".to_string()));
        assert!(saves.contains(&"test_save3".to_string()));

        cleanup_test_saves();
    }

    #[test]
    fn test_save_file_is_valid_json() {
        cleanup_test_saves();

        let game_state = create_test_game_state();
        save_to_file(&game_state, "test_json_check").unwrap();

        // Read the file and verify it's valid JSON
        let json_content = fs::read_to_string("saves/test_json_check.json").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Verify some expected fields exist in JSON
        assert!(parsed.get("character").is_some());
        assert!(parsed.get("day").is_some());
        assert!(parsed.get("location").is_some());

        cleanup_test_saves();
    }

    #[test]
    fn test_save_preserves_game_progress() {
        cleanup_test_saves();

        let mut game_state = create_test_game_state();

        // Modify game state
        game_state.character.add_experience(500);
        game_state.day = 42;
        game_state.location = "The Wasteland".to_string();

        // Save and reload
        save_to_file(&game_state, "test_progress").unwrap();
        let loaded = load_from_file("test_progress").unwrap();

        // Verify progress was preserved
        assert_eq!(loaded.character.experience, 500);
        assert_eq!(loaded.day, 42);
        assert_eq!(loaded.location, "The Wasteland");

        cleanup_test_saves();
    }

    #[test]
    fn test_save_filename_validation() {
        let game_state = create_test_game_state();

        // Invalid filenames should be rejected
        assert!(save_to_file(&game_state, "").is_err());
        assert!(save_to_file(&game_state, "   ").is_err());
        assert!(save_to_file(&game_state, "has/slash").is_err());
        assert!(save_to_file(&game_state, "has\\backslash").is_err());

        // Valid filename should work
        cleanup_test_saves();
        assert!(save_to_file(&game_state, "valid_name_test").is_ok());
        assert!(save_to_file(&game_state, "name-with-dash").is_ok());
        assert!(save_to_file(&game_state, "name_with_123").is_ok());

        cleanup_test_saves();
    }
}
