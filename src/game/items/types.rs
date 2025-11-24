//! # Item Types Module
//!
//! Core item structures that form the foundation of the inventory system.
//!
//! ## Item Structure
//!
//! All items in the game share a common `Item` structure with:
//! - Unique ID for lookups
//! - Display name and description
//! - Weight and value (in caps)
//! - Quantity for stackable items
//! - Type-specific data via `ItemType`
//!
//! ## Item Categories
//!
//! Items are categorized by their `ItemType`:
//! - **Weapon**: Combat items with damage stats
//! - **Armor**: Protective gear with DR/AC
//! - **Consumable**: Single-use items with effects
//! - **Misc**: General items without special mechanics

use serde::{Deserialize, Serialize};
use smartstring::alias::String as SmartString;

use super::armor::ArmorStats;
use super::consumables::ConsumableEffect;
use super::weapons::{DamageType, WeaponStats, WeaponType};

/// Categorization of items by their function.
///
/// Each variant contains type-specific statistics and behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ItemType {
    /// Combat weapon with damage statistics
    Weapon(WeaponStats),
    /// Protective armor with defense statistics
    Armor(ArmorStats),
    /// Single-use item with an effect
    Consumable(ConsumableEffect),
    /// General item without special mechanics
    #[default]
    Misc,
}

/// A game item that can be carried, traded, and used.
///
/// Items form the basis of the inventory system and can represent
/// weapons, armor, consumables, or miscellaneous objects.
///
/// # Example
///
/// ```
/// use fallout_dnd::game::items::{Item, DamageType, WeaponType};
///
/// let pistol = Item::new_weapon(
///     "10mm_pistol",
///     "10mm Pistol",
///     "A reliable sidearm.",
///     "1d10+2",
///     DamageType::Normal,
///     WeaponType::SmallGun,
///     4,
///     150,
/// );
///
/// assert_eq!(pistol.name, "10mm Pistol");
/// assert_eq!(pistol.value, 150);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier for lookups (snake_case)
    pub id: SmartString,
    /// Display name shown to the player
    pub name: SmartString,
    /// Flavor text description
    pub description: SmartString,
    /// Category and type-specific stats
    pub item_type: ItemType,
    /// Weight in pounds (affects carry capacity)
    pub weight: f32,
    /// Value in bottle caps
    pub value: u32,
    /// Stack count (for consumables and ammo)
    pub quantity: u32,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            id: SmartString::new(),
            name: SmartString::new(),
            description: SmartString::new(),
            item_type: ItemType::Misc,
            weight: 0.0,
            value: 0,
            quantity: 1,
        }
    }
}

impl Item {
    /// Create a new weapon item.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier (snake_case)
    /// * `name` - Display name
    /// * `description` - Flavor text
    /// * `damage` - Dice notation (e.g., "2d6+3")
    /// * `damage_type` - Type of damage inflicted
    /// * `weapon_type` - Category determining which skill is used
    /// * `ap_cost` - Action points required to attack
    /// * `value` - Worth in caps
    ///
    /// # Example
    ///
    /// ```
    /// use fallout_dnd::game::items::{Item, DamageType, WeaponType};
    ///
    /// let shotgun = Item::new_weapon(
    ///     "combat_shotgun",
    ///     "Combat Shotgun",
    ///     "Devastating at close range.",
    ///     "3d6",
    ///     DamageType::Normal,
    ///     WeaponType::SmallGun,
    ///     5,
    ///     350,
    /// );
    /// ```
    pub fn new_weapon(
        id: &str,
        name: &str,
        description: &str,
        damage: &str,
        damage_type: DamageType,
        weapon_type: WeaponType,
        ap_cost: i32,
        value: u32,
    ) -> Self {
        Item {
            id: SmartString::from(id),
            name: SmartString::from(name),
            description: SmartString::from(description),
            item_type: ItemType::Weapon(WeaponStats {
                damage: SmartString::from(damage),
                damage_type,
                weapon_type,
                ap_cost,
                ammo_type: None,
                range: 30,
                critical_multiplier: 2.0,
            }),
            weight: 3.0,
            value,
            quantity: 1,
        }
    }

