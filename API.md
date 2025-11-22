# Public API Documentation

## Table of Contents
- [Overview](#overview)
- [Core Types](#core-types)
- [Game Module](#game-module)
- [AI Module](#ai-module)
- [TUI Module](#tui-module)
- [Configuration](#configuration)
- [Error Handling](#error-handling)
- [Usage Examples](#usage-examples)

---

## Overview

This document describes the public API for the Fallout D&D project. The API is organized into three main modules:

1. **`game`** - Core game logic (character, combat, worldbook)
2. **`ai`** - AI dungeon master integration
3. **`tui`** - Terminal user interface

Most types are serializable (implement `Serialize`/`Deserialize`) for save/load functionality.

---

## Core Types

### GameState

**Location:** `src/game/mod.rs`

The central hub for all game data.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub character: Character,
    pub combat: CombatState,
    pub conversation: ConversationManager,
    pub story: StoryManager,  // Legacy, use conversation instead
    pub location: String,
    pub quest_log: Vec<String>,
    pub worldbook: Worldbook,
    pub day: u32,
}
```

#### Methods

```rust
impl GameState {
    /// Create a new game with the given character
    pub fn new(character: Character) -> Self;

    /// Migrate legacy story context to conversation system
    pub fn migrate_story_to_conversation(&mut self);
}
```

#### Example

```rust
use fallout_dnd::game::{GameState, character::{Character, Special}};

let special = Special::new();
let character = Character::new("Vault Dweller".to_string(), special);
let mut game = GameState::new(character);

// Add conversation turns
game.conversation.add_player_turn("I explore the wasteland".to_string());
game.conversation.add_dm_turn("You see ruins ahead".to_string());
```

---

## Game Module

### Character

**Location:** `src/game/character.rs`

Represents the player character with SPECIAL stats, skills, and inventory.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: SmartString,
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
    pub equipped_weapon: Option<SmartString>,
    pub equipped_armor: Option<SmartString>,
    pub traits: Vec<SmartString>,
    pub perks: Vec<SmartString>,
}
```

#### Methods

```rust
impl Character {
    /// Create a new character with given name and SPECIAL stats
    pub fn new(name: impl Into<SmartString>, special: Special) -> Self;

    /// Apply damage to the character
    pub fn take_damage(&mut self, damage: i32);

    /// Heal the character (up to max HP)
    pub fn heal(&mut self, amount: i32);

    /// Check if character is alive
    pub fn is_alive(&self) -> bool;

    /// Restore action points to maximum
    pub fn restore_ap(&mut self);

    /// Use action points (returns true if enough AP available)
    pub fn use_ap(&mut self, amount: i32) -> bool;

    /// Add experience points
    pub fn add_experience(&mut self, xp: u32);

    /// Check if character can level up
    pub fn can_level_up(&self) -> bool;

    /// Level up the character (increases max HP, level)
    pub fn level_up(&mut self);

    /// Get damage string for equipped weapon
    pub fn get_equipped_damage(&self) -> SmartString;

    /// Get combat skill for currently equipped weapon
    pub fn get_weapon_skill(&self) -> u8;
}
```

#### Example

```rust
use fallout_dnd::game::character::{Character, Special};

let mut special = Special::new();
special.strength = 6;
special.endurance = 7;
special.agility = 8;

let mut character = Character::new("Lone Wanderer", special);
println!("HP: {}/{}", character.current_hp, character.max_hp);

// Take damage
character.take_damage(15);
println!("After damage: {}/{}", character.current_hp, character.max_hp);

// Heal
character.heal(10);
println!("After healing: {}/{}", character.current_hp, character.max_hp);

// Gain XP
character.add_experience(500);
if character.can_level_up() {
    character.level_up();
    println!("Leveled up to level {}!", character.level);
}
```

---

### Special

**Location:** `src/game/character.rs`

SPECIAL attribute system.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Special {
    pub strength: u8,     // 1-10
    pub perception: u8,   // 1-10
    pub endurance: u8,    // 1-10
    pub charisma: u8,     // 1-10
    pub intelligence: u8, // 1-10
    pub agility: u8,      // 1-10
    pub luck: u8,         // 1-10
}
```

#### Methods

```rust
impl Special {
    /// Create default SPECIAL (all stats = 1)
    pub fn new() -> Self;

    /// Calculate total points allocated
    pub fn total_points(&self) -> u8;
}
```

#### Stat Effects

- **Strength**: Melee damage, carry weight
- **Perception**: Accuracy, detection
- **Endurance**: Max HP (15 + STR + END×2)
- **Charisma**: Speech, barter
- **Intelligence**: Skill points
- **Agility**: Max AP (5 + AGI/2), small guns
- **Luck**: Critical chance

---

### Skills

**Location:** `src/game/character.rs`

18 Fallout-style skills derived from SPECIAL.

```rust
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
```

#### Methods

```rust
impl Skills {
    /// Calculate skills from SPECIAL stats
    pub fn from_special(special: &Special) -> Self;

    /// Get a skill value by name
    pub fn get_skill(&self, name: &str) -> u8;
}
```

#### Skill Formulas

```rust
small_guns = 5 + (agility × 4)
speech = charisma × 5
science = intelligence × 4
lockpick = 10 + perception + agility
// ... etc.
```

---

### CombatState

**Location:** `src/game/combat.rs`

Manages active combat encounters.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub active: bool,
    pub round: u32,
    pub enemies: SmallVec<[Enemy; 8]>,  // Optimized for ≤8 enemies
}
```

#### Methods

```rust
impl CombatState {
    /// Create a new inactive combat state
    pub fn new() -> Self;

    /// Start combat with given enemies
    pub fn start_combat(&mut self, enemies: Vec<Enemy>);

    /// End combat (cleanup)
    pub fn end_combat(&mut self);

    /// Advance to next round
    pub fn next_round(&mut self);

    /// Remove dead enemies
    pub fn remove_dead_enemies(&mut self);
}
```

---

### Enemy

**Location:** `src/game/combat.rs`

Represents an enemy in combat.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: SmartString,
    pub level: u32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub armor_class: i32,
    pub damage: SmartString,  // e.g., "2d6+3"
    pub ap: i32,
    pub xp_reward: u32,
    pub skill: u8,
    pub strength: u8,
}
```

#### Methods

```rust
impl Enemy {
    /// Create a generic enemy with scaling stats
    pub fn new(name: &str, level: u32) -> Self;

    /// Create a raider enemy (human with guns)
    pub fn raider(level: u32) -> Self;

    /// Create a radroach enemy (weak insect)
    pub fn radroach(level: u32) -> Self;

    /// Create a super mutant enemy (tough elite)
    pub fn super_mutant(level: u32) -> Self;

    /// Check if enemy is alive
    pub fn is_alive(&self) -> bool;

    /// Apply damage to enemy
    pub fn take_damage(&mut self, damage: i32);
}
```

#### Example

```rust
use fallout_dnd::game::combat::{CombatState, Enemy};

let mut combat = CombatState::new();
combat.start_combat(vec![
    Enemy::raider(3),
    Enemy::raider(2),
    Enemy::radroach(1),
]);

println!("Combat started! {} enemies", combat.enemies.len());
```

---

### Combat Functions

**Location:** `src/game/combat.rs`

```rust
/// Roll dice (supports "2d6+3", "1d20", etc.)
pub fn roll_dice(dice_str: &str) -> i32;

/// Replace stat modifiers ("1d8+STR" → "1d8+3")
/// Returns Cow<str> to avoid allocations (hot path)
pub fn resolve_stat_modifiers(damage_str: &str, strength: u8) -> Cow<'_, str>;

/// Calculate damage with optional critical hit
pub fn calculate_damage(damage_str: &str, strength: u8, critical: bool) -> i32;

/// Make an attack roll
/// Returns (hit: bool, critical: bool)
pub fn attack_roll(attacker_skill: u8, target_ac: i32) -> (bool, bool);
```

#### Example

```rust
use fallout_dnd::game::combat::{roll_dice, calculate_damage, attack_roll};

// Roll dice
let damage = roll_dice("2d6+3");
println!("Rolled: {}", damage);

// Calculate weapon damage
let total_damage = calculate_damage("1d8+STR", 6, false);
println!("Total damage: {}", total_damage);

// Attack roll
let (hit, critical) = attack_roll(50, 15);
if hit {
    if critical {
        println!("Critical hit!");
    } else {
        println!("Hit!");
    }
}
```

---

### Items

**Location:** `src/game/items.rs`

```rust
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Weapon(WeaponStats),
    Armor(ArmorStats),
    Consumable(ConsumableEffect),
    Misc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeaponStats {
    pub damage: SmartString,
    pub damage_type: DamageType,
    pub weapon_type: WeaponType,
    pub ap_cost: i32,
    pub ammo_type: Option<SmartString>,
    pub range: u32,
    pub critical_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WeaponType {
    SmallGun,      // Pistols, SMGs
    BigGun,        // Miniguns, rocket launchers
    EnergyWeapon,  // Laser, plasma
    MeleeWeapon,   // Swords, clubs
    Unarmed,       // Fists, brass knuckles
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsumableEffect {
    Healing(i32),
    RadAway(i32),
    StatBuff { stat: SmartString, amount: i32, duration: u32 },
    Addiction { effect: SmartString },
}
```

#### Methods

```rust
impl Item {
    /// Create a new weapon
    pub fn new_weapon(
        id: &str, name: &str, description: &str,
        damage: &str, damage_type: DamageType,
        weapon_type: WeaponType, ap_cost: i32, value: u32
    ) -> Self;

    /// Create a new armor
    pub fn new_armor(
        id: &str, name: &str, description: &str,
        dr: i32, value: u32
    ) -> Self;

    /// Create a new consumable
    pub fn new_consumable(
        id: &str, name: &str, description: &str,
        effect: ConsumableEffect, value: u32
    ) -> Self;
}

/// Get starting items for new characters
pub fn get_starting_items() -> Vec<Item>;
```

---

### Worldbook

**Location:** `src/game/worldbook.rs`

Persistent world knowledge tracking.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worldbook {
    pub locations: HashMap<String, Location>,
    pub npcs: HashMap<String, NPC>,
    pub events: Vec<Event>,
    pub current_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub description: String,
    pub visited: bool,
    pub visited_at: Option<DateTime<Utc>>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub id: String,
    pub name: String,
    pub description: String,
    pub faction: Option<String>,
    pub met: bool,
    pub relationship: i32,  // -100 to 100
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub location: Option<String>,
    pub npcs: Vec<String>,
}
```

#### Methods

```rust
impl Worldbook {
    /// Create a new empty worldbook
    pub fn new() -> Self;

    /// Create worldbook with default Fallout locations
    pub fn with_defaults() -> Self;

    /// Add or update a location
    pub fn add_location(&mut self, location: Location);

    /// Get a location by ID
    pub fn get_location(&self, id: &str) -> Option<&Location>;

    /// Add or update an NPC
    pub fn add_npc(&mut self, npc: NPC);

    /// Get an NPC by ID
    pub fn get_npc(&self, id: &str) -> Option<&NPC>;

    /// Add an event to the timeline
    pub fn add_event(&mut self, event: Event);

    /// Set current location
    pub fn set_current_location(&mut self, id: Option<String>);

    /// Count total locations
    pub fn count_locations(&self) -> usize;

    /// Count total NPCs
    pub fn count_npcs(&self) -> usize;
}
```

---

### Persistence

**Location:** `src/game/persistence.rs`

Save/load system with security validation.

```rust
/// Save game state to file
pub fn save_to_file(game: &GameState, name: &str) -> Result<()>;

/// Load game state from file
pub fn load_from_file(name: &str) -> Result<GameState>;

/// List all available save files
pub fn list_saves() -> Vec<String>;

/// Delete a save file
pub fn delete_save(name: &str) -> Result<()>;
```

#### Security

- Validates filenames (prevents path traversal)
- Rejects names with: `..`, `/`, `\`
- Save directory: `saves/`
- Format: JSON

#### Example

```rust
use fallout_dnd::game::{GameState, persistence};

// Save game
persistence::save_to_file(&game, "my_save")?;

// List saves
let saves = persistence::list_saves();
for save in saves {
    println!("Found save: {}", save);
}

// Load game
let loaded_game = persistence::load_from_file("my_save")?;
```

---

## AI Module

### AIDungeonMaster

**Location:** `src/ai/mod.rs`

Main AI client for generating DM responses.

```rust
#[derive(Clone)]
pub struct AIDungeonMaster {
    config: LlamaConfig,
    client: reqwest::Client,
}
```

#### Methods

```rust
impl AIDungeonMaster {
    /// Create a new AI dungeon master
    pub fn new(config: LlamaConfig) -> Self;

    /// Generate a response from the AI
    pub async fn generate_response(
        &self,
        game_state: &GameState,
        player_action: &str
    ) -> Result<String>;

    /// Generate a streaming response (returns channel)
    pub async fn generate_response_stream(
        &self,
        game_state: &GameState,
        player_action: &str
    ) -> Result<mpsc::Receiver<Result<String, String>>>;

    /// Test connection to llama.cpp server
    pub async fn test_connection(&self) -> Result<()>;

    /// Build prompt with game context (internal)
    fn build_prompt(&self, game_state: &GameState, player_action: &str) -> String;
}
```

#### Example

```rust
use fallout_dnd::ai::AIDungeonMaster;
use fallout_dnd::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load("config.toml")?;
    let dm = AIDungeonMaster::new(config.llama);

    // Test connection
    dm.test_connection().await?;

    // Generate response
    let response = dm.generate_response(&game, "I explore the ruins").await?;
    println!("DM: {}", response);

    Ok(())
}
```

---

### Command Extractor

**Location:** `src/ai/extractor.rs`

Extracts structured commands from AI responses.

```rust
#[derive(Debug, Clone)]
pub struct CommandExtractor;

#[derive(Debug, Clone, PartialEq)]
pub enum GameCommand {
    StartCombat { enemy_type: String, level: u32 },
    GrantItem { item_id: String },
    SetLocation { location: String },
}
```

#### Methods

```rust
impl CommandExtractor {
    pub fn new() -> Self;

    /// Extract all commands from AI response
    pub fn extract_commands(&self, text: &str) -> Vec<GameCommand>;
}
```

#### Command Formats

- Combat: `[COMBAT: raider, 3]`
- Item: `[ITEM: stimpak]`
- Location: `[LOCATION: vault_13]`

#### Example

```rust
use fallout_dnd::ai::extractor::{CommandExtractor, GameCommand};

let extractor = CommandExtractor::new();
let text = "You encounter bandits! [COMBAT: raider, 3]";
let commands = extractor.extract_commands(text);

for cmd in commands {
    match cmd {
        GameCommand::StartCombat { enemy_type, level } => {
            println!("Combat: {} at level {}", enemy_type, level);
        }
        _ => {}
    }
}
```

---

### Response Cache

**Location:** `src/ai/cache.rs`

Moka-based cache for AI responses.

```rust
pub struct ResponseCache {
    cache: Cache<String, String>,
}
```

#### Methods

```rust
impl ResponseCache {
    /// Create a new cache (100 entries, 1 hour TTL)
    pub fn new() -> Self;

    /// Get cached response
    pub async fn get(&self, key: &str) -> Option<String>;

    /// Store response in cache
    pub async fn insert(&self, key: String, value: String);

    /// Clear all cached responses
    pub async fn clear(&self);
}
```

---

## TUI Module

### App

**Location:** `src/tui/app.rs`

Main TUI application state.

```rust
pub struct App {
    pub state: AppState,
    pub game: Option<GameState>,
    pub log: VecDeque<LogMessage>,
    pub input: String,
    pub dm: Option<Arc<AIDungeonMaster>>,
    // ... other fields
}

pub enum AppState {
    MainMenu,
    CharacterCreation,
    Gameplay,
    Combat,
    Inventory,
    WorldbookViewer,
    SettingsEditor,
    Exiting,
}
```

#### Methods

```rust
impl App {
    /// Create a new app
    pub fn new(config: Config) -> Self;

    /// Handle user input
    pub fn handle_input(&mut self, key: KeyEvent);

    /// Update app state (called each frame)
    pub fn update(&mut self) -> Result<()>;

    /// Render to terminal
    pub fn render(&mut self, terminal: &mut Terminal<...>) -> Result<()>;

    /// Check if should exit
    pub fn should_exit(&self) -> bool;
}
```

---

### Terminal Initialization

**Location:** `src/tui/mod.rs`

```rust
/// Initialize terminal for TUI
pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>>;

/// Restore terminal to original state
pub fn restore_terminal(terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()>;
```

#### Example

```rust
use fallout_dnd::tui::{init_terminal, restore_terminal, App};

let mut terminal = init_terminal()?;
let mut app = App::new(config);

loop {
    app.render(&mut terminal)?;
    app.handle_input(/* key event */);

    if app.should_exit() {
        break;
    }
}

restore_terminal(terminal)?;
```

---

## Configuration

### Config

**Location:** `src/config.rs`

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub llama: LlamaConfig,
    pub game: GameConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlamaConfig {
    pub server_url: String,
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameConfig {
    pub starting_level: u32,
    pub starting_caps: u32,
    pub permadeath: bool,
    pub autosave_interval: u32,
}
```

#### Methods

```rust
impl Config {
    /// Load config from TOML file
    pub fn load(path: &str) -> Result<Self>;

    /// Load config with default fallback
    pub fn load_or_default(path: &str) -> Self;

    /// Create default config
    pub fn default() -> Self;
}
```

---

## Error Handling

### GameError

**Location:** `src/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("AI connection error: {0}")]
    AIConnectionError(String),

    #[error("Invalid save name: {0}")]
    InvalidSaveName(String),

    #[error("Save file not found: {0}")]
    SaveNotFound(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    // ... other variants
}
```

All functions return `Result<T, GameError>` or `anyhow::Result<T>`.

---

## Usage Examples

### Complete Game Loop

```rust
use fallout_dnd::{
    config::Config,
    game::{GameState, character::{Character, Special}, persistence},
    ai::AIDungeonMaster,
    tui::{init_terminal, restore_terminal, App},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config
    let config = Config::load("config.toml")?;

    // Initialize AI
    let dm = AIDungeonMaster::new(config.llama.clone());
    dm.test_connection().await?;

    // Create character
    let mut special = Special::new();
    special.strength = 6;
    special.agility = 8;
    special.intelligence = 7;

    let character = Character::new("Wanderer", special);
    let mut game = GameState::new(character);

    // Initialize TUI
    let mut terminal = init_terminal()?;
    let mut app = App::new(config);
    app.game = Some(game);

    // Game loop
    loop {
        app.render(&mut terminal)?;

        // Handle input (simplified)
        // app.handle_input(key_event);

        if app.should_exit() {
            break;
        }
    }

    // Cleanup
    restore_terminal(terminal)?;

    // Save on exit
    if let Some(game) = app.game {
        persistence::save_to_file(&game, "autosave")?;
    }

    Ok(())
}
```

---

### Custom Combat Encounter

```rust
use fallout_dnd::game::combat::{CombatState, Enemy, calculate_damage};

fn run_combat(character: &mut Character) {
    let mut combat = CombatState::new();

    // Spawn enemies
    combat.start_combat(vec![
        Enemy::raider(3),
        Enemy::raider(2),
        Enemy::super_mutant(5),
    ]);

    while combat.active {
        // Player turn
        if let Some(enemy) = combat.enemies.first_mut() {
            let damage = calculate_damage("2d6+3", character.special.strength, false);
            enemy.take_damage(damage);
            println!("Dealt {} damage to {}", damage, enemy.name);
        }

        // Remove dead enemies
        combat.remove_dead_enemies();

        if combat.enemies.is_empty() {
            println!("Victory!");
            break;
        }

        // Enemy turn (simplified)
        for enemy in &combat.enemies {
            let damage = roll_dice(&enemy.damage);
            character.take_damage(damage);
            println!("{} dealt {} damage", enemy.name, damage);
        }

        if !character.is_alive() {
            println!("Game over!");
            break;
        }

        combat.next_round();
    }
}
```

---

### Working with Worldbook

```rust
use fallout_dnd::game::worldbook::{Worldbook, Location, NPC, Event};
use chrono::Utc;

let mut worldbook = Worldbook::with_defaults();

// Add a location
let location = Location {
    id: "ruins".to_string(),
    name: "Abandoned Ruins".to_string(),
    description: "Crumbling buildings covered in radioactive dust".to_string(),
    visited: true,
    visited_at: Some(Utc::now()),
    notes: vec!["Found a stimpak here".to_string()],
};
worldbook.add_location(location);

// Add an NPC
let npc = NPC {
    id: "doc_mitchell".to_string(),
    name: "Doc Mitchell".to_string(),
    description: "Friendly town doctor".to_string(),
    faction: Some("Goodsprings".to_string()),
    met: true,
    relationship: 50,
    notes: vec!["Saved my life".to_string()],
};
worldbook.add_npc(npc);

// Add an event
let event = Event {
    timestamp: Utc::now(),
    description: "Discovered the ruins".to_string(),
    location: Some("ruins".to_string()),
    npcs: vec![],
};
worldbook.add_event(event);

// Query worldbook
println!("Locations: {}", worldbook.count_locations());
println!("NPCs: {}", worldbook.count_npcs());
```

---

## Performance Considerations

### Hot Path Functions

These functions are called frequently and have been optimized:

- `resolve_stat_modifiers` - Returns `Cow<str>` to avoid allocations
- `roll_dice` - Inline, minimal allocations
- `calculate_damage` - Inline, fast path for common case
- Worldbook lookups - HashMap O(1) average

### Memory-Efficient Types

- `SmallVec<[Enemy; 8]>` - Stack allocation for ≤8 enemies
- `SmartString` - Stack allocation for strings ≤23 bytes
- `mimalloc` - Global allocator (5-6x faster)

### Caching

- AI responses cached with Moka (10-50x speedup)
- Token counting uses OnceLock singleton

---

## Thread Safety

Most types are `Clone` but not `Send`/`Sync`. Use `Arc` for shared ownership across threads:

```rust
let dm = Arc::new(AIDungeonMaster::new(config));
let dm_clone = Arc::clone(&dm);

tokio::spawn(async move {
    dm_clone.generate_response(&game, "Hello").await
});
```

---

## Versioning

Current version: `0.1.0`

Save file compatibility:
- Supports migration from legacy story context
- Forward compatibility not guaranteed between versions
- Always backup saves before updating

---

## See Also

- [ARCHITECTURE.md](ARCHITECTURE.md) - System design overview
- [PERFORMANCE.md](PERFORMANCE.md) - Performance optimizations
- [TESTING.md](TESTING.md) - Testing guide
- [README.md](README.md) - Getting started

---

**Last updated:** 2025-11-21
