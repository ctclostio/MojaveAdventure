/// Comprehensive tests for the item system
mod helpers;

use fallout_dnd::game::{
    character::{Character, Special},
    items::{ConsumableEffect, DamageType, Item, ItemType, WeaponType},
};
use helpers::*;

#[test]
fn test_create_weapon_item() {
    let weapon = Item::new_weapon(
        "test_pistol",
        "Test Pistol",
        "A test weapon",
        "1d10+2",
        DamageType::Normal,
        WeaponType::SmallGun,
        3,
        100,
    );

    assert_eq!(weapon.id, "test_pistol");
    assert_eq!(weapon.name, "Test Pistol");
    assert_eq!(weapon.value, 100);

    if let ItemType::Weapon(stats) = weapon.item_type {
        assert_eq!(stats.damage, "1d10+2");
        assert_eq!(stats.ap_cost, 3);
        assert_eq!(stats.weapon_type, WeaponType::SmallGun);
        assert_eq!(stats.damage_type, DamageType::Normal);
    } else {
        panic!("Expected weapon item type");
    }
}

#[test]
fn test_create_armor_item() {
    let armor = Item::new_armor("test_armor", "Test Armor", "A test armor", 10, 200);

    assert_eq!(armor.id, "test_armor");
    assert_eq!(armor.value, 200);

    if let ItemType::Armor(stats) = armor.item_type {
        assert_eq!(stats.damage_resistance, 10);
        assert_eq!(stats.armor_class, 10); // 5 + (10/2)
    } else {
        panic!("Expected armor item type");
    }
}

#[test]
fn test_create_consumable_item() {
    let stimpak = Item {
        id: "stimpak".into(),
        name: "Stimpak".into(),
        description: "Heals HP".into(),
        item_type: ItemType::Consumable(ConsumableEffect::Healing(30)),
        weight: 0.5,
        value: 25,
        quantity: 1,
    };

    if let ItemType::Consumable(ConsumableEffect::Healing(amount)) = stimpak.item_type {
        assert_eq!(amount, 30);
    } else {
        panic!("Expected healing consumable");
    }
}

#[test]
fn test_consumable_healing_effect() {
    let mut character = create_test_character("Test");
    character.current_hp = 50;
    character.max_hp = 100;

    let result = character.use_consumable("stimpak");

    assert!(result.is_ok(), "Using stimpak should succeed");
    assert!(
        character.current_hp > 50,
        "HP should increase after using stimpak"
    );
}

#[test]
fn test_consumable_not_in_inventory() {
    let mut character = create_test_character("Test");

    let result = character.use_consumable("nonexistent_item");

    assert!(result.is_err(), "Using nonexistent item should fail");
    assert_eq!(result.unwrap_err(), "Item not found in inventory");
}

#[test]
fn test_consumable_wrong_type() {
    let mut character = create_test_character("Test");

    // Try to use a weapon (10mm_pistol is in starting items) as a consumable
    let result = character.use_consumable("10mm_pistol");

    assert!(result.is_err(), "Using non-consumable should fail");
    assert_eq!(result.unwrap_err(), "Item is not consumable");
}

#[test]
fn test_healing_doesnt_exceed_max_hp() {
    let mut character = create_test_character("Test");
    character.current_hp = character.max_hp - 5; // Almost full HP
    let initial_max = character.max_hp;

    // Use stimpak (heals 30 HP from starting items)
    let _ = character.use_consumable("stimpak");

    assert_eq!(
        character.current_hp, character.max_hp,
        "HP should not exceed max"
    );
    assert_eq!(character.max_hp, initial_max, "Max HP should not change");
}

#[test]
fn test_get_equipped_weapon_damage() {
    let character = create_test_character("Test");

    // Starting character has 10mm_pistol equipped
    let damage = character.get_equipped_damage();

    // Should return the pistol's damage, not default unarmed
    assert_ne!(damage, "1d4", "Should not be unarmed damage");
    assert!(damage.contains("d"), "Should be a dice expression");
}

#[test]
fn test_get_equipped_weapon_skill() {
    let character = create_test_character("Test");

    // Starting character has 10mm_pistol (small gun) equipped
    let skill = character.get_weapon_skill();

    // Should return small guns skill
    assert_eq!(skill, character.skills.small_guns);
}

#[test]
fn test_unequipped_returns_unarmed() {
    let mut character = create_test_character("Test");
    character.equipped_weapon = None;

    let damage = character.get_equipped_damage();
    let skill = character.get_weapon_skill();

    assert_eq!(damage, "1d4", "Unarmed should do 1d4 damage");
    assert_eq!(skill, character.skills.unarmed);
}

