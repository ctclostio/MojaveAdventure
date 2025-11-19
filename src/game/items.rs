use serde::{Deserialize, Serialize};
use smartstring::alias::String as SmartString;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DamageType {
    Normal,
    Laser,
    Plasma,
    Fire,
    Explosive,
    Poison,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ItemType {
    Weapon(WeaponStats),
    Armor(ArmorStats),
    Consumable(ConsumableEffect),
    #[default]
    Misc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WeaponType {
    SmallGun,     // Pistols, SMGs
    BigGun,       // Miniguns, rocket launchers
    EnergyWeapon, // Laser, plasma
    MeleeWeapon,  // Swords, clubs
    Unarmed,      // Fists, brass knuckles
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeaponStats {
    pub damage: SmartString, // e.g., "1d10+2"
    pub damage_type: DamageType,
    pub weapon_type: WeaponType,
    pub ap_cost: i32,
    pub ammo_type: Option<SmartString>,
    pub range: u32, // in meters
    pub critical_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArmorStats {
    pub damage_resistance: i32,
    pub radiation_resistance: i32,
    pub armor_class: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsumableEffect {
    Healing(i32),
    RadAway(i32),
    StatBuff {
        stat: SmartString,
        amount: i32,
        duration: u32,
    },
    Addiction {
        effect: SmartString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: SmartString,
    pub name: SmartString,
    pub description: SmartString,
    pub item_type: ItemType,
    pub weight: f32,
    pub value: u32,
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

// Predefined item database
pub fn get_starting_items() -> Vec<Item> {
    vec![
        Item::new_weapon(
            "10mm_pistol",
            "10mm Pistol",
            "A common semi-automatic pistol. Reliable and easy to maintain.",
            "1d10+2",
            DamageType::Normal,
            WeaponType::SmallGun,
            4,
            150,
        ),
        Item::new_weapon(
            "baseball_bat",
            "Baseball Bat",
            "Pre-war sporting equipment, now a popular melee weapon.",
            "1d8+STR",
            DamageType::Normal,
            WeaponType::MeleeWeapon,
            3,
            50,
        ),
        Item::new_consumable(
            "stimpak",
            "Stimpak",
            "Heals 30 HP instantly.",
            ConsumableEffect::Healing(30),
            50,
        ),
        Item::new_consumable(
            "radaway",
            "RadAway",
            "Removes 50 rads.",
            ConsumableEffect::RadAway(50),
            75,
        ),
        Item::new_armor(
            "leather_armor",
            "Leather Armor",
            "Basic protection made from brahmin hide.",
            5,
            200,
        ),
    ]
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
    fn test_damage_types() {
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
    fn test_starting_items_contains_expected_items() {
        let items = get_starting_items();

        assert!(!items.is_empty(), "Starting items should not be empty");

        // Check for specific starting items
        let has_pistol = items.iter().any(|i| i.id == "10mm_pistol");
        let has_stimpak = items.iter().any(|i| i.id == "stimpak");
        let has_armor = items.iter().any(|i| i.id == "leather_armor");

        assert!(has_pistol, "Should have 10mm pistol");
        assert!(has_stimpak, "Should have stimpak");
        assert!(has_armor, "Should have leather armor");
    }

    #[test]
    fn test_starting_items_variety() {
        let items = get_starting_items();

        let mut has_weapon = false;
        let mut has_armor = false;
        let mut has_consumable = false;

        for item in &items {
            match item.item_type {
                ItemType::Weapon(_) => has_weapon = true,
                ItemType::Armor(_) => has_armor = true,
                ItemType::Consumable(_) => has_consumable = true,
                _ => {}
            }
        }

        assert!(has_weapon, "Starting items should include weapons");
        assert!(has_armor, "Starting items should include armor");
        assert!(has_consumable, "Starting items should include consumables");
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
}
