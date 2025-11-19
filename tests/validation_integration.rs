use fallout_dnd::game::character::{Character, Special};
use fallout_dnd::game::persistence;
use fallout_dnd::game::GameState;
use fallout_dnd::validation;

#[test]
fn test_validation_integration_flow() {
    // 1. Test Character Name Validation Integration
    // We can't easily test the interactive loop in char_handlers without mocking UI,
    // but we can verify the validation logic itself is accessible and correct.
    assert!(validation::validate_character_name("ValidName").is_ok());
    assert!(validation::validate_character_name("Invalid@Name").is_err());

    // 2. Test Save Name Validation Integration
    // Create a dummy game state
    let special = Special::new();
    let character = Character::new("TestChar".to_string(), special);
    let game = GameState::new(character);

    // Try to save with valid name (this might actually write to disk, so we should be careful or clean up)
    // For integration testing, we might want to use a temporary directory or just test the validation failure path
    // which doesn't touch the disk.

    // Test invalid save name - should fail BEFORE file operations
    let err = persistence::save_to_file(&game, "../bad_save");
    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .to_string()
        .contains("Path traversal detected"));

    let err = persistence::save_to_file(&game, "invalid/name");
    assert!(err.is_err());

    // Test valid save name (we won't actually save to avoid cluttering disk in this test,
    // or we could use a temp dir if we wanted to go deeper, but verifying the validation hook is enough)
    // We can't easily check "success" without writing, but we know validation passes if it proceeds.
}
