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

/// Validate a filename for security
///
/// Checks for path traversal attempts, invalid characters, and other security issues.
/// Returns an error if the filename is invalid.
fn validate_filename(filename: &str) -> Result<()> {
    // Check if empty
    if filename.is_empty() {
        return Err(GameError::InvalidInput("Empty filename".to_string()).into());
    }

    // Check for path separators and traversal
    if filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains('\0')  // Null byte injection
        || filename.contains(':')
    // Windows drive letters
    {
        return Err(GameError::PathTraversalError(filename.to_string()).into());
    }

    // Additional safety: ensure filename doesn't start with dot
    if filename.starts_with('.') {
        return Err(GameError::InvalidInput("Filename cannot start with '.'".to_string()).into());
    }

    // Limit filename length to prevent abuse
    if filename.len() > 100 {
        return Err(
            GameError::InvalidInput("Filename too long (max 100 chars)".to_string()).into(),
        );
    }

    Ok(())
}

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
    validate_filename(filename)?;

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
    validate_filename(filename)?;

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
    let game_state: GameState = serde_json::from_str(&json)?;
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
