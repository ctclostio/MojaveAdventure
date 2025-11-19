/// Property-based tests using proptest for invariants
use proptest::prelude::*;
use fallout_dnd::game::combat::roll_dice;

proptest! {
    /// Test that dice rolls always return values within expected bounds
    #[test]
    fn test_dice_roll_bounds(num_dice in 1u32..10, die_size in 1u32..100, modifier in 0i32..50) {
        let dice_str = format!("{}d{}+{}", num_dice, die_size, modifier);

        let result = roll_dice(&dice_str);

        // Minimum: num_dice (all ones) + modifier
        let min_value = num_dice as i32 + modifier;
        // Maximum: num_dice * die_size + modifier
        let max_value = (num_dice * die_size) as i32 + modifier;

        prop_assert!(result >= min_value, "Roll {} gave {} which is less than min {}", dice_str, result, min_value);
        prop_assert!(result <= max_value, "Roll {} gave {} which is more than max {}", dice_str, result, max_value);
    }

    /// Test that single die rolls are always in range 1..=die_size
    #[test]
    fn test_single_die_range(die_size in 1u32..100) {
        let dice_str = format!("1d{}", die_size);
        let result = roll_dice(&dice_str);

        prop_assert!(result >= 1);
        prop_assert!(result <= die_size as i32);
    }

    /// Test that adding dice increases the range
    #[test]
    fn test_multiple_dice_range(num_dice in 2u32..20) {
        let dice_str = format!("{}d6", num_dice);
        let result = roll_dice(&dice_str);

        // With multiple dice, minimum is num_dice (all ones)
        prop_assert!(result >= num_dice as i32);
        // Maximum is num_dice * 6
        prop_assert!(result <= (num_dice * 6) as i32);
    }

    /// Test character HP is always non-negative after damage
    #[test]
    fn test_character_hp_non_negative(damage in 0i32..1000) {
        use fallout_dnd::game::character::{Character, Special};

        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);

        character.take_damage(damage);

        prop_assert!(character.current_hp >= 0, "HP should never go below 0");
        prop_assert!(character.current_hp <= character.max_hp, "Current HP should not exceed max HP");
    }

    /// Test character leveling with various XP amounts
    #[test]
    fn test_character_level_from_xp(xp in 0u32..50000) {
        use fallout_dnd::game::character::{Character, Special};

        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);
        let initial_max_hp = character.max_hp;

        character.add_experience(xp);
        if character.can_level_up() {
            character.level_up();
        }

        // Level should be 1 + (xp / 1000)
        let expected_level = 1 + (xp / 1000);
        prop_assert_eq!(character.level, expected_level);

        // HP should increase with leveling
        if expected_level > 1 {
            prop_assert!(character.max_hp >= initial_max_hp, "Max HP should not decrease");
        }

        // Current HP should equal max HP after leveling
        if character.level > 1 {
            prop_assert_eq!(character.current_hp, character.max_hp, "HP should be restored on level up");
        }
    }

    /// Test AP usage constraints
    #[test]
    fn test_ap_usage_constraints(ap_to_use in 0i32..20) {
        use fallout_dnd::game::character::{Character, Special};

        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);
        let initial_ap = character.current_ap;

        let success = character.use_ap(ap_to_use);

        if ap_to_use <= initial_ap {
            prop_assert!(success, "Should succeed when using <= available AP");
            prop_assert_eq!(character.current_ap, initial_ap - ap_to_use);
        } else {
            prop_assert!(!success, "Should fail when using > available AP");
            prop_assert_eq!(character.current_ap, initial_ap, "AP should not change on failed use");
        }
    }

    /// Test enemy scaling properties
    #[test]
    fn test_enemy_scaling_properties(level in 1u32..25) {
        use fallout_dnd::game::combat::Enemy;

        let enemy = Enemy::raider(level);

        // HP should scale with level
        prop_assert!(enemy.max_hp > 0);
        prop_assert_eq!(enemy.current_hp, enemy.max_hp);

        // Higher levels should have more HP
        if level > 1 {
            let lower_enemy = Enemy::raider(1);
            prop_assert!(enemy.max_hp >= lower_enemy.max_hp);
        }

        // Skill should be capped properly to avoid u8 overflow (max 255)
        // Raiders have formula: 40 + (level * 8).min(80), which caps at 120
        prop_assert!(enemy.skill <= 255, "Skill should not overflow u8");
        prop_assert!(enemy.skill <= 120, "Raider skill should cap at 120");

        // XP reward should scale with level
        prop_assert!(enemy.xp_reward > 0);
        prop_assert_eq!(enemy.xp_reward, level * 100);
    }

    /// Test story manager capacity constraints
    #[test]
    fn test_story_manager_capacity(capacity in 1usize..100, num_entries in 1usize..200) {
        use fallout_dnd::game::story_manager::StoryManager;

        let mut story = StoryManager::with_capacity(capacity);

        for i in 0..num_entries {
            story.add(format!("Entry {}", i));
        }

        // Should never exceed capacity
        prop_assert!(story.len() <= capacity, "Story length {} exceeds capacity {}", story.len(), capacity);

        // Should be at capacity if we added more entries than capacity
        if num_entries >= capacity {
            prop_assert_eq!(story.len(), capacity);
        } else {
            prop_assert_eq!(story.len(), num_entries);
        }
    }
}
