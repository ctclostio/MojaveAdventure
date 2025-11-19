/// Common test helpers and fixtures for integration tests
use fallout_dnd::game::{
    character::{Character, Special},
    combat::Enemy,
    items::{ConsumableEffect, DamageType, Item, ItemType, WeaponType},
    GameState,
};

/// Create a test character with default stats
pub fn create_test_character(name: &str) -> Character {
    let special = Special {
        strength: 5,
        perception: 5,
        endurance: 5,
        charisma: 5,
        intelligence: 5,
        agility: 5,
        luck: 5,
    };
    Character::new(name.to_string(), special)
}

/// Create a test character with custom SPECIAL stats
pub fn create_custom_character(name: &str, special: Special) -> Character {
    Character::new(name.to_string(), special)
}

/// Create a test game state with a default character
pub fn create_test_game_state() -> GameState {
    let character = create_test_character("Test Hero");
    GameState::new(character)
}

/// Create a high-stat test character for reliable skill checks
pub fn create_high_stat_character(name: &str) -> Character {
    let special = Special {
        strength: 10,
        perception: 10,
        endurance: 10,
        charisma: 10,
        intelligence: 10,
        agility: 10,
        luck: 10,
    };
    Character::new(name.to_string(), special)
}

/// Create a test healing item
pub fn create_healing_item(id: &str, heal_amount: i32) -> Item {
    Item {
        id: id.to_string(),
        name: format!("Test Stimpak {}", id),
        description: "A test healing item".to_string(),
        item_type: ItemType::Consumable(ConsumableEffect::Healing(heal_amount)),
        weight: 0.5,
        value: 25,
        quantity: 1,
    }
}

/// Create a test weapon
pub fn create_test_weapon(id: &str, damage: &str) -> Item {
    Item::new_weapon(
        id,
        "Test Weapon",
        "A test weapon",
        damage,
        DamageType::Normal,
        WeaponType::SmallGun,
        3,
        100,
    )
}

/// Create a test armor
pub fn create_test_armor(id: &str, dr: i32) -> Item {
    Item::new_armor(id, "Test Armor", "A test armor", dr, 200)
}

/// Create a test enemy with specific stats
pub fn create_test_enemy(name: &str, hp: i32, skill: u8) -> Enemy {
    Enemy {
        name: name.to_string(),
        level: 1,
        max_hp: hp,
        current_hp: hp,
        armor_class: 10,
        skill,
        damage: "1d6".to_string(),
        ap: 5,
        xp_reward: 50,
        strength: 5,
    }
}

/// Create a weak enemy for testing combat
pub fn create_weak_enemy() -> Enemy {
    create_test_enemy("Weak Radroach", 5, 5)
}

/// Create a strong enemy for testing combat
pub fn create_strong_enemy() -> Enemy {
    create_test_enemy("Deathclaw", 100, 15)
}
