# FalloutDnD - Claude Code Project Context

> A terminal-based RPG game set in the Fallout universe with AI-powered dungeon master using local LLMs via llama.cpp.

## Quick Reference

```bash
# Build & Run
cargo build --release
cargo run

# Testing
cargo nextest run              # Run all tests
cargo nextest run <test_name>  # Run specific test
cargo test --doc               # Run doctests

# Quality
cargo clippy -- -D warnings    # Lint (treat warnings as errors)
cargo fmt --check              # Format check
cargo fmt                      # Auto-format

# Benchmarks
cargo bench                    # Run divan benchmarks
```

## Architecture Overview

```
src/
├── main.rs              # Entry point, async runtime setup
├── lib.rs               # Library root, module exports
├── config.rs            # Configuration (Garde validation)
├── error.rs             # Error types (thiserror + miette)
├── game/                # Core game logic
│   ├── mod.rs           # GameState central struct
│   ├── character.rs     # SPECIAL stats, skills, inventory
│   ├── combat.rs        # Turn-based combat, Enemy types
│   ├── items.rs         # Item system (weapons, armor, consumables)
│   ├── worldbook.rs     # Persistent world knowledge
│   ├── conversation.rs  # AI conversation context (new)
│   ├── story_manager.rs # Legacy context (backward compat)
│   ├── rolls.rs         # Dice mechanics, skill checks
│   ├── handlers.rs      # Main command processing
│   ├── persistence.rs   # Save/load with security checks
│   └── ...
├── ai/                  # AI integration
│   ├── mod.rs           # AIDungeonMaster, llama.cpp client
│   ├── extractor.rs     # Parse AI responses for commands
│   ├── cache.rs         # Moka response caching
│   └── server_manager.rs # Auto-start llama.cpp servers
└── tui/                 # Terminal UI (Ratatui)
    ├── mod.rs           # Terminal init/restore
    ├── app.rs           # App state machine
    ├── ui.rs            # Main renderer
    ├── theme.rs         # Fallout-style colors
    └── ...
```

## Key Data Structures

### GameState (`src/game/mod.rs`)
Central hub containing all game data:
```rust
pub struct GameState {
    pub character: Character,           // Player stats, inventory
    pub combat: CombatState,           // Active combat encounters
    pub conversation: ConversationManager, // AI context (new)
    pub story: StoryManager,           // Legacy context
    pub location: String,
    pub quest_log: Vec<String>,
    pub worldbook: Worldbook,          // NPCs, locations, events
    pub day: u32,
}
```

### Enemy (`src/game/combat.rs`)
```rust
pub struct Enemy {
    pub name: SmartString,
    pub level: u32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub armor_class: i32,
    pub damage: SmartString,    // Dice notation: "2d6+3"
    pub ap: i32,
    pub xp_reward: u32,
    pub skill: u8,              // Combat skill (0-100)
    pub strength: u8,           // For melee damage
}
```

### Item (`src/game/items.rs`)
```rust
pub struct Item {
    pub id: SmartString,
    pub name: SmartString,
    pub description: SmartString,
    pub item_type: ItemType,    // Weapon, Armor, Consumable, Misc
    pub weight: f32,
    pub value: u32,             // Caps value
    pub quantity: u32,
}

pub enum ItemType {
    Weapon(WeaponStats),        // damage, weapon_type, ap_cost, etc.
    Armor(ArmorStats),          // DR, radiation_resistance, AC
    Consumable(ConsumableEffect), // Healing, RadAway, StatBuff
    Misc,
}
```

## Code Conventions

### Performance Patterns (HOT PATHS)
- **SmartString**: Use for short strings (names, IDs) - stack-allocated for ≤23 chars
- **SmallVec<[T; 8]>**: Use for combat enemies - avoids heap for ≤8 items
- **Cow<str>**: Use when string might not need modification (e.g., `resolve_stat_modifiers`)
- **mimalloc**: Global allocator for 5-6x faster allocations

### Error Handling
- Use `GameError` enum from `src/error.rs`
- Specialized sub-errors: `CombatError`, `CharacterError`, `ConfigError`
- Always include helpful diagnostic messages via miette

### Serialization
- All game data must derive `Serialize, Deserialize`
- Use `#[serde(default = "...")]` for backward compatibility with saves
- JSON for saves, TOML for configuration

### Testing Patterns
- **Snapshot tests**: Use `insta::assert_json_snapshot!` for complex structs
- **Property tests**: Use `proptest` for invariants
- **Test helpers**: Use `tests/helpers.rs` for fixtures
- **Naming**: `test_<feature>_<scenario>` or `snapshot_<struct>_<case>`

