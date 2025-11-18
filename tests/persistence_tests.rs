/// Tests for game state persistence (simplified version)
mod helpers;

use fallout_dnd::game::{persistence};
use helpers::*;

#[test]
fn test_save_and_load_basic_game_state() {
    let character = create_test_character("Hero");
    let game_state = fallout_dnd::game::GameState::new(character);

    let filename = "test_basic_save";

    // Save
    let save_result = persistence::save_to_file(&game_state, filename);
    assert!(save_result.is_ok(), "Save should succeed: {:?}", save_result.err());

    // Load
    let load_result = persistence::load_from_file(filename);
    assert!(save_result.is_ok(), "Load should succeed: {:?}", load_result.err());

    let loaded = load_result.unwrap();
    assert_eq!(loaded.character.name, "Hero");

    // Cleanup
    let _ = std::fs::remove_file(format!("saves/{}.json", filename));
}

#[test]
fn test_save_with_modified_character() {
    let mut character = create_test_character("TestHero");
    character.add_experience(1500);
    character.take_damage(10);

    let game_state = fallout_dnd::game::GameState::new(character);
    let filename = "test_modified_save";

    persistence::save_to_file(&game_state, filename).unwrap();
    let loaded = persistence::load_from_file(filename).unwrap();

    assert_eq!(loaded.character.name, "TestHero");
    assert_eq!(loaded.character.experience, 1500);
    assert_eq!(loaded.character.level, 2);

    // Cleanup
    let _ = std::fs::remove_file(format!("saves/{}.json", filename));
}

#[test]
fn test_filename_validation() {
    let character = create_test_character("Hero");
    let game_state = fallout_dnd::game::GameState::new(character);

    // Path traversal should fail
    let result = persistence::save_to_file(&game_state, "../etc/passwd");
    assert!(result.is_err(), "Path traversal should be rejected");

    // Slashes should fail
    let result = persistence::save_to_file(&game_state, "save/test");
    assert!(result.is_err(), "Slashes should be rejected");

    // Dot files should fail
    let result = persistence::save_to_file(&game_state, ".hidden");
    assert!(result.is_err(), "Dot files should be rejected");
}
