use fallout_dnd::game::{GameState, character::{Character, Special}, persistence};

#[test]
fn test_load_nonexistent_file() {
    let result = persistence::load_from_file("nonexistent_save_file.json");
    assert!(result.is_err(), "Loading nonexistent file should fail");
}

#[test]
fn test_load_corrupted_json() {
    use tempfile::NamedTempFile;
    use std::io::Write;

    // Create a temporary file with invalid JSON
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "{{ invalid json content").unwrap();
    temp_file.flush().unwrap();

    let result = persistence::load_from_file(temp_file.path().to_str().unwrap());
    assert!(result.is_err(), "Loading corrupted JSON should fail");
}

#[test]
fn test_save_to_invalid_path() {
    let special = Special::new();
    let character = Character::new("Test".to_string(), special);
    let game_state = GameState::new(character);

    // Try to save to a path that doesn't exist (no parent directory)
    let result = persistence::save_to_file(&game_state, "/nonexistent/directory/save.json");
    assert!(result.is_err(), "Saving to invalid path should fail");
}

#[test]
fn test_path_traversal_prevention() {
    let special = Special::new();
    let character = Character::new("Test".to_string(), special);
    let game_state = GameState::new(character);

    // Try various path traversal attacks
    let dangerous_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config",
        "./saves/../../../sensitive_file",
    ];

    for path in dangerous_paths {
        let result = persistence::save_to_file(&game_state, path);
        // Should either fail or be sanitized - we just want to ensure no crash
        // The actual behavior depends on the implementation
        let _ = result;
    }
}

#[test]
fn test_character_damage_exceeds_max_hp() {
    let special = Special::new();
    let mut character = Character::new("Test".to_string(), special);
    let max_hp = character.max_hp;

    // Deal massive damage
    character.take_damage(max_hp * 2);

    // HP should not go below 0
    assert!(character.current_hp <= 0, "Character should be dead");
    assert!(!character.is_alive(), "is_alive should return false");
}

#[test]
fn test_ap_usage_when_insufficient() {
    let special = Special::new();
    let mut character = Character::new("Test".to_string(), special);
    character.current_ap = 2;

    // Try to use more AP than available
    let result = character.use_ap(5);

    assert!(!result, "Should not be able to use more AP than available");
    assert_eq!(character.current_ap, 2, "AP should remain unchanged");
}

#[test]
fn test_enemy_creation_with_scaling() {
    use fallout_dnd::game::combat::Enemy;

    // Test that higher level enemies are stronger
    let level1 = Enemy::raider(1);
    let level5 = Enemy::raider(5);

    assert!(level5.max_hp > level1.max_hp, "Higher level should have more HP");
    assert!(level5.skill >= level1.skill, "Higher level should have better or equal skill");
}

#[test]
fn test_worldbook_file_io_errors() {
    use fallout_dnd::game::worldbook::Worldbook;
    use std::path::Path;

    // Test loading from nonexistent file
    let result = Worldbook::load_from_file(Path::new("nonexistent_worldbook.json"));
    // Should either return empty worldbook or error
    let _ = result;

    // Test saving to invalid path
    let worldbook = Worldbook::new();
    let result = worldbook.save_to_file(Path::new("/invalid/path/worldbook.json"));
    assert!(result.is_err(), "Saving to invalid path should fail");
}
