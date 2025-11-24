//! # Weapons Module
//!
//! Weapon-specific types and stats for the Fallout RPG system.
//!
//! ## Weapon Types
//!
//! Weapons are categorized by their governing skill:
//! - **SmallGun**: Pistols, rifles, SMGs, shotguns
//! - **BigGun**: Miniguns, rocket launchers, flamers
//! - **EnergyWeapon**: Laser and plasma weapons
//! - **MeleeWeapon**: Swords, clubs, knives
//! - **Unarmed**: Fists, brass knuckles, power fists
//!
//! ## Damage Types
//!
//! Different damage types may have different interactions with armor:
//! - **Normal**: Standard ballistic/physical damage
//! - **Laser**: Energy damage from laser weapons
//! - **Plasma**: High-energy plasma damage
//! - **Fire**: Incendiary damage
//! - **Explosive**: Area-of-effect blast damage
//! - **Poison**: Damage over time

use serde::{Deserialize, Serialize};
use smartstring::alias::String as SmartString;

/// Types of damage that weapons can inflict.
///
/// Different damage types may interact differently with armor and resistances.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DamageType {
    /// Standard ballistic or physical damage
    Normal,
    /// Energy damage from laser weapons
    Laser,
    /// High-energy plasma damage
    Plasma,
    /// Incendiary/fire damage
    Fire,
    /// Explosive blast damage
    Explosive,
    /// Toxic/poison damage
    Poison,
}

/// Categories of weapons based on the skill used to wield them.
///
/// Each weapon type corresponds to a character skill that determines
/// hit chance and effectiveness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WeaponType {
    /// Pistols, rifles, SMGs, shotguns - uses Small Guns skill
    SmallGun,
    /// Miniguns, rocket launchers, flamers - uses Big Guns skill
    BigGun,
    /// Laser and plasma weapons - uses Energy Weapons skill
    EnergyWeapon,
    /// Swords, clubs, knives - uses Melee Weapons skill
    MeleeWeapon,
    /// Fists, brass knuckles, power fists - uses Unarmed skill
    Unarmed,
}

/// Combat statistics for a weapon.
///
/// Contains all the mechanical properties that affect combat calculations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeaponStats {
    /// Damage dice notation (e.g., "2d6+3" or "1d8+STR" for melee)
    pub damage: SmartString,
    /// Type of damage inflicted
    pub damage_type: DamageType,
    /// Category determining which skill is used
    pub weapon_type: WeaponType,
    /// Action points required to attack
    pub ap_cost: i32,
    /// Type of ammunition used (None for melee/unarmed)
    pub ammo_type: Option<SmartString>,
    /// Effective range in meters
    pub range: u32,
    /// Damage multiplier on critical hits
    pub critical_multiplier: f32,
}

impl Default for WeaponStats {
    fn default() -> Self {
        Self {
            damage: SmartString::from("1d4"),
            damage_type: DamageType::Normal,
            weapon_type: WeaponType::Unarmed,
            ap_cost: 3,
            ammo_type: None,
            range: 1,
            critical_multiplier: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_types_equality() {
        let types = vec![
            DamageType::Normal,
            DamageType::Laser,
            DamageType::Plasma,
            DamageType::Fire,
            DamageType::Explosive,
            DamageType::Poison,
        ];

        // Test that all damage types can be created and compared
        for dtype in &types {
            assert_eq!(dtype, dtype);
        }

        // Test inequality
        assert_ne!(&DamageType::Normal, &DamageType::Laser);
        assert_ne!(&DamageType::Fire, &DamageType::Poison);
    }

    #[test]
    fn test_weapon_types_equality() {
        assert_eq!(WeaponType::SmallGun, WeaponType::SmallGun);
        assert_ne!(WeaponType::SmallGun, WeaponType::BigGun);
        assert_ne!(WeaponType::MeleeWeapon, WeaponType::Unarmed);
    }

    #[test]
    fn test_weapon_stats_default() {
        let stats = WeaponStats::default();
        assert_eq!(stats.damage, "1d4");
        assert_eq!(stats.damage_type, DamageType::Normal);
        assert_eq!(stats.weapon_type, WeaponType::Unarmed);
        assert_eq!(stats.ap_cost, 3);
        assert!(stats.ammo_type.is_none());
        assert_eq!(stats.range, 1);
        assert_eq!(stats.critical_multiplier, 2.0);
    }

    #[test]
    fn test_weapon_stats_serialization() {
        let stats = WeaponStats {
            damage: SmartString::from("2d6+3"),
            damage_type: DamageType::Normal,
            weapon_type: WeaponType::SmallGun,
            ap_cost: 5,
            ammo_type: Some(SmartString::from("12_gauge")),
            range: 20,
            critical_multiplier: 2.0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("2d6+3"));
        assert!(json.contains("12_gauge"));

        let deserialized: WeaponStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, stats);
    }
}
