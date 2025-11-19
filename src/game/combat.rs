//! # Combat Module
//!
//! Turn-based combat system with action points and tactical decisions.
//!
//! ## Overview
//!
//! This module implements a turn-based combat system inspired by classic Fallout:
//! - **Action Points (AP)**: Each action costs AP (shooting, reloading, using items)
//! - **To-Hit Calculations**: Based on skill, range, and armor class
//! - **Enemy Types**: Various wasteland creatures with different stats
//! - **Round-Based**: Combat progresses in rounds with player and enemy turns
//!
//! ## Combat Flow
//!
//! 1. Combat begins when enemies are encountered
//! 2. Player chooses actions (attack, use item, flee)
//! 3. Each action costs AP
//! 4. When player ends turn, enemies act
//! 5. Round ends, AP regenerates
//! 6. Combat continues until enemies are defeated or player flees/dies
//!
//! ## Action Point System
//!
//! - **Shooting**: Costs 4 AP
//! - **Reloading**: Costs 2 AP
//! - **Using Item**: Costs 2 AP
//! - AP regenerates fully at the start of each round
//!
//! ## Enemy Types
//!
//! - **Radroach**: Weak, low HP, common in vaults
//! - **Raider**: Human enemies with guns, scales with level
//! - **Super Mutant**: Tough, high HP and damage, elite enemy
//!
//! ## To-Hit Calculation
//!
//! Hit chance is calculated as:
//! ```text
//! base_chance = skill + (perception * 2)
//! modified_chance = base_chance - (enemy_armor_class - 10)
//! final_chance = clamp(modified_chance, 5%, 95%)
//! ```
//!
//! ## Example
//!
//! ```no_run
//! use fallout_dnd::game::combat::{CombatState, Enemy};
//!
//! let mut combat = CombatState::new();
//! combat.start_combat(vec![
//!     Enemy::raider(3),
//!     Enemy::raider(3),
//! ]);
//!
//! println!("Combat started! Round {}", combat.round);
//! println!("Enemies: {}", combat.enemies.len());
//! ```

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Combat state tracking active encounters.
///
/// Manages the combat encounter including enemies, round counter,
/// and combat status (active/inactive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub active: bool,
    pub round: u32,
    pub enemies: Vec<Enemy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub level: u32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub armor_class: i32,
    pub damage: String,
    pub ap: i32,
    pub xp_reward: u32,
    pub skill: u8,    // Combat skill (like small_guns for raiders)
    pub strength: u8, // For melee damage bonus
}

impl Enemy {
    pub fn new(name: &str, level: u32) -> Self {
        let max_hp = 20 + (level as i32 * 10);
        Enemy {
            name: name.to_string(),
            level,
            max_hp,
            current_hp: max_hp,
            armor_class: 10 + level as i32,
            damage: format!("{}d6+{}", 1 + level / 3, level),
            ap: 5 + (level as i32 / 2),
            xp_reward: level * 100,
            skill: 30 + (level as u8 * 10).min(70), // Scales with level, caps at 70
            strength: 5,
        }
    }

    pub fn raider(level: u32) -> Self {
        let mut enemy = Enemy::new(&format!("Raider (Level {})", level), level);
        enemy.skill = 40 + (level as u8 * 8).min(80); // Raiders are better shots
        enemy.strength = 5 + level as u8;
        enemy
    }

    pub fn radroach(level: u32) -> Self {
        let mut enemy = Enemy::new(&format!("Radroach"), level);
        enemy.max_hp = 10 + (level as i32 * 3);
        enemy.current_hp = enemy.max_hp;
        enemy.damage = "1d4".to_string();
        enemy.skill = 20; // Low combat skill
        enemy.strength = 2; // Weak
        enemy.armor_class = 8;
        enemy
    }

    pub fn super_mutant(level: u32) -> Self {
        let mut enemy = Enemy::new(&format!("Super Mutant"), level + 2);
        enemy.max_hp = 40 + (level as i32 * 15);
        enemy.current_hp = enemy.max_hp;
        enemy.damage = format!("{}d8+{}", 1 + level / 2, level * 2);
        enemy.skill = 60 + (level as u8 * 5).min(90); // Very skilled
        enemy.strength = 10 + level as u8; // Very strong
        enemy.armor_class = 15 + level as i32;
        enemy
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.current_hp = (self.current_hp - damage).max(0);
    }
}

