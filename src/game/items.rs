use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DamageType {
    Normal,
    Laser,
    Plasma,
    Fire,
    Explosive,
    Poison,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Weapon(WeaponStats),
    Armor(ArmorStats),
    Consumable(ConsumableEffect),
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
    pub damage: String, // e.g., "1d10+2"
    pub damage_type: DamageType,
    pub weapon_type: WeaponType,
    pub ap_cost: i32,
    pub ammo_type: Option<String>,
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
        stat: String,
        amount: i32,
        duration: u32,
    },
    Addiction {
        effect: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub weight: f32,
    pub value: u32,
    pub quantity: u32,
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
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            item_type: ItemType::Weapon(WeaponStats {
                damage: damage.to_string(),
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
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
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
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
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
