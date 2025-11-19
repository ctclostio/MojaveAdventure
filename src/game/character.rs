//! # Character Module
//!
//! Character creation, SPECIAL attributes, skills, and progression.
//!
//! ## Overview
//!
//! This module implements the core Fallout character system including:
//! - **SPECIAL** attributes (Strength, Perception, Endurance, Charisma, Intelligence, Agility, Luck)
//! - **Skills** derived from SPECIAL stats (18 different skills including Small Guns, Speech, Science)
//! - **Character progression** with leveling, XP, and stat increases
//! - **Inventory management** for items and equipment
//!
//! ## SPECIAL System
//!
//! The SPECIAL system defines a character's core attributes:
//! - **Strength**: Physical power, melee damage, carrying capacity
//! - **Perception**: Awareness, accuracy, detecting hidden objects
//! - **Endurance**: Health points, radiation resistance, stamina
//! - **Charisma**: Speech effectiveness, barter prices, companion loyalty
//! - **Intelligence**: Skill points per level, hacking ability
//! - **Agility**: Action points, combat speed, weapon handling
//! - **Luck**: Critical hit chance, gambling success, general fortune
//!
//! ## Skills
//!
//! Skills are derived from SPECIAL stats using Fallout-like formulas.
//! For example:
//! - Small Guns = 5 + (Agility × 4)
//! - Speech = 0 + (Charisma × 5)
//! - Science = 0 + (Intelligence × 4)
//!
//! ## Character Progression
//!
//! Characters gain experience points (XP) from combat and quests.
//! Each level requires more XP: Level 2 needs 2000 XP, Level 3 needs 3000 XP, etc.
//!
//! ## Example
//!
//! ```no_run
//! use fallout_dnd::game::character::{Character, Special};
//!
//! // Create a new character
//! let mut special = Special::new();
//! special.strength = 6;
//! special.intelligence = 8;
//! special.luck = 7;
//! let mut character = Character::new("Vault Dweller".to_string(), special);
//!
//! println!("HP: {}/{}", character.current_hp, character.max_hp);
//! println!("Science skill: {}", character.skills.science);
//! ```

use super::items::Item;
use serde::{Deserialize, Serialize};

/// SPECIAL stats - core Fallout character attributes
///
/// The SPECIAL system is the foundation of character attributes in Fallout.
/// Each stat ranges from 1-10 and affects derived stats and skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Special {
    pub strength: u8,     // Physical power, melee damage
    pub perception: u8,   // Awareness, accuracy
    pub endurance: u8,    // Health, radiation resistance
    pub charisma: u8,     // Speech, barter
    pub intelligence: u8, // Skill points, hacking
    pub agility: u8,      // Action points, speed
    pub luck: u8,         // Critical chance, general fortune
}

impl Default for Special {
    fn default() -> Self {
        Self::new()
    }
}

impl Special {
    pub fn new() -> Self {
        Special {
            strength: 1,
            perception: 1,
            endurance: 1,
            charisma: 1,
            intelligence: 1,
            agility: 1,
            luck: 1,
        }
    }

    #[allow(dead_code)]
    pub fn total_points(&self) -> u8 {
        self.strength
            + self.perception
            + self.endurance
            + self.charisma
            + self.intelligence
            + self.agility
            + self.luck
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub small_guns: u8,
    pub big_guns: u8,
    pub energy_weapons: u8,
    pub unarmed: u8,
    pub melee_weapons: u8,
    pub throwing: u8,
    pub first_aid: u8,
    pub doctor: u8,
    pub sneak: u8,
    pub lockpick: u8,
    pub steal: u8,
    pub traps: u8,
    pub science: u8,
    pub repair: u8,
    pub speech: u8,
    pub barter: u8,
    pub gambling: u8,
    pub outdoorsman: u8,
}

impl Skills {
    pub fn from_special(special: &Special) -> Self {
        Skills {
            small_guns: 5 + (special.agility * 4),
            big_guns: special.agility * 2,
            energy_weapons: special.agility * 2,
            unarmed: 30 + ((special.agility + special.strength) * 2),
            melee_weapons: 20 + ((special.agility + special.strength) * 2),
            throwing: special.agility * 4,
            first_aid: (special.perception + special.intelligence) * 2,
            doctor: 5 + (special.perception + special.intelligence),
            sneak: 5 + (special.agility * 3),
            lockpick: 10 + (special.perception + special.agility),
            steal: special.agility * 3,
            traps: 10 + (special.perception + special.agility),
            science: special.intelligence * 4,
            repair: special.intelligence * 3,
            speech: special.charisma * 5,
            barter: special.charisma * 4,
            gambling: special.luck * 5,
            outdoorsman: special.endurance + special.intelligence,
        }
    }