/// Dice rolling system
pub fn roll_dice(dice_str: &str) -> i32 {
    let mut rng = rand::thread_rng();

    // Parse dice notation like "2d6+3" or "1d20"
    // Handle both numeric modifiers and text (like "1d8+STR" -> "1d8+0" for basic parsing)
    let parts: Vec<&str> = dice_str.split('+').collect();
    let dice_part = parts[0];
    let modifier: i32 = if parts.len() > 1 {
        // Try to parse as number, if it fails (like "STR"), return 0
        parts[1].parse().unwrap_or(0)
    } else {
        0
    };

    let dice_components: Vec<&str> = dice_part.split('d').collect();
    if dice_components.len() != 2 {
        return modifier; // Invalid format, return just modifier
    }

    let num_dice: i32 = dice_components[0].parse().unwrap_or(1);
    let die_size: i32 = dice_components[1].parse().unwrap_or(6);

    let mut total = modifier;
    for _ in 0..num_dice {
        total += rng.gen_range(1..=die_size);
    }

    total
}

/// Replace stat modifiers in damage string with actual values
/// e.g., "1d8+STR" with STR=6 becomes "1d8+3" (STR/2)
pub fn resolve_stat_modifiers(damage_str: &str, strength: u8) -> String {
    if damage_str.contains("STR") {
        let stat_bonus = (strength / 2) as i32;
        damage_str.replace("STR", &stat_bonus.to_string())
    } else {
        damage_str.to_string()
    }
}

/// Make an attack roll
pub fn attack_roll(attacker_skill: u8, target_ac: i32) -> (bool, bool) {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(1..=20);

    let critical = roll == 20;
    let total = roll + attacker_skill as i32;
    let hit = total >= target_ac || critical;

    (hit, critical)
}

/// Calculate damage with modifiers
pub fn calculate_damage(base_damage: &str, stat_bonus: i32, is_critical: bool) -> i32 {
    let base = roll_dice(base_damage);
    let damage = base + stat_bonus;

    if is_critical {
        damage * 2
    } else {
        damage
    }
}

impl CombatState {
    pub fn new() -> Self {
        CombatState {
            active: false,
            round: 0,
            enemies: Vec::new(),
        }
    }

    pub fn start_combat(&mut self, enemies: Vec<Enemy>) {
        self.active = true;
        self.round = 1;
        self.enemies = enemies;
    }

    pub fn end_combat(&mut self) {
        self.active = false;
        self.round = 0;
        self.enemies.clear();
    }

    pub fn next_round(&mut self) {
        self.round += 1;
    }

    pub fn all_enemies_dead(&self) -> bool {
        self.enemies.iter().all(|e| !e.is_alive())
    }

