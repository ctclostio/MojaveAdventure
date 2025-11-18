use fallout_dnd::game::{
    character::{Character, Special},
    persistence, GameState,
};

#[test]
fn test_character_creation_to_combat_flow() {
    // Create a new character with SPECIAL stats
    let special = Special::new();
    let character = Character::new("Test Hero".to_string(), special);

    // Verify initial state
    assert_eq!(character.name, "Test Hero");
    assert_eq!(character.level, 1);
    assert_eq!(character.current_hp, character.max_hp);
    assert_eq!(character.current_ap, character.max_ap);

    // Verify SPECIAL total is valid
    let total = character.special.strength
        + character.special.perception
        + character.special.endurance
        + character.special.charisma
        + character.special.intelligence
        + character.special.agility
        + character.special.luck;
    assert!(total > 0, "SPECIAL stats should be initialized");
}

#[test]
fn test_game_state_save_load_roundtrip() {
    use std::fs;

    // Create initial game state with a character
    let special = Special::new();
    let character = Character::new("Test Character".to_string(), special);
    let mut game_state = GameState::new(character);
    game_state.location = "Test Vault".to_string();
    // Test both conversation systems
    game_state.conversation.add_player_turn("I explore the vault".to_string());
    game_state.conversation.add_dm_turn("You find a locked door".to_string());
    game_state.story.add("Test event happened".to_string()); // Legacy support

    // Use a unique test save name
    let save_name = "test_integration_save";

    // Save the game
    let result = persistence::save_to_file(&game_state, save_name);
    assert!(result.is_ok(), "Save should succeed: {:?}", result.err());

    // Load the game
    let loaded = persistence::load_from_file(save_name);
    assert!(loaded.is_ok(), "Load should succeed: {:?}", loaded.err());

    let loaded_state = loaded.unwrap();
    assert_eq!(loaded_state.character.name, "Test Character");
    assert_eq!(loaded_state.location, "Test Vault");
    assert_eq!(loaded_state.story.len(), 1);

    // Cleanup - remove test save file
    let _ = fs::remove_file("saves/test_integration_save.json");
}

#[test]
fn test_combat_encounter_lifecycle() {
    use fallout_dnd::game::combat::Enemy;

    let special = Special::new();
    let character = Character::new("Test Hero".to_string(), special);
    let mut game_state = GameState::new(character);

    // Start combat with test enemies
    let enemies = vec![Enemy::radroach(1), Enemy::raider(1)];
    game_state.combat.start_combat(enemies);

    assert!(game_state.combat.active, "Combat should be active");
    assert_eq!(game_state.combat.enemies.len(), 2);
    assert_eq!(game_state.combat.round, 1);

    // Kill all enemies
    for enemy in &mut game_state.combat.enemies {
        enemy.current_hp = 0;
    }

    // Check all enemies dead
    assert!(game_state.combat.enemies.iter().all(|e| !e.is_alive()));

    // End combat
    game_state.combat.end_combat();
    assert!(!game_state.combat.active, "Combat should end");
    assert_eq!(game_state.combat.enemies.len(), 0);
}
