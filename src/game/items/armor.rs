//! # Armor Module
//!
//! Armor statistics and protection mechanics for the Fallout RPG system.
//!
//! ## Armor Class Calculation
//!
//! Armor Class (AC) is derived from Damage Resistance:
//! ```text
//! AC = 5 + (DR / 2)
//! ```
//!
//! ## Protection Tiers
//!
//! | Tier | DR | AC | Examples |
//! |------|-----|-----|----------|
//! | Light | 2-5 | 6-7 | Leather Armor, Vault Suit |
//! | Medium | 8-12 | 9-11 | Metal Armor, Combat Leather |
//! | Heavy | 15-20 | 12-15 | Combat Armor, Riot Gear |
//! | Power | 25-35 | 17-22 | T-45d, T-51b Power Armor |

use serde::{Deserialize, Serialize};

/// Protective statistics for armor items.
///
/// Armor provides damage reduction and may offer radiation protection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArmorStats {
    /// Damage Resistance - reduces incoming damage
    pub damage_resistance: i32,
    /// Radiation Resistance - reduces rad accumulation
    pub radiation_resistance: i32,
    /// Armor Class - makes the wearer harder to hit
    pub armor_class: i32,
}

// Test-only constructors
#[cfg(test)]
impl ArmorStats {
    pub fn new(damage_resistance: i32, radiation_resistance: i32) -> Self {
        Self {
            damage_resistance,
            radiation_resistance,
            armor_class: 5 + (damage_resistance / 2),
        }
    }

    pub fn with_dr(damage_resistance: i32) -> Self {
        Self::new(damage_resistance, 0)
    }
}

impl Default for ArmorStats {
    fn default() -> Self {
        Self {
            damage_resistance: 0,
            radiation_resistance: 0,
            armor_class: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_armor_stats_new() {
        let armor = ArmorStats::new(15, 10);
        assert_eq!(armor.damage_resistance, 15);
        assert_eq!(armor.radiation_resistance, 10);
        assert_eq!(armor.armor_class, 12); // 5 + (15 / 2) = 12
    }

    #[test]
    fn test_armor_stats_with_dr() {
        let armor = ArmorStats::with_dr(10);
        assert_eq!(armor.damage_resistance, 10);
        assert_eq!(armor.radiation_resistance, 0);
        assert_eq!(armor.armor_class, 10); // 5 + (10 / 2) = 10
    }

    #[test]
    fn test_armor_stats_default() {
        let armor = ArmorStats::default();
        assert_eq!(armor.damage_resistance, 0);
        assert_eq!(armor.radiation_resistance, 0);
        assert_eq!(armor.armor_class, 5);
    }

    #[test]
    fn test_armor_ac_calculation() {
        // AC = 5 + (DR / 2)
        let weak_armor = ArmorStats::with_dr(2);
        assert_eq!(weak_armor.armor_class, 5 + (2 / 2)); // 6

        let medium_armor = ArmorStats::with_dr(10);
        assert_eq!(medium_armor.armor_class, 5 + (10 / 2)); // 10

        let strong_armor = ArmorStats::with_dr(20);
        assert_eq!(strong_armor.armor_class, 5 + (20 / 2)); // 15

        let power_armor = ArmorStats::with_dr(30);
        assert_eq!(power_armor.armor_class, 5 + (30 / 2)); // 20
    }

    #[test]
    fn test_armor_stats_serialization() {
        let armor = ArmorStats::new(15, 10);

        let json = serde_json::to_string(&armor).unwrap();
        assert!(json.contains("15"));
        assert!(json.contains("10"));

        let deserialized: ArmorStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, armor);
    }

    #[test]
    fn test_armor_stats_equality() {
        let armor1 = ArmorStats::new(15, 10);
        let armor2 = ArmorStats::new(15, 10);
        let armor3 = ArmorStats::new(20, 10);

        assert_eq!(armor1, armor2);
        assert_ne!(armor1, armor3);
    }
}
