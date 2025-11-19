/// Comprehensive tests for character leveling and experience system
mod helpers;

use fallout_dnd::game::character::{Character, Special};
use helpers::*;

#[test]
fn test_character_starts_at_level_1() {
    let character = create_test_character("Test");
    assert_eq!(character.level, 1);
    assert_eq!(character.experience, 0);
}

#[test]
fn test_add_experience_no_level_up() {
    let mut character = create_test_character("Test");
    let initial_level = character.level;
    let initial_max_hp = character.max_hp;

    character.add_experience(500);

    assert_eq!(character.experience, 500);
    assert!(!character.can_level_up());
    assert_eq!(
        character.level, initial_level,
        "Should not level up with 500 XP"
    );
    assert_eq!(
        character.max_hp, initial_max_hp,
        "HP should not change without level up"
    );
}

#[test]
fn test_add_experience_single_level_up() {
    let mut character = create_test_character("Test");
    let initial_max_hp = character.max_hp;
    let endurance = character.special.endurance as i32;

    character.add_experience(1000);
    assert!(character.can_level_up());
    character.level_up();

    assert_eq!(character.experience, 1000);
    assert_eq!(character.level, 2, "Should level up to 2 at 1000 XP");
    assert_eq!(
        character.max_hp,
        initial_max_hp + 5 + endurance,
        "HP should increase by 5 + endurance"
    );
    assert_eq!(
        character.current_hp, character.max_hp,
        "Current HP should be fully restored on level up"
    );
}

#[test]
fn test_add_experience_multiple_level_ups() {
    let mut character = create_test_character("Test");
    let initial_max_hp = character.max_hp;
    let endurance = character.special.endurance as i32;

    // Add 2500 XP = should reach level 3 (1 + 2500/1000 = 3)
    character.add_experience(2500);
    assert!(character.can_level_up());
    character.level_up();

    assert_eq!(character.experience, 2500);
    assert_eq!(character.level, 3, "Should level up to 3 at 2500 XP");

    // HP increases twice: level 1->2 and level 2->3
    let expected_hp = initial_max_hp + 2 * (5 + endurance);
    assert_eq!(character.max_hp, expected_hp);
}

#[test]
fn test_experience_threshold_boundary() {
    let mut character = create_test_character("Test");

    // Test just before level up (999 XP)
    character.add_experience(999);
    assert!(!character.can_level_up());
    assert_eq!(character.level, 1, "Should still be level 1 at 999 XP");

    // Add 1 more XP to cross threshold
    character.add_experience(1);
    assert!(character.can_level_up());
    character.level_up();
    assert_eq!(character.experience, 1000);
    assert_eq!(character.level, 2, "Should be level 2 at 1000 XP");
}

#[test]
fn test_experience_accumulation() {
    let mut character = create_test_character("Test");

    character.add_experience(300);
    assert_eq!(character.experience, 300);
    assert!(!character.can_level_up());

    character.add_experience(200);
    assert_eq!(character.experience, 500);
    assert!(!character.can_level_up());

    character.add_experience(600);
    assert_eq!(character.experience, 1100);
    assert!(character.can_level_up());
    character.level_up();
    assert_eq!(character.level, 2, "Should have leveled up");
}

#[test]
fn test_high_experience_multiple_levels() {
    let mut character = create_test_character("Test");

    // Add enough XP for 5 levels (5000 XP = level 6)
    character.add_experience(5000);
    assert!(character.can_level_up());
    character.level_up();

    assert_eq!(character.level, 6, "Should reach level 6 with 5000 XP");
    assert_eq!(character.experience, 5000);
}

#[test]
fn test_level_up_restores_hp() {
    let mut character = create_test_character("Test");

    // Damage the character
    character.current_hp = character.max_hp / 2;
    let damaged_hp = character.current_hp;

    // Level up
    character.add_experience(1000);
    assert!(character.can_level_up());
    character.level_up();

    assert_eq!(character.level, 2);
    assert!(
        character.current_hp > damaged_hp,
        "HP should be restored on level up"
    );
    assert_eq!(
        character.current_hp, character.max_hp,
        "HP should be fully restored"
    );
}