    /// Create a new armor item.
    ///
    /// Armor Class is automatically calculated as: 5 + (DR / 2)
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier (snake_case)
    /// * `name` - Display name
    /// * `description` - Flavor text
    /// * `dr` - Damage Resistance value
    /// * `value` - Worth in caps
    ///
    /// # Example
    ///
    /// ```
    /// use fallout_dnd::game::items::Item;
    ///
    /// let armor = Item::new_armor(
    ///     "combat_armor",
    ///     "Combat Armor",
    ///     "Military-grade protection.",
    ///     15,
    ///     500,
    /// );
    /// ```
    pub fn new_armor(id: &str, name: &str, description: &str, dr: i32, value: u32) -> Self {
        Item {
            id: SmartString::from(id),
            name: SmartString::from(name),
            description: SmartString::from(description),
            item_type: ItemType::Armor(ArmorStats {
                damage_resistance: dr,
                radiation_resistance: 0,
                armor_class: 5 + (dr / 2),
            }),
            weight: 8.0,
            value,
            quantity: 1,
        }
    }

    /// Create a new consumable item.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier (snake_case)
    /// * `name` - Display name
    /// * `description` - Flavor text
    /// * `effect` - The effect when consumed
    /// * `value` - Worth in caps
    ///
    /// # Example
    ///
    /// ```
    /// use fallout_dnd::game::items::{Item, ConsumableEffect};
    ///
    /// let stimpak = Item::new_consumable(
    ///     "stimpak",
    ///     "Stimpak",
    ///     "Auto-injecting healing device.",
    ///     ConsumableEffect::Healing(30),
    ///     50,
    /// );
    /// ```
    pub fn new_consumable(
        id: &str,
        name: &str,
        description: &str,
        effect: ConsumableEffect,
        value: u32,
    ) -> Self {
        Item {
            id: SmartString::from(id),
            name: SmartString::from(name),
            description: SmartString::from(description),
            item_type: ItemType::Consumable(effect),
            weight: 0.5,
            value,
            quantity: 1,
        }
    }
}

// Test-only helper methods
#[cfg(test)]
impl Item {
    pub fn new_misc(id: &str, name: &str, description: &str, weight: f32, value: u32) -> Self {
        Item {
            id: SmartString::from(id),
            name: SmartString::from(name),
            description: SmartString::from(description),
            item_type: ItemType::Misc,
            weight,
            value,
            quantity: 1,
        }
    }

    pub fn is_weapon(&self) -> bool {
        matches!(self.item_type, ItemType::Weapon(_))
    }

    pub fn is_armor(&self) -> bool {
        matches!(self.item_type, ItemType::Armor(_))
    }

    pub fn is_consumable(&self) -> bool {
        matches!(self.item_type, ItemType::Consumable(_))
    }

    pub fn as_weapon(&self) -> Option<&WeaponStats> {
        match &self.item_type {
            ItemType::Weapon(stats) => Some(stats),
            _ => None,
        }
    }

    pub fn as_armor(&self) -> Option<&ArmorStats> {
        match &self.item_type {
            ItemType::Armor(stats) => Some(stats),
            _ => None,
        }
    }

