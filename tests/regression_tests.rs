/// Regression tests for previously identified bugs
mod helpers;

use helpers::*;

/// Regression test for bug: Character leveling only applied HP bonus once when
/// gaining multiple levels at once.
///
/// Bug fix: Changed add_experience to loop through each level gained and
/// apply the HP bonus for each level.
#[test]
fn test_regression_multi_level_hp_bonus() {
    let mut character = create_test_character("Test");
    let initial_hp = character.max_hp;
    let endurance = character.special.endurance as i32;

    // Jump from level 1 to level 6 (5000 XP)
    character.add_experience(5000);
    character.level_up();

    // Should have gained 5 levels (2, 3, 4, 5, 6)
    assert_eq!(character.level, 6);

    // HP should have increased 5 times
    let expected_hp = initial_hp + 5 * (5 + endurance);
    assert_eq!(
        character.max_hp, expected_hp,
        "Multi-level gain should apply HP bonus for each level"
    );
}

/// Regression test: Verify leveling by increments gives same result as jumping levels
#[test]
fn test_regression_incremental_vs_jump_leveling() {
    let special = fallout_dnd::game::character::Special {
        strength: 5,
        perception: 5,
        endurance: 5,
        charisma: 5,
        intelligence: 5,
        agility: 5,
        luck: 5,
    };

    // Character that levels incrementally
    let mut char_incremental = fallout_dnd::game::character::Character::new("Inc".to_string(), special.clone());
    char_incremental.add_experience(1000); // Level 2
    char_incremental.add_experience(1000); // Level 3
    char_incremental.add_experience(1000); // Level 4

    // Character that jumps levels
    let mut char_jump = fallout_dnd::game::character::Character::new("Jump".to_string(), special);
    char_jump.add_experience(3000); // Jump to level 4

    // Both should have same level and HP
    assert_eq!(char_incremental.level, char_jump.level);
    assert_eq!(char_incremental.max_hp, char_jump.max_hp,
        "Incremental and jump leveling should result in same HP");
}

/// Regression test: HP should never go negative
#[test]
fn test_regression_hp_never_negative() {
    let mut character = create_test_character("Test");

    // Deal massive damage
    character.take_damage(999999);

    assert_eq!(character.current_hp, 0, "HP should stop at 0");
    assert!(!character.is_alive());
}

/// Regression test: AP usage should not allow negative AP
#[test]
fn test_regression_ap_never_negative() {
    let mut character = create_test_character("Test");

    // Try to use more AP than available
    character.current_ap = 3;
    let success = character.use_ap(5);

    assert!(!success, "Should fail to use more AP than available");
    assert_eq!(character.current_ap, 3, "AP should remain unchanged");
}

/// Regression test: Consumable healing should not exceed max HP
#[test]
fn test_regression_healing_cap() {
    let mut character = create_test_character("Test");
    let max_hp = character.max_hp;

    // Take minimal damage
    character.take_damage(5);

    // Use stimpak (heals 30 HP from starting items)
    let _ = character.use_consumable("stimpak");

    assert_eq!(character.current_hp, max_hp, "Healing should not exceed max HP");
}

/// Regression test: Using non-existent item should not crash
#[test]
fn test_regression_invalid_item_use() {
    let mut character = create_test_character("Test");

    let result = character.use_consumable("nonexistent_item");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Item not found in inventory");
}

/// Regression test: Using non-consumable item should fail gracefully
#[test]
fn test_regression_use_weapon_as_consumable() {
    let mut character = create_test_character("Test");

    // Try to "consume" the starting pistol
    let result = character.use_consumable("10mm_pistol");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Item is not consumable");
}

/// Regression test: Combat state should reset properly
#[test]
fn test_regression_combat_state_reset() {
    use fallout_dnd::game::combat::{CombatState, Enemy};

    let mut combat = CombatState::new();

    // Start combat
    combat.start_combat(vec![Enemy::raider(1), Enemy::raider(1)]);
    assert!(combat.active);
    assert_eq!(combat.enemies.len(), 2);

    // End combat
    combat.end_combat();
    assert!(!combat.active);
    assert_eq!(combat.enemies.len(), 0);
    assert_eq!(combat.round, 0);
}

/// Regression test: Story manager FIFO behavior
#[test]
fn test_regression_story_manager_fifo() {
    use fallout_dnd::game::story_manager::StoryManager;

    let mut story = StoryManager::with_capacity(3);

    story.add("First".to_string());
    story.add("Second".to_string());
    story.add("Third".to_string());
    story.add("Fourth".to_string()); // Should evict "First"

    let all = story.get_all();
    let entries: Vec<&String> = all.iter().collect();

    assert_eq!(entries.len(), 3);
    assert!(!entries.iter().any(|e| *e == "First"), "Oldest entry should be evicted");
    assert!(entries.iter().any(|e| *e == "Fourth"), "Newest entry should be present");
}

/// Regression test: XP rewards calculation
#[test]
fn test_regression_xp_reward_calculation() {
    use fallout_dnd::game::combat::{CombatState, Enemy};

    let mut combat = CombatState::new();
    let raider = Enemy::raider(2); // Level 2 = 200 XP
    let radroach = Enemy::radroach(1); // Level 1 = 100 XP

    let expected_total = raider.xp_reward + radroach.xp_reward;

    combat.start_combat(vec![raider, radroach]);

    // Kill all enemies
    for enemy in &mut combat.enemies {
        enemy.current_hp = 0;
    }

    let actual_xp = combat.total_xp_reward();
    assert_eq!(actual_xp, expected_total, "XP rewards should sum correctly");
}