    pub fn total_xp_reward(&self) -> u32 {
        self.enemies
            .iter()
            .filter(|e| !e.is_alive())
            .map(|e| e.xp_reward)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_dice_basic() {
        // Test basic dice rolling - result should be between min and max
        let result = roll_dice("1d6");
        assert!(
            result >= 1 && result <= 6,
            "1d6 should be between 1 and 6, got {}",
            result
        );

        let result = roll_dice("2d6");
        assert!(
            result >= 2 && result <= 12,
            "2d6 should be between 2 and 12, got {}",
            result
        );
    }

    #[test]
    fn test_roll_dice_with_modifier() {
        let result = roll_dice("1d6+3");
        assert!(
            result >= 4 && result <= 9,
            "1d6+3 should be between 4 and 9, got {}",
            result
        );

        let result = roll_dice("2d6+5");
        assert!(
            result >= 7 && result <= 17,
            "2d6+5 should be between 7 and 17, got {}",
            result
        );
    }

    #[test]
    fn test_roll_dice_invalid_format() {
        // Should handle invalid formats gracefully
        // Invalid format with no 'd' returns the modifier (0 if no +)
        let result = roll_dice("invalid");
        // The function tries to parse and may return a random value for malformed input
        // Just verify it doesn't panic
        assert!(
            result >= 0,
            "Should handle invalid format without panicking"
        );
    }

    #[test]
    fn test_resolve_stat_modifiers() {
        let result = resolve_stat_modifiers("1d8+STR", 6);
        assert_eq!(result, "1d8+3", "STR 6 should become +3");

        let result = resolve_stat_modifiers("1d10+2", 8);
        assert_eq!(result, "1d10+2", "No STR should remain unchanged");
    }

    #[test]
    fn test_calculate_damage() {
        let damage = calculate_damage("1d6+0", 0, false);
        assert!(damage >= 1 && damage <= 6);

        let damage = calculate_damage("1d6+0", 0, true);
        assert!(damage >= 2 && damage <= 12, "Critical should double damage");
    }

    #[test]
    fn test_dice_roll() {
        for _ in 0..100 {
            let roll = roll_dice("1d20");
            assert!(roll >= 1 && roll <= 20);
        }
    }

    #[test]
    fn test_attack_roll() {
        // Test high skill against low AC (should always hit)
        let (hit, _) = attack_roll(100, 5);
        assert!(hit);

        // Test low skill against high AC (should usually miss)
        // We can't guarantee a miss because of critical hits, so we check that
        // if it's a hit, it must be a critical.
        let (hit, critical) = attack_roll(5, 100);
        if hit {
            assert!(critical);
        }
    }

    #[test]
    fn test_enemy_creation() {
        let raider = Enemy::raider(1);
        assert_eq!(raider.level, 1);
        assert!(raider.is_alive());
        assert_eq!(raider.skill, 48); // 40 + 1*8

        let radroach = Enemy::radroach(1);
        assert_eq!(radroach.max_hp, 13); // 10 + 1*3
        assert_eq!(radroach.skill, 20);
        assert_eq!(radroach.strength, 2);

        let super_mutant = Enemy::super_mutant(1);
        assert!(super_mutant.max_hp > raider.max_hp);
        assert!(super_mutant.skill > raider.skill);
        assert!(super_mutant.strength > raider.strength);
    }

    #[test]
    fn test_enemy_damage() {
        let mut enemy = Enemy::raider(1);
        let initial_hp = enemy.current_hp;

        enemy.take_damage(5);
        assert_eq!(enemy.current_hp, initial_hp - 5);

        enemy.take_damage(1000);
        assert_eq!(enemy.current_hp, 0);
        assert!(!enemy.is_alive());
    }

    #[test]
    fn test_combat_state() {
        let mut combat = CombatState::new();
        assert!(!combat.active);

        let enemies = vec![Enemy::raider(1), Enemy::raider(1)];
        combat.start_combat(enemies);
        assert!(combat.active);
        assert_eq!(combat.round, 1);
        assert_eq!(combat.enemies.len(), 2);

        combat.next_round();
        assert_eq!(combat.round, 2);

        combat.end_combat();
        assert!(!combat.active);
        assert_eq!(combat.enemies.len(), 0);
    }

    #[test]
    fn test_all_enemies_dead() {
        let mut combat = CombatState::new();
        let enemies = vec![Enemy::raider(1), Enemy::raider(1)];
        combat.start_combat(enemies);

        assert!(!combat.all_enemies_dead());

        combat.enemies[0].take_damage(1000);
        assert!(!combat.all_enemies_dead());

        combat.enemies[1].take_damage(1000);
        assert!(combat.all_enemies_dead());
    }

    #[test]
    fn test_xp_reward() {
        let mut combat = CombatState::new();
        let enemies = vec![Enemy::raider(1), Enemy::raider(1)];
        combat.start_combat(enemies);

        combat.enemies[0].take_damage(1000);
        let xp = combat.total_xp_reward();
        assert_eq!(xp, 100); // One level 1 raider

        combat.enemies[1].take_damage(1000);
        let xp = combat.total_xp_reward();
        assert_eq!(xp, 200); // Two level 1 raiders
    }
}
