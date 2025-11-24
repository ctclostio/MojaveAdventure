//! # Item Database Module
//!
//! Starting equipment for new characters.

use super::consumables::ConsumableEffect;
use super::types::Item;
use super::weapons::{DamageType, WeaponType};

/// Get the standard starting items for a new character.
///
/// Returns a collection of basic equipment suitable for
/// beginning the game in the wasteland.
///
/// # Contents
///
/// - **10mm Pistol**: Reliable sidearm (1d10+2 damage, 4 AP)
/// - **Baseball Bat**: Melee backup (1d8+STR damage, 3 AP)
/// - **Stimpak**: Emergency healing (30 HP)
/// - **RadAway**: Radiation treatment (50 rads)
/// - **Leather Armor**: Basic protection (DR 5)
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
    use crate::game::items::ItemType;

    // Test helper functions for creating items in tests
    fn combat_shotgun() -> Item {
        Item::new_weapon(
            "combat_shotgun",
            "Combat Shotgun",
            "A military-grade semi-automatic shotgun.",
            "3d6",
            DamageType::Normal,
            WeaponType::SmallGun,
            5,
            350,
        )
    }

    fn hunting_rifle() -> Item {
        Item::new_weapon(
            "hunting_rifle",
            "Hunting Rifle",
            "A reliable bolt-action rifle.",
            "2d8+3",
            DamageType::Normal,
            WeaponType::SmallGun,
            5,
            300,
        )
    }

    fn laser_pistol() -> Item {
        Item::new_weapon(
            "laser_pistol",
            "Laser Pistol",
            "A standard energy sidearm.",
            "1d10",
            DamageType::Laser,
            WeaponType::EnergyWeapon,
            4,
            200,
        )
    }

    fn laser_rifle() -> Item {
        Item::new_weapon(
            "laser_rifle",
            "Laser Rifle",
            "A military-grade energy weapon.",
            "2d10",
            DamageType::Laser,
            WeaponType::EnergyWeapon,
            6,
            500,
        )
    }

    fn plasma_rifle() -> Item {
        Item::new_weapon(
            "plasma_rifle",
            "Plasma Rifle",
            "Advanced energy weapon firing superheated plasma bolts.",
            "2d12",
            DamageType::Plasma,
            WeaponType::EnergyWeapon,
            6,
            750,
        )
    }

    fn combat_knife() -> Item {
        Item::new_weapon(
            "combat_knife",
            "Combat Knife",
            "A military fighting knife.",
            "1d6+STR",
            DamageType::Normal,
            WeaponType::MeleeWeapon,
            2,
            75,
        )
    }

    fn super_sledge() -> Item {
        Item::new_weapon(
            "super_sledge",
            "Super Sledge",
            "A rocket-assisted sledgehammer.",
            "2d10+STR",
            DamageType::Normal,
            WeaponType::MeleeWeapon,
            4,
            400,
        )
    }

    fn minigun() -> Item {
        Item::new_weapon(
            "minigun",
            "Minigun",
            "A motorized multi-barrel weapon.",
            "3d8+5",
            DamageType::Normal,
            WeaponType::BigGun,
            8,
            1500,
        )
    }

    fn metal_armor() -> Item {
        Item::new_armor(
            "metal_armor",
            "Metal Armor",
            "Armor cobbled together from scrap metal.",
            10,
            350,
        )
    }

    fn combat_armor() -> Item {
        Item::new_armor(
            "combat_armor",
            "Combat Armor",
            "Pre-war military armor.",
            15,
            500,
        )
    }

    fn power_armor_t51b() -> Item {
        Item::new_armor(
            "power_armor_t51b",
            "T-51b Power Armor",
            "Advanced pre-war power armor.",
            30,
            5000,
        )
    }

    fn super_stimpak() -> Item {
        Item::new_consumable(
            "super_stimpak",
            "Super Stimpak",
            "A more potent healing injection.",
            ConsumableEffect::Healing(60),
            100,
        )
    }

    fn buffout() -> Item {
        Item::new_consumable(
            "buffout",
            "Buffout",
            "Military-grade steroid.",
            ConsumableEffect::StatBuff {
                stat: "strength".into(),
                amount: 2,
                duration: 300,
            },
            100,
        )
    }

    fn mentats() -> Item {
        Item::new_consumable(
            "mentats",
            "Mentats",
            "Pre-war pharmaceutical.",
            ConsumableEffect::StatBuff {
                stat: "intelligence".into(),
                amount: 2,
                duration: 300,
            },
            100,
        )
    }

    fn jet() -> Item {
        Item::new_consumable(
            "jet",
            "Jet",
            "Highly addictive inhaler.",
            ConsumableEffect::StatBuff {
                stat: "agility".into(),
                amount: 2,
                duration: 100,
            },
            80,
        )
    }

    fn nuka_cola() -> Item {
        Item::new_consumable(
            "nuka_cola",
            "Nuka-Cola",
            "The iconic pre-war soft drink.",
            ConsumableEffect::Healing(10),
            20,
        )
    }

    #[test]
    fn test_starting_items_not_empty() {
        let items = get_starting_items();
        assert!(!items.is_empty(), "Starting items should not be empty");
    }

    #[test]
    fn test_starting_items_contains_expected_items() {
        let items = get_starting_items();

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
    fn test_combat_shotgun() {
        let shotgun = combat_shotgun();

        assert_eq!(shotgun.id, "combat_shotgun");
        assert_eq!(shotgun.name, "Combat Shotgun");

        if let ItemType::Weapon(stats) = shotgun.item_type {
            assert_eq!(stats.damage, "3d6");
            assert_eq!(stats.ap_cost, 5);
            assert_eq!(stats.weapon_type, WeaponType::SmallGun);
            assert_eq!(stats.damage_type, DamageType::Normal);
        } else {
            panic!("Expected weapon");
        }
    }

    #[test]
    fn test_hunting_rifle() {
        let rifle = hunting_rifle();

        assert_eq!(rifle.id, "hunting_rifle");
        if let ItemType::Weapon(stats) = rifle.item_type {
            assert_eq!(stats.damage, "2d8+3");
            assert_eq!(stats.weapon_type, WeaponType::SmallGun);
        } else {
            panic!("Expected weapon");
        }
    }

    #[test]
    fn test_energy_weapons() {
        let laser_p = laser_pistol();
        let laser_r = laser_rifle();
        let plasma = plasma_rifle();

        if let ItemType::Weapon(stats) = laser_p.item_type {
            assert_eq!(stats.damage_type, DamageType::Laser);
            assert_eq!(stats.weapon_type, WeaponType::EnergyWeapon);
        } else {
            panic!("Expected weapon");
        }

        if let ItemType::Weapon(stats) = laser_r.item_type {
            assert_eq!(stats.damage_type, DamageType::Laser);
        } else {
            panic!("Expected weapon");
        }

        if let ItemType::Weapon(stats) = plasma.item_type {
            assert_eq!(stats.damage_type, DamageType::Plasma);
        } else {
            panic!("Expected weapon");
        }
    }

    #[test]
    fn test_melee_weapons() {
        let knife = combat_knife();
        let sledge = super_sledge();

        if let ItemType::Weapon(stats) = knife.item_type {
            assert_eq!(stats.weapon_type, WeaponType::MeleeWeapon);
            assert!(stats.damage.contains("STR"));
            assert_eq!(stats.ap_cost, 2);
        } else {
            panic!("Expected weapon");
        }

        if let ItemType::Weapon(stats) = sledge.item_type {
            assert_eq!(stats.weapon_type, WeaponType::MeleeWeapon);
            assert!(stats.damage.contains("STR"));
            assert_eq!(stats.ap_cost, 4);
        } else {
            panic!("Expected weapon");
        }
    }

    #[test]
    fn test_minigun() {
        let gun = minigun();

        if let ItemType::Weapon(stats) = gun.item_type {
            assert_eq!(stats.weapon_type, WeaponType::BigGun);
            assert_eq!(stats.ap_cost, 8);
        } else {
            panic!("Expected weapon");
        }
    }

    #[test]
    fn test_armor_progression() {
        let metal = metal_armor();
        let combat = combat_armor();
        let power = power_armor_t51b();

        let metal_dr = match metal.item_type {
            ItemType::Armor(stats) => stats.damage_resistance,
            _ => panic!("Expected armor"),
        };

        let combat_dr = match combat.item_type {
            ItemType::Armor(stats) => stats.damage_resistance,
            _ => panic!("Expected armor"),
        };

        let power_dr = match power.item_type {
            ItemType::Armor(stats) => stats.damage_resistance,
            _ => panic!("Expected armor"),
        };

        assert!(combat_dr > metal_dr);
        assert!(power_dr > combat_dr);
    }

    #[test]
    fn test_consumables() {
        let super_stim = super_stimpak();
        let buff = buffout();
        let ment = mentats();
        let j = jet();
        let nuka = nuka_cola();

        if let ItemType::Consumable(ConsumableEffect::Healing(amount)) = super_stim.item_type {
            assert_eq!(amount, 60);
        } else {
            panic!("Expected healing consumable");
        }

        assert!(matches!(
            buff.item_type,
            ItemType::Consumable(ConsumableEffect::StatBuff { .. })
        ));
        assert!(matches!(
            ment.item_type,
            ItemType::Consumable(ConsumableEffect::StatBuff { .. })
        ));
        assert!(matches!(
            j.item_type,
            ItemType::Consumable(ConsumableEffect::StatBuff { .. })
        ));

        if let ItemType::Consumable(ConsumableEffect::Healing(amount)) = nuka.item_type {
            assert_eq!(amount, 10);
        } else {
            panic!("Expected healing consumable");
        }
    }

    #[test]
    fn test_unique_item_ids() {
        let items = vec![
            combat_shotgun(),
            hunting_rifle(),
            laser_pistol(),
            laser_rifle(),
            plasma_rifle(),
            combat_knife(),
            super_sledge(),
            minigun(),
            metal_armor(),
            combat_armor(),
            power_armor_t51b(),
            super_stimpak(),
            buffout(),
            mentats(),
            jet(),
            nuka_cola(),
        ];

        let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();

        assert_eq!(ids.len(), original_len, "All item IDs should be unique");
    }
}