#[test]
fn test_weapon_types_map_to_correct_skills() {
    let special = Special::new();
    let mut character = Character::new("Test", special);

    // Test different weapon types
    let small_gun = Item::new_weapon(
        "pistol",
        "Pistol",
        "Test",
        "1d8",
        DamageType::Normal,
        WeaponType::SmallGun,
        3,
        100,
    );

    let energy_weapon = Item::new_weapon(
        "laser",
        "Laser",
        "Test",
        "2d6",
        DamageType::Laser,
        WeaponType::EnergyWeapon,
        4,
        200,
    );

    let melee = Item::new_weapon(
        "sword",
        "Sword",
        "Test",
        "1d10",
        DamageType::Normal,
        WeaponType::MeleeWeapon,
        2,
        50,
    );

    // Add weapons to inventory
    character.inventory.push(small_gun);
    character.inventory.push(energy_weapon);
    character.inventory.push(melee);

    // Test small gun
    character.equipped_weapon = Some("pistol".into());
    assert_eq!(character.get_weapon_skill(), character.skills.small_guns);

    // Test energy weapon
    character.equipped_weapon = Some("laser".into());
    assert_eq!(
        character.get_weapon_skill(),
        character.skills.energy_weapons
    );

    // Test melee
    character.equipped_weapon = Some("sword".into());
    assert_eq!(character.get_weapon_skill(), character.skills.melee_weapons);
}

#[test]
fn test_find_item_by_id() {
    let character = create_test_character("Test");

    // Should find starting item
    let item = character.find_item_by_id("10mm_pistol");
    assert!(
        item.is_some(),
        "Should find 10mm pistol in starting inventory"
    );

    // Should not find nonexistent item
    let item = character.find_item_by_id("nonexistent");
    assert!(item.is_none(), "Should not find nonexistent item");
}

#[test]
fn test_radaway_consumable_effect() {
    let radaway = Item {
        id: "radaway".into(),
        name: "RadAway".into(),
        description: "Removes radiation".into(),
        item_type: ItemType::Consumable(ConsumableEffect::RadAway(20)),
        weight: 0.5,
        value: 50,
        quantity: 1,
    };

    if let ItemType::Consumable(ConsumableEffect::RadAway(amount)) = radaway.item_type {
        assert_eq!(amount, 20);
    } else {
        panic!("Expected RadAway consumable");
    }
}

#[test]
fn test_stat_buff_consumable_effect() {
    let buff = Item {
        id: "buffout".into(),
        name: "Buffout".into(),
        description: "Increases strength".into(),
        item_type: ItemType::Consumable(ConsumableEffect::StatBuff {
            stat: "strength".into(),
            amount: 2,
            duration: 300,
        }),
        weight: 0.1,
        value: 75,
        quantity: 1,
    };

    if let ItemType::Consumable(ConsumableEffect::StatBuff {
        stat,
        amount,
        duration,
    }) = buff.item_type
    {
        assert_eq!(stat, "strength");
        assert_eq!(amount, 2);
        assert_eq!(duration, 300);
    } else {
        panic!("Expected stat buff consumable");
    }
}

#[test]
fn test_different_damage_types() {
    let laser = DamageType::Laser;
    let plasma = DamageType::Plasma;
    let fire = DamageType::Fire;
    let explosive = DamageType::Explosive;
    let poison = DamageType::Poison;
    let normal = DamageType::Normal;

    // Just verify all damage types exist and are different
    assert_ne!(laser, plasma);
    assert_ne!(fire, explosive);
    assert_ne!(poison, normal);
}

#[test]
fn test_item_weight_and_value() {
    let weapon = create_test_weapon("test", "1d6");

    assert!(weapon.weight > 0.0, "Weapon should have weight");
    assert!(weapon.value > 0, "Weapon should have value");
}

#[test]
fn test_item_quantity() {
    let mut item = create_healing_item("test_stim", 20);
    assert_eq!(item.quantity, 1);

    item.quantity = 5;
    assert_eq!(item.quantity, 5);
}

#[test]
fn test_armor_class_calculation() {
    // AC = 5 + (DR / 2)
    let armor1 = Item::new_armor("armor1", "Test", "Test", 10, 100);
    let armor2 = Item::new_armor("armor2", "Test", "Test", 20, 200);

    if let ItemType::Armor(stats1) = armor1.item_type {
        assert_eq!(stats1.armor_class, 10); // 5 + (10/2)
    }

    if let ItemType::Armor(stats2) = armor2.item_type {
        assert_eq!(stats2.armor_class, 15); // 5 + (20/2)
    }
}

#[test]
fn test_weapon_critical_multiplier() {
    let weapon = Item::new_weapon(
        "test",
        "Test",
        "Test",
        "1d6",
        DamageType::Normal,
        WeaponType::SmallGun,
        3,
        100,
    );

    if let ItemType::Weapon(stats) = weapon.item_type {
        assert_eq!(stats.critical_multiplier, 2.0);
    }
}