    /// Get a skill value by name - useful for dynamic skill checks
    #[allow(dead_code)]
    pub fn get_skill(&self, name: &str) -> u8 {
        match name {
            "small_guns" => self.small_guns,
            "big_guns" => self.big_guns,
            "energy_weapons" => self.energy_weapons,
            "unarmed" => self.unarmed,
            "melee_weapons" => self.melee_weapons,
            "throwing" => self.throwing,
            "first_aid" => self.first_aid,
            "doctor" => self.doctor,
            "sneak" => self.sneak,
            "lockpick" => self.lockpick,
            "steal" => self.steal,
            "traps" => self.traps,
            "science" => self.science,
            "repair" => self.repair,
            "speech" => self.speech,
            "barter" => self.barter,
            "gambling" => self.gambling,
            "outdoorsman" => self.outdoorsman,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub level: u32,
    pub experience: u32,
    pub special: Special,
    pub skills: Skills,
    pub max_hp: i32,
    pub current_hp: i32,
    pub max_ap: i32,
    pub current_ap: i32,
    pub caps: u32,
    pub inventory: Vec<Item>,
    pub equipped_weapon: Option<String>,
    pub equipped_armor: Option<String>,
    pub traits: Vec<String>,
    pub perks: Vec<String>,
}

impl Character {
    pub fn new(name: String, special: Special) -> Self {
        let skills = Skills::from_special(&special);
        let max_hp = 15 + special.strength as i32 + (special.endurance as i32 * 2);
        let max_ap = 5 + (special.agility as i32 / 2);

        Character {
            name,
            level: 1,
            experience: 0,
            special,
            skills,
            max_hp,
            current_hp: max_hp,
            max_ap,
            current_ap: max_ap,
            caps: 500,
            inventory: super::items::get_starting_items(),
            equipped_weapon: Some("10mm_pistol".to_string()),
            equipped_armor: None,
            traits: Vec::new(),
            perks: Vec::new(),
        }
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.current_hp = (self.current_hp - damage).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn restore_ap(&mut self) {
        self.current_ap = self.max_ap;
    }

    pub fn use_ap(&mut self, amount: i32) -> bool {
        if self.current_ap >= amount {
            self.current_ap -= amount;
            true
        } else {
            false
        }
    }

    pub fn add_experience(&mut self, xp: u32) {
        self.experience += xp;
    }

    #[allow(dead_code)]
    pub fn can_level_up(&self) -> bool {
        let new_level = 1 + (self.experience / 1000);
        new_level > self.level
    }

    #[allow(dead_code)]
    pub fn level_up(&mut self) {
        let new_level = 1 + (self.experience / 1000);
        if new_level > self.level {
            let levels_gained = new_level - self.level;
            for _ in 0..levels_gained {
                self.max_hp += 5 + self.special.endurance as i32;
            }
            self.level = new_level;
            self.current_hp = self.max_hp;
        }
    }

    pub fn get_equipped_damage(&self) -> String {
        if let Some(weapon_id) = &self.equipped_weapon {
            // Find the weapon in inventory
            if let Some(item) = self.inventory.iter().find(|i| &i.id == weapon_id) {
                // Extract damage if it's a weapon
                if let super::items::ItemType::Weapon(ref stats) = item.item_type {
                    return stats.damage.clone();
                }
            }
        }
        // Default unarmed damage
        "1d4".to_string()
    }

    /// Get the appropriate combat skill for the currently equipped weapon
    pub fn get_weapon_skill(&self) -> u8 {
        if let Some(weapon_id) = &self.equipped_weapon {
            if let Some(item) = self.inventory.iter().find(|i| &i.id == weapon_id) {
                if let super::items::ItemType::Weapon(ref stats) = item.item_type {
                    return match stats.weapon_type {
                        super::items::WeaponType::SmallGun => self.skills.small_guns,
                        super::items::WeaponType::BigGun => self.skills.big_guns,
                        super::items::WeaponType::EnergyWeapon => self.skills.energy_weapons,
                        super::items::WeaponType::MeleeWeapon => self.skills.melee_weapons,
                        super::items::WeaponType::Unarmed => self.skills.unarmed,
                    };
                }
            }
        }
        // Default to unarmed skill
        self.skills.unarmed
    }

    /// Find an item in inventory by its ID - helper for item lookups
    #[allow(dead_code)]
    pub fn find_item_by_id(&self, id: &str) -> Option<&Item> {
        self.inventory.iter().find(|item| item.id == id)
    }

    pub fn use_consumable(&mut self, item_id: &str) -> Result<String, String> {
        // Find the item
        let item_index = self
            .inventory
            .iter()
            .position(|i| i.id == item_id)
            .ok_or_else(|| "Item not found in inventory".to_string())?;

        // Clone the effect to avoid borrow issues
        let effect = if let super::items::ItemType::Consumable(ref effect) =
            self.inventory[item_index].item_type
        {
            effect.clone()
        } else {
            return Err("Item is not consumable".to_string());
        };

        // Apply the effect
        let message = match effect {
            super::items::ConsumableEffect::Healing(amount) => {
                self.heal(amount);
                format!("Healed {} HP", amount)
            }
            super::items::ConsumableEffect::RadAway(amount) => {
                format!("Removed {} rads", amount)
            }
            super::items::ConsumableEffect::StatBuff {
                stat,
                amount,
                duration,
            } => {
                format!("Gained +{} {} for {} rounds", amount, stat, duration)
            }
            super::items::ConsumableEffect::Addiction { effect } => {
                format!("Warning: {}", effect)
            }
        };

        // Decrease quantity
        if self.inventory[item_index].quantity > 1 {
            self.inventory[item_index].quantity -= 1;
        } else {
            self.inventory.remove(item_index);
        }

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_creation() {
        let special = Special::new();
        assert_eq!(special.strength, 1);
        assert_eq!(special.total_points(), 7); // 7 stats at 1 each
    }

    #[test]
    fn test_special_total_points() {
        let mut special = Special::new();
        special.strength = 5;
        special.perception = 6;
        special.endurance = 7;
        // Total: 5 + 6 + 7 + 1 + 1 + 1 + 1 = 22
        assert_eq!(special.total_points(), 22);
    }

    #[test]
    fn test_skills_from_special() {
        let mut special = Special::new();
        special.agility = 5;
        special.strength = 6;

        let skills = Skills::from_special(&special);
        // small_guns = 5 + (agility * 4) = 5 + (5 * 4) = 25
        assert_eq!(skills.small_guns, 25);
        // unarmed = 30 + ((agility + strength) * 2) = 30 + (11 * 2) = 52
        assert_eq!(skills.unarmed, 52);
    }

    #[test]
    fn test_character_creation() {
        let special = Special::new();
        let character = Character::new("Test Hero".to_string(), special.clone());

        assert_eq!(character.name, "Test Hero");
        assert_eq!(character.level, 1);
        assert_eq!(character.experience, 0);
        assert!(character.is_alive());
        assert_eq!(character.caps, 500);
        assert!(!character.inventory.is_empty()); // Should have starting items
    }

    #[test]
    fn test_character_hp_calculation() {
        let mut special = Special::new();
        special.strength = 5;
        special.endurance = 7;

        let character = Character::new("Test".to_string(), special);
        // max_hp = 15 + strength + (endurance * 2) = 15 + 5 + 14 = 34
        assert_eq!(character.max_hp, 34);
        assert_eq!(character.current_hp, 34);
    }

    #[test]
    fn test_character_take_damage() {
        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);
        let initial_hp = character.current_hp;

        character.take_damage(10);
        assert_eq!(character.current_hp, initial_hp - 10);

        character.take_damage(1000);
        assert_eq!(character.current_hp, 0);
        assert!(!character.is_alive());
    }

    #[test]
    fn test_character_heal() {
        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);
        let max_hp = character.max_hp;

        character.take_damage(20);
        let damaged_hp = character.current_hp;

        character.heal(10);
        assert_eq!(character.current_hp, damaged_hp + 10);

        character.heal(1000);
        assert_eq!(character.current_hp, max_hp); // Can't exceed max
    }

    #[test]
    fn test_character_ap_system() {
        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);
        let max_ap = character.max_ap;

        assert!(character.use_ap(2));
        assert_eq!(character.current_ap, max_ap - 2);

        character.use_ap(max_ap - 2);
        assert_eq!(character.current_ap, 0);

        assert!(!character.use_ap(1)); // Not enough AP

        character.restore_ap();
        assert_eq!(character.current_ap, max_ap);
    }

    #[test]
    fn test_character_leveling() {
        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);

        assert_eq!(character.level, 1);

        character.add_experience(500);
        assert!(!character.can_level_up());
        assert_eq!(character.level, 1); // Still level 1

        character.add_experience(500);
        assert!(character.can_level_up());
        character.level_up();
        assert_eq!(character.level, 2); // Now level 2
        assert!(character.max_hp > 0);
    }