    pub fn as_consumable(&self) -> Option<&ConsumableEffect> {
        match &self.item_type {
            ItemType::Consumable(effect) => Some(effect),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_creation() {
        let weapon = Item::new_weapon(
            "test_gun",
            "Test Gun",
            "A test weapon",
            "2d6+3",
            DamageType::Normal,
            WeaponType::SmallGun,
            5,
            100,
        );

        assert_eq!(weapon.id, "test_gun");
        assert_eq!(weapon.name, "Test Gun");
        assert_eq!(weapon.description, "A test weapon");
        assert_eq!(weapon.value, 100);
        assert_eq!(weapon.quantity, 1);

        match weapon.item_type {
            ItemType::Weapon(stats) => {
                assert_eq!(stats.damage, "2d6+3");
                assert_eq!(stats.damage_type, DamageType::Normal);
                assert_eq!(stats.weapon_type, WeaponType::SmallGun);
                assert_eq!(stats.ap_cost, 5);
                assert_eq!(stats.critical_multiplier, 2.0);
            }
            _ => panic!("Expected weapon item type"),
        }
    }

    #[test]
    fn test_armor_creation() {
        let armor = Item::new_armor("combat_armor", "Combat Armor", "Heavy protection", 15, 500);

        assert_eq!(armor.id, "combat_armor");
        assert_eq!(armor.name, "Combat Armor");
        assert_eq!(armor.value, 500);

        match armor.item_type {
            ItemType::Armor(stats) => {
                assert_eq!(stats.damage_resistance, 15);
                assert_eq!(stats.radiation_resistance, 0);
                // AC should be 5 + (DR / 2) = 5 + 7 = 12
                assert_eq!(stats.armor_class, 5 + (15 / 2));
            }
            _ => panic!("Expected armor item type"),
        }
    }

    #[test]
    fn test_consumable_healing_creation() {
        let stimpak = Item::new_consumable(
            "stimpak",
            "Stimpak",
            "Heals wounds",
            ConsumableEffect::Healing(30),
            50,
        );

        assert_eq!(stimpak.id, "stimpak");
        assert_eq!(stimpak.name, "Stimpak");
        assert_eq!(stimpak.weight, 0.5);

        match stimpak.item_type {
            ItemType::Consumable(ConsumableEffect::Healing(amount)) => {
                assert_eq!(amount, 30);
            }
            _ => panic!("Expected healing consumable"),
        }
    }

    #[test]
    fn test_consumable_radaway_creation() {
        let radaway = Item::new_consumable(
            "radaway",
            "RadAway",
            "Removes radiation",
            ConsumableEffect::RadAway(50),
            75,
        );

        match radaway.item_type {
            ItemType::Consumable(ConsumableEffect::RadAway(amount)) => {
                assert_eq!(amount, 50);
            }
            _ => panic!("Expected RadAway consumable"),
        }
    }

    #[test]
    fn test_consumable_stat_buff() {
        let buff = Item::new_consumable(
            "mentats",
            "Mentats",
            "Increases intelligence",
            ConsumableEffect::StatBuff {
                stat: "intelligence".to_string().into(),
                amount: 2,
                duration: 300,
            },
            100,
        );

        match buff.item_type {
            ItemType::Consumable(ConsumableEffect::StatBuff {
                stat,
                amount,
                duration,
            }) => {
                assert_eq!(stat, "intelligence");
                assert_eq!(amount, 2);
                assert_eq!(duration, 300);
            }
            _ => panic!("Expected stat buff consumable"),
        }
    }

    #[test]
    fn test_misc_item_creation() {
        let junk = Item::new_misc(
            "sensor_module",
            "Sensor Module",
            "A valuable component",
            1.0,
            25,
        );

        assert_eq!(junk.id, "sensor_module");
        assert_eq!(junk.name, "Sensor Module");
        assert_eq!(junk.weight, 1.0);
        assert_eq!(junk.value, 25);
        assert!(matches!(junk.item_type, ItemType::Misc));
    }

    #[test]
    fn test_item_default() {
        let item = Item::default();

        assert_eq!(item.id, "");
        assert_eq!(item.name, "");
        assert_eq!(item.weight, 0.0);
        assert_eq!(item.value, 0);
        assert_eq!(item.quantity, 1);
        assert_eq!(item.item_type, ItemType::Misc);
    }

    #[test]
    fn test_item_type_checks() {
        let weapon = Item::new_weapon(
            "gun",
            "Gun",
            "A gun",
            "1d6",
            DamageType::Normal,
            WeaponType::SmallGun,
            4,
            100,
        );
        let armor = Item::new_armor("armor", "Armor", "Protection", 10, 200);
        let consumable = Item::new_consumable(
            "stim",
            "Stimpak",
            "Heals",
            ConsumableEffect::Healing(30),
            50,
        );
        let misc = Item::new_misc("junk", "Junk", "Worthless", 0.5, 1);

        assert!(weapon.is_weapon());
        assert!(!weapon.is_armor());
        assert!(!weapon.is_consumable());

        assert!(!armor.is_weapon());
        assert!(armor.is_armor());
        assert!(!armor.is_consumable());

        assert!(!consumable.is_weapon());
        assert!(!consumable.is_armor());
        assert!(consumable.is_consumable());

        assert!(!misc.is_weapon());
        assert!(!misc.is_armor());
        assert!(!misc.is_consumable());
    }

    #[test]
    fn test_item_as_methods() {
        let weapon = Item::new_weapon(
            "gun",
            "Gun",
            "A gun",
            "1d6",
            DamageType::Normal,
            WeaponType::SmallGun,
            4,
            100,
        );

        assert!(weapon.as_weapon().is_some());
        assert!(weapon.as_armor().is_none());
        assert!(weapon.as_consumable().is_none());

        let stats = weapon.as_weapon().unwrap();
        assert_eq!(stats.damage, "1d6");
    }

    #[test]
    fn test_weapon_types() {
        let laser_gun = Item::new_weapon(
            "laser",
            "Laser Rifle",
            "Energy weapon",
            "2d10",
            DamageType::Laser,
            WeaponType::EnergyWeapon,
            6,
            400,
        );

        match laser_gun.item_type {
            ItemType::Weapon(stats) => {
                assert_eq!(stats.weapon_type, WeaponType::EnergyWeapon);
                assert_eq!(stats.damage_type, DamageType::Laser);
            }
            _ => panic!("Expected weapon"),
        }
    }

    #[test]
    fn test_armor_ac_calculation() {
        // AC = 5 + (DR / 2)
        let weak_armor = Item::new_armor("weak", "Weak", "Low protection", 2, 50);
        let strong_armor = Item::new_armor("strong", "Strong", "High protection", 20, 500);

        match weak_armor.item_type {
            ItemType::Armor(stats) => {
                assert_eq!(stats.armor_class, 5 + (2 / 2)); // 5 + 1 = 6
            }
            _ => panic!("Expected armor"),
        }

        match strong_armor.item_type {
            ItemType::Armor(stats) => {
                assert_eq!(stats.armor_class, 5 + (20 / 2)); // 5 + 10 = 15
            }
            _ => panic!("Expected armor"),
        }
    }

    #[test]
    fn test_weapon_ap_costs() {
        let pistol = Item::new_weapon(
            "pistol",
            "Pistol",
            "Fast",
            "1d6",
            DamageType::Normal,
            WeaponType::SmallGun,
            3,
            100,
        );

        let big_gun = Item::new_weapon(
            "minigun",
            "Minigun",
            "Heavy",
            "3d8",
            DamageType::Normal,
            WeaponType::BigGun,
            8,
            1000,
        );

        match pistol.item_type {
            ItemType::Weapon(stats) => assert_eq!(stats.ap_cost, 3),
            _ => panic!("Expected weapon"),
        }

        match big_gun.item_type {
            ItemType::Weapon(stats) => assert_eq!(stats.ap_cost, 8),
            _ => panic!("Expected weapon"),
        }
    }

    #[test]
    fn test_item_weight_values() {
        let weapon = Item::new_weapon(
            "gun",
            "Gun",
            "Desc",
            "1d6",
            DamageType::Normal,
            WeaponType::SmallGun,
            4,
            100,
        );

        let armor = Item::new_armor("armor", "Armor", "Desc", 10, 200);

        let consumable = Item::new_consumable(
            "stim",
            "Stimpak",
            "Heals",
            ConsumableEffect::Healing(30),
            50,
        );

        // Weapons should be heavier than consumables
        assert_eq!(weapon.weight, 3.0);
        assert_eq!(armor.weight, 8.0);
        assert_eq!(consumable.weight, 0.5);

        assert!(armor.weight > weapon.weight);
        assert!(weapon.weight > consumable.weight);
    }

    #[test]
    fn test_melee_weapon_creation() {
        let melee = Item::new_weapon(
            "sword",
            "Sword",
            "Sharp blade",
            "2d6+STR",
            DamageType::Normal,
            WeaponType::MeleeWeapon,
            3,
            150,
        );

        match melee.item_type {
            ItemType::Weapon(stats) => {
                assert_eq!(stats.weapon_type, WeaponType::MeleeWeapon);
                assert!(stats.damage.contains("STR"));
            }
            _ => panic!("Expected weapon"),
        }
    }

    #[test]
    fn test_critical_multiplier_default() {
        let weapon = Item::new_weapon(
            "test",
            "Test",
            "Desc",
            "1d6",
            DamageType::Normal,
            WeaponType::SmallGun,
            4,
            100,
        );

        match weapon.item_type {
            ItemType::Weapon(stats) => {
                assert_eq!(stats.critical_multiplier, 2.0);
            }
            _ => panic!("Expected weapon"),
        }
    }

    #[test]
    fn test_item_serialization() {
        let weapon = Item::new_weapon(
            "test",
            "Test",
            "Desc",
            "1d6",
            DamageType::Normal,
            WeaponType::SmallGun,
            4,
            100,
        );

        // Serialize to JSON
        let json = serde_json::to_string(&weapon).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("Test"));

        // Deserialize back
        let deserialized: Item = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, weapon.id);
        assert_eq!(deserialized.name, weapon.name);
    }
}
