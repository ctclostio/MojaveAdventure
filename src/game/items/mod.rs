//! # Items Module
//!
//! Item system and inventory management for the Fallout RPG.
//!
//! ## Module Structure
//!
//! - [`weapons`]: Weapon types, damage types, and combat stats
//! - [`armor`]: Armor statistics and protection mechanics
//! - [`consumables`]: Consumable effects (healing, chems, etc.)
//! - [`types`]: Core `Item` and `ItemType` structures
//! - [`database`]: Starting equipment
//!
//! ## Creating Items
//!
//! Use the `Item` constructor methods:
//!
//! ```
//! use fallout_dnd::game::items::{Item, DamageType, WeaponType, ConsumableEffect};
//!
//! // Create a weapon
//! let pistol = Item::new_weapon(
//!     "10mm_pistol",
//!     "10mm Pistol",
//!     "A reliable sidearm.",
//!     "1d10+2",
//!     DamageType::Normal,
//!     WeaponType::SmallGun,
//!     4,
//!     150,
//! );
//!
//! // Create armor
//! let armor = Item::new_armor(
//!     "leather_armor",
//!     "Leather Armor",
//!     "Basic protection.",
//!     5,
//!     200,
//! );
//!
//! // Create a consumable
//! let stimpak = Item::new_consumable(
//!     "stimpak",
//!     "Stimpak",
//!     "Heals 30 HP.",
//!     ConsumableEffect::Healing(30),
//!     50,
//! );
//! ```

pub mod armor;
pub mod consumables;
pub mod database;
pub mod types;
pub mod weapons;

// Re-export public types
pub use consumables::ConsumableEffect;
pub use database::get_starting_items;
pub use types::{Item, ItemType};
#[allow(unused_imports)] // Re-exported for public API; used by tests and external consumers
pub use weapons::{DamageType, WeaponType};