    #[test]
    fn test_get_equipped_damage() {
        let special = Special::new();
        let character = Character::new("Test".to_string(), special);

        // Character starts with 10mm pistol equipped
        let damage = character.get_equipped_damage();
        assert_eq!(damage, "1d10+2");
    }

    #[test]
    fn test_find_item_by_id() {
        let special = Special::new();
        let character = Character::new("Test".to_string(), special);

        let item = character.find_item_by_id("10mm_pistol");
        assert!(item.is_some());
        assert_eq!(item.unwrap().name, "10mm Pistol");

        let item = character.find_item_by_id("nonexistent");
        assert!(item.is_none());
    }

    #[test]
    fn test_use_consumable_healing() {
        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);

        character.take_damage(20);
        let hp_before = character.current_hp;

        let result = character.use_consumable("stimpak");
        assert!(result.is_ok());
        assert!(character.current_hp > hp_before);
    }

    #[test]
    fn test_use_consumable_not_found() {
        let special = Special::new();
        let mut character = Character::new("Test".to_string(), special);

        let result = character.use_consumable("nonexistent_item");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_skill() {
        let special = Special::new();
        let character = Character::new("Test".to_string(), special);

        let skill = character.skills.get_skill("small_guns");
        assert!(skill > 0);

        let skill = character.skills.get_skill("nonexistent");
        assert_eq!(skill, 0);
    }

    #[test]
    fn test_get_weapon_skill() {
        let special = Special::new();
        let character = Character::new("Test".to_string(), special);

        // Character starts with 10mm pistol (SmallGun) equipped
        let skill = character.get_weapon_skill();
        assert_eq!(skill, character.skills.small_guns);

        // If we could equip baseball bat (MeleeWeapon), it would use melee_weapons
        // For now, just verify the default weapon works
        assert!(skill > 0, "Equipped weapon should have a skill value");
    }
}