#[test]
fn test_endurance_affects_hp_gain() {
    let low_end = Special {
        strength: 5,
        perception: 5,
        endurance: 1, // Low endurance
        charisma: 5,
        intelligence: 5,
        agility: 5,
        luck: 5,
    };

    let high_end = Special {
        strength: 5,
        perception: 5,
        endurance: 10, // High endurance
        charisma: 5,
        intelligence: 5,
        agility: 5,
        luck: 5,
    };

    let mut char_low = Character::new("Low End".to_string(), low_end);
    let mut char_high = Character::new("High End".to_string(), high_end);

    let low_initial_hp = char_low.max_hp;
    let high_initial_hp = char_high.max_hp;

    // Level both up
    char_low.add_experience(1000);
    char_low.level_up();
    char_high.add_experience(1000);
    char_high.level_up();

    let low_hp_gain = char_low.max_hp - low_initial_hp;
    let high_hp_gain = char_high.max_hp - high_initial_hp;

    assert!(
        high_hp_gain > low_hp_gain,
        "Higher endurance should give more HP per level"
    );
    assert_eq!(low_hp_gain, 5 + 1, "Low endurance should give 6 HP");
    assert_eq!(high_hp_gain, 5 + 10, "High endurance should give 15 HP");
}

#[test]
fn test_character_damage_and_healing() {
    let mut character = create_test_character("Test");
    let max_hp = character.max_hp;

    // Take damage
    character.take_damage(20);
    assert_eq!(character.current_hp, max_hp - 20);

    // Heal
    character.heal(10);
    assert_eq!(character.current_hp, max_hp - 10);

    // Heal beyond max
    character.heal(100);
    assert_eq!(character.current_hp, max_hp, "HP should not exceed max");
}

#[test]
fn test_character_death() {
    let mut character = create_test_character("Test");

    character.take_damage(1000);

    assert_eq!(character.current_hp, 0, "HP should not go below 0");
    assert!(!character.is_alive(), "Character should be dead");
}

#[test]
fn test_ap_usage() {
    let mut character = create_test_character("Test");
    let max_ap = character.max_ap;

    // Use some AP
    let success = character.use_ap(3);
    assert!(success, "Should be able to use 3 AP");
    assert_eq!(character.current_ap, max_ap - 3);

    // Try to use more AP than available
    character.current_ap = 2;
    let success = character.use_ap(5);
    assert!(!success, "Should not be able to use more AP than available");
    assert_eq!(character.current_ap, 2, "AP should remain unchanged");
}

#[test]
fn test_ap_restoration() {
    let mut character = create_test_character("Test");
    let max_ap = character.max_ap;

    character.use_ap(5);
    character.restore_ap();

    assert_eq!(character.current_ap, max_ap, "AP should be fully restored");
}

#[test]
fn test_special_stats_initialization() {
    let special = Special::new();

    // SPECIAL stats should sum to a reasonable total
    let total = special.strength
        + special.perception
        + special.endurance
        + special.charisma
        + special.intelligence
        + special.agility
        + special.luck;

    assert!(total > 0, "SPECIAL stats should be initialized");
    assert!(
        total <= 70,
        "SPECIAL total should not exceed maximum (10*7)"
    );
}

#[test]
fn test_custom_special_stats() {
    let special = Special {
        strength: 10,
        perception: 1,
        endurance: 1,
        charisma: 1,
        intelligence: 1,
        agility: 1,
        luck: 10,
    };

    let character = Character::new("Test".to_string(), special);

    assert_eq!(character.special.strength, 10);
    assert_eq!(character.special.luck, 10);
    assert_eq!(character.special.perception, 1);
}

#[test]
fn test_skills_derived_from_special() {
    let special = Special {
        strength: 10,
        perception: 8,
        endurance: 5,
        charisma: 7,
        intelligence: 9,
        agility: 6,
        luck: 5,
    };

    let character = Character::new("Test".to_string(), special);

    // Skills should be influenced by SPECIAL stats
    // The exact formula depends on implementation, but higher stats should give higher skills
    assert!(character.skills.small_guns > 0);
    assert!(character.skills.speech > 0);
    assert!(character.skills.science > 0);
}

#[test]
fn test_character_inventory_starts_with_items() {
    let character = create_test_character("Test");

    assert!(!character.inventory.is_empty(), "Should start with items");
    assert!(
        character.equipped_weapon.is_some(),
        "Should have a weapon equipped"
    );
}

#[test]
fn test_caps_management() {
    let mut character = create_test_character("Test");
    let initial_caps = character.caps;

    character.caps += 100;
    assert_eq!(character.caps, initial_caps + 100);

    character.caps -= 50;
    assert_eq!(character.caps, initial_caps + 50);
}

#[test]
fn test_character_traits_and_perks() {
    let mut character = create_test_character("Test");

    assert_eq!(character.traits.len(), 0, "Should start with no traits");
    assert_eq!(character.perks.len(), 0, "Should start with no perks");

    character.traits.push("Gifted".to_string());
    character.perks.push("Toughness".to_string());

    assert_eq!(character.traits.len(), 1);
    assert_eq!(character.perks.len(), 1);
}