## Adding New Enemies

Location: `src/game/combat.rs`

1. Add a constructor method to `Enemy` impl:
```rust
pub fn new_enemy_type(level: u32) -> Self {
    let mut enemy = Enemy::new("Enemy Name", level);
    enemy.max_hp = /* formula */;
    enemy.current_hp = enemy.max_hp;
    enemy.damage = SmartString::from("XdY+Z");
    enemy.skill = /* 0-100 */;
    enemy.strength = /* for melee bonus */;
    enemy.armor_class = /* base + level scaling */;
    enemy
}
```

2. Add tests in `tests/combat_tests.rs`:
```rust
#[test]
fn test_enemy_creation_new_type() {
    let enemy = Enemy::new_enemy_type(1);
    assert!(enemy.name.contains("Enemy Name"));
    assert!(enemy.is_alive());
}

#[test]
fn snapshot_enemy_new_type() {
    insta::assert_json_snapshot!(Enemy::new_enemy_type(3));
}
```

## Adding New Items

Location: `src/game/items.rs`

### Weapons
```rust
Item::new_weapon(
    "unique_id",           // snake_case, unique
    "Display Name",
    "Description text",
    "2d6+3",               // Dice notation
    DamageType::Normal,    // Normal, Laser, Plasma, Fire, Explosive, Poison
    WeaponType::SmallGun,  // SmallGun, BigGun, EnergyWeapon, MeleeWeapon, Unarmed
    4,                     // AP cost
    150,                   // Caps value
)
```

### Armor
```rust
Item::new_armor(
    "unique_id",
    "Display Name",
    "Description text",
    15,                    // Damage Resistance
    500,                   // Caps value
)
// AC calculated as: 5 + (DR / 2)
```

### Consumables
```rust
Item::new_consumable(
    "unique_id",
    "Display Name",
    "Description text",
    ConsumableEffect::Healing(30),  // or RadAway(50), StatBuff{...}
    75,                             // Caps value
)
```

## AI Integration

### Two-Model Setup
1. **Narrative AI** (port 8080): Large model for storytelling (Cydonia 24B)
2. **Extraction AI** (port 8081): Small model for parsing (Hermes-2-Pro 8B)

### AI Response Commands
The extraction AI parses narrative for structured commands:
- `[COMBAT: enemy_type, level]` - Trigger combat
- `[ITEM: item_name]` - Grant item
- `[LOCATION: location_name]` - Update location

### Prompt Templates
Located in AI module, use Tera templating. Context includes:
- Character stats (SPECIAL, skills, HP, AP)
- Combat state (enemies, round)
- Conversation history (last 10 turns)
- Worldbook knowledge

## Configuration

`config.toml` structure:
```toml
[llama]
server_url = "http://localhost:8080"
extraction_url = "http://localhost:8081"
temperature = 0.8
# GPU settings
narrative_gpu_layers = 99  # All layers to GPU
flash_attention = true
continuous_batching = true

[game]
starting_level = 1
starting_caps = 500
permadeath = false
```

Validation via Garde framework in `src/config.rs`.

## CI/CD Pipeline

GitHub Actions (`.github/workflows/rust.yml`):
1. **Build**: `cargo build --verbose`
2. **Test**: `cargo nextest run`
3. **Coverage**: `cargo llvm-cov` (target: ~45%)
4. **Security**: `cargo audit` via rustsec

## Common Tasks

### Running the Game
```bash
cargo run --release
# Or use the launcher scripts:
./run.sh        # Linux/Mac
run.bat         # Windows
```

### Debugging AI
```bash
# Test AI server connectivity
./scripts/test_ai_servers.ps1

# Check debug logs
cat fallout-dnd-debug.log
```

### Updating Snapshots
```bash
cargo insta test           # Run tests and review
cargo insta accept         # Accept all changes
cargo insta reject         # Reject all changes
```

## File Naming Conventions

- **Source**: `snake_case.rs`
- **Tests**: `<module>_tests.rs` in `tests/`
- **Benchmarks**: `<module>_benchmarks.rs` in `benches/`
- **Item IDs**: `snake_case` (e.g., `10mm_pistol`, `leather_armor`)
- **Enemy names**: Title Case in display, snake_case in code

## Dependencies to Know

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI framework |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `garde` | Config validation |
| `miette` / `thiserror` | Error handling |
| `tiktoken-rs` | Token counting |
| `moka` | Async cache |
| `insta` | Snapshot testing |
| `proptest` | Property testing |
| `divan` | Benchmarking |
