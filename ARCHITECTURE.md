# Fallout D&D - Architecture Overview

## Table of Contents
- [System Design](#system-design)
- [Core Modules](#core-modules)
- [Data Flow](#data-flow)
- [Module Relationships](#module-relationships)
- [Design Patterns](#design-patterns)
- [Performance Considerations](#performance-considerations)

---

## System Design

Fallout D&D is a terminal-based RPG that combines:
- **Game Logic** (Rust) - Character progression, combat mechanics, inventory
- **AI Integration** (llama.cpp) - Dynamic dungeon master responses
- **Terminal UI** (Ratatui) - Retro Fallout-style interface
- **Persistence** (JSON) - Save/load game states

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         User Input                          │
│                      (Terminal/Keyboard)                    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    TUI Layer (Ratatui)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │     App      │  │  UI Renderer │  │    Theme     │      │
│  │   (State)    │  │   (Widgets)  │  │  (Styling)   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   Game Logic Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   GameState  │  │   Character  │  │    Combat    │      │
│  │  (Central)   │  │  (SPECIAL)   │  │ (Turn-based) │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Worldbook   │  │    Items     │  │    Rolls     │      │
│  │  (Knowledge) │  │ (Inventory)  │  │   (Dice)     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                      AI Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ DungeonMaster│  │  Extractor   │  │    Cache     │      │
│  │  (llama.cpp) │  │  (Commands)  │  │   (Moka)     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Persistence Layer                          │
│                (JSON Save/Load System)                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Modules

### 1. `game/` - Game Logic
**Location:** `src/game/`

The game module is the heart of the application, containing all core gameplay mechanics.

#### Key Components:

**`character.rs`** - Character System
- SPECIAL attributes (Strength, Perception, Endurance, Charisma, Intelligence, Agility, Luck)
- 18 derived skills (Small Guns, Speech, Science, etc.)
- Experience points and leveling system
- Inventory management

**`combat.rs`** - Combat System
- Turn-based combat with action points (AP)
- Dice rolling mechanics (2d6+3, 1d8+STR, etc.)
- Enemy creation and AI
- Critical hits and damage calculation
- Optimized hot paths (Cow for resolve_stat_modifiers)

**`worldbook.rs`** - Persistent World Knowledge
- Location tracking (HashMap-based)
- NPC registry
- Event history
- Serialization/deserialization for saves

**`items.rs`** - Item System
- Weapons (melee, ranged, energy)
- Armor (damage reduction)
- Consumables (healing, buffs)
- Quest items

**`rolls.rs`** - Dice & Skill Checks
- Dice roll parsing (supports "2d6+3", "1d20-2", etc.)
- Skill check resolution
- Critical success/failure detection

**`story_manager.rs`** - Legacy Story Context
- FIFO queue for conversation history (kept for backward compatibility)
- Maximum capacity management

**`conversation.rs`** - Conversation Manager (NEW)
- Structured turn-based conversation tracking
- Player/DM turn separation
- Replaces legacy story_manager in new code

**`persistence.rs`** - Save/Load System
- JSON serialization
- Path traversal protection
- Save directory management

**`handlers.rs`** - Command Processing
- User input parsing
- Action execution (attack, use item, inventory)
- State transitions

---

### 2. `ai/` - AI Integration
**Location:** `src/ai/`

Integrates with local llama.cpp server for dynamic storytelling.

#### Key Components:

**`mod.rs`** - AIDungeonMaster
- HTTP client for llama.cpp server
- Prompt building with game context
- Streaming response support
- Token counting with tiktoken-rs
- Tera template rendering

**`extractor.rs`** - Command Extraction
- Parses AI responses for game commands
- Extracts combat triggers: `[COMBAT: enemy_type, level]`
- Extracts item grants: `[ITEM: item_name]`
- Extracts location updates: `[LOCATION: location_name]`
- Uses regex for robust parsing

**`cache.rs`** - Response Caching
- Moka-based async cache
- 10-50x speedup for repeated queries
- TTL and size limits

---

### 3. `tui/` - Terminal UI
**Location:** `src/tui/`

Provides the retro Fallout-inspired terminal interface using Ratatui.

#### Key Components:

**`app.rs`** - Application State
- App state machine (MainMenu, CharacterCreation, Gameplay, etc.)
- Log message queue
- Input buffer management
- Frame update logic

**`ui.rs`** - Main UI Renderer
- Layout composition
- Widget rendering
- Status bar, message log, input field

**`theme.rs`** - Visual Styling
- Fallout-inspired color palette (green, amber, red)
- ASCII borders and decorations
- Consistent styling across screens

**`combat_display.rs`** - Combat UI
- Enemy list rendering
- HP bars and status indicators
- Combat log display

**`narrative.rs`** - Story Display
- Scrollable narrative text
- Formatting for AI responses
- Message history

**`worldbook_ui.rs` / `worldbook_browser.rs`** - Worldbook Viewer
- Location/NPC browsing
- Event timeline
- Search functionality

**`animations.rs`** - UI Animations
- Typing effects
- Progress indicators
- Transition effects

**`events.rs`** - Event Handling
- Keyboard input capture
- Event polling
- Input debouncing

---

### 4. Supporting Modules

**`config.rs`** - Configuration Management
- TOML config loading
- Validation with Garde
- Default values

**`error.rs`** - Error Handling
- Custom error types
- Miette integration for pretty error messages
- Context propagation

**`templates.rs`** - Tera Templates
- AI prompt templates
- Context serialization
- Template validation

**`validation.rs` / `validation_garde.rs`** - Input Validation
- Character name validation
- SPECIAL point allocation
- Command parsing
- Garde-based declarative validation

---

## Data Flow

### 1. User Input Flow

```
User Types Command
      ↓
EventHandler captures input
      ↓
App.handle_input() processes
      ↓
┌─────────────────────────────┐
│   Command Type Detection    │
├─────────────────────────────┤
│ • "attack 1" → Combat       │
│ • "inventory" → Show items  │
│ • "I explore" → AI query    │
│ • "save" → Persistence      │
└─────────────────────────────┘
      ↓
Handler executes action
      ↓
GameState updated
      ↓
UI re-renders
```

### 2. AI Response Flow

```
Player Action ("I explore the ruins")
      ↓
AIDungeonMaster.generate_response()
      ↓
Build prompt with context:
  • Character stats (HP, SPECIAL, skills)
  • Inventory
  • Combat state (if in combat)
  • Worldbook (known locations, NPCs)
  • Conversation history (last 10 turns)
  • Current location
      ↓
HTTP POST to llama.cpp
      ↓
Receive AI response
      ↓
Extractor.extract_commands()
      ↓
Parse and execute commands:
  • [COMBAT: raider, 3] → Start combat
  • [ITEM: Stimpak] → Add to inventory
  • [LOCATION: Ruins] → Update worldbook
      ↓
Update GameState
      ↓
Display narrative + execute commands
      ↓
UI updates
```

### 3. Combat Flow

```
Combat Initiated
      ↓
CombatState.start_combat(enemies)
      ↓
┌─────────────────────────────┐
│      Combat Turn Loop       │
├─────────────────────────────┤
│ 1. Player turn (AP-based)   │
│    • Attack (costs AP)      │
│    • Use item (costs AP)    │
│    • Run (costs AP)         │
│                             │
│ 2. Enemy turns              │
│    • Each enemy attacks     │
│    • Damage calculation     │
│                             │
│ 3. Round cleanup            │
│    • Remove dead enemies    │
│    • Refill AP              │
│    • Increment round        │
└─────────────────────────────┘
      ↓
Combat ends when:
  • All enemies dead → Victory + XP
  • Player dead → Game over
  • Player flees → Escape
      ↓
GameState updated
```

### 4. Save/Load Flow

```
User: "save my_game"
      ↓
persistence::save_to_file()
      ↓
Serialize GameState to JSON
      ↓
Validate filename (prevent path traversal)
      ↓
Write to saves/my_game.json
      ↓
Success confirmation

User: "load my_game"
      ↓
persistence::load_from_file()
      ↓
Read saves/my_game.json
      ↓
Deserialize JSON to GameState
      ↓
Migrate legacy data (story → conversation)
      ↓
GameState loaded
```

---

## Module Relationships

### Dependency Graph

```
main.rs
  ├── config.rs (Config loading)
  ├── tui/
  │   ├── app.rs
  │   │   ├── game/ (GameState, handlers)
  │   │   ├── ai/ (AIDungeonMaster)
  │   │   └── tui/ui.rs (rendering)
  │   ├── ui.rs
  │   │   ├── theme.rs
  │   │   ├── combat_display.rs
  │   │   ├── narrative.rs
  │   │   └── worldbook_ui.rs
  │   └── events.rs
  ├── game/
  │   ├── mod.rs (GameState)
  │   ├── character.rs
  │   ├── combat.rs
  │   ├── worldbook.rs
  │   ├── items.rs
  │   ├── rolls.rs
  │   ├── story_manager.rs
  │   ├── conversation.rs
  │   ├── persistence.rs
  │   └── handlers.rs
  └── ai/
      ├── mod.rs (AIDungeonMaster)
      ├── extractor.rs
      └── cache.rs
```

### Key Relationships:

1. **TUI → Game**: TUI layer depends on game logic but not vice versa
2. **Game → AI**: Game calls AI for DM responses, but game logic is independent
3. **Handlers → Everything**: Handlers orchestrate between TUI, game, and AI
4. **Persistence → GameState**: Persistence only depends on GameState serialization
5. **AI → Game**: AI needs game state for context, but doesn't modify it directly

---

## Design Patterns

### 1. State Machine Pattern
**Location:** `tui/app.rs`

The TUI uses an explicit state machine for screen transitions:

```rust
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

**Benefits:**
- Clear screen transitions
- Prevents invalid states
- Easy to add new screens

### 2. Repository Pattern
**Location:** `game/persistence.rs`

Save/load operations are abstracted:

```rust
pub fn save_to_file(game: &GameState, name: &str) -> Result<()>
pub fn load_from_file(name: &str) -> Result<GameState>
pub fn list_saves() -> Vec<String>
```

**Benefits:**
- Decouples game logic from storage format
- Easy to change serialization format
- Centralized validation

### 3. Cache-Aside Pattern
**Location:** `ai/cache.rs`

Moka cache implements cache-aside:

```rust
// Check cache first
if let Some(cached) = cache.get(&key).await {
    return cached;
}

// Cache miss: compute value
let result = expensive_operation().await;

// Store in cache
cache.insert(key, result.clone()).await;

return result;
```

**Benefits:**
- 10-50x speedup for repeated AI queries
- Automatic TTL expiration
- Size limits prevent memory issues

### 4. Template Method Pattern
**Location:** `ai/mod.rs`, `templates.rs`

AI prompts use Tera templates:

```rust
let template = tera.render("dm_prompt", &context)?;
```

**Benefits:**
- Separates prompt logic from code
- Easy to iterate on prompts
- Consistent formatting

### 5. Builder Pattern
**Location:** `tests/helpers.rs`

Test helpers use builder pattern:

```rust
fn create_custom_character(name: &str, special: Special) -> Character {
    Character::new(name.to_string(), special)
}
```

**Benefits:**
- Readable test setup
- Flexible configuration
- Reduces test boilerplate

### 6. Observer Pattern (Implicit)
**Location:** `tui/app.rs`

TUI observes game state changes and re-renders:

```rust
// Game state changes
game.character.take_damage(10);

// UI automatically reflects changes on next frame
app.render(&mut terminal)?;
```

**Benefits:**
- Decoupled rendering from game logic
- Efficient updates (only render when needed)

---

## Performance Considerations

### Memory Optimizations

**1. mimalloc Allocator**
- 5-6x faster allocations
- Enabled globally in `main.rs`
- Minimal configuration required

**2. SmallVec for Combat**
- Eliminates heap allocations for ≤8 enemies (80%+ of encounters)
- `SmallVec<[Enemy; 8]>` stores up to 8 enemies on stack

**3. SmartString for Short Strings**
- Stack allocation for strings ≤23 bytes
- Reduces heap pressure
- Used for character names, item names, etc.

**4. Cow for Conditional Allocations**
- `resolve_stat_modifiers` returns `Cow<str>`
- Zero-cost when no string replacement needed (85% of cases)
- 7.8x faster than always allocating

### Caching Strategies

**1. Moka Cache (AI Layer)**
- Async cache with TTL
- Size limits prevent unbounded growth
- 10-50x speedup for repeated queries

**2. OnceLock for Singletons**
- Thread-safe lazy initialization
- Used for tiktoken tokenizer
- Zero overhead after first access

### Hot Path Optimizations

**1. Dice Rolling (combat.rs)**
- Inline critical functions
- Minimize allocations
- Pre-compute common rolls

**2. Attack Resolution (combat.rs)**
- Fast path for common case (no STR modifiers)
- Cow avoids allocations
- Benchmarked: 8.21ns for no-replacement case

**3. Enemy Scaling (combat.rs)**
- Simple formulas
- No complex lookups
- Predictable performance

---

## Testing Strategy

### Test Organization

1. **Unit Tests**: Inline in source files (`#[cfg(test)]`)
2. **Integration Tests**: `tests/` directory
3. **Property Tests**: `tests/property_tests.rs` (proptest)
4. **Benchmarks**: `benches/` directory (divan)

### Test Coverage

- **Core gameplay**: 80%+ line coverage
- **Game logic**: 70%+ line coverage
- **Overall project**: 60%+ line coverage

### Test Infrastructure

- **insta**: Snapshot testing for complex outputs
- **proptest**: Property-based testing for invariants
- **serial_test**: Serialize tests that share resources
- **divan**: Fast benchmarking with allocation tracking

---

## Configuration

### config.toml Structure

```toml
[llama]
server_url = "http://localhost:8080"
temperature = 0.8
max_tokens = 512

[game]
starting_level = 1
starting_caps = 500
permadeath = false
autosave_interval = 5

[ui]
theme = "fallout-green"
animation_speed = "normal"
```

Validated using **Garde** framework for declarative validation.

---

## Security Considerations

### Path Traversal Prevention
**Location:** `game/persistence.rs`

```rust
// Validate filename to prevent "../../../etc/passwd"
if name.contains("..") || name.contains("/") || name.contains("\\") {
    return Err(GameError::InvalidSaveName);
}
```

### Input Sanitization
**Location:** `validation.rs`, `validation_garde.rs`

- Character names: alphanumeric + spaces only
- SPECIAL points: 1-10 range, total ≤ 40
- Command parsing: whitelist-based

---

## Future Architecture Considerations

### Potential Improvements

1. **Plugin System**: Modular content (new items, enemies, quests)
2. **Multiplayer Support**: Shared worldbook, turn-based co-op
3. **Remote AI Backend**: Support for cloud-based LLMs
4. **Database Persistence**: Replace JSON with SQLite for complex queries
5. **WASM Target**: Run in browser with web-based UI

### Scalability

Current architecture scales well for:
- Single-player experiences
- Local AI inference
- Moderate save file sizes (< 10MB)

Potential bottlenecks:
- Large worldbooks (> 10,000 entities)
- Slow AI inference (> 5s per response)
- Frequent saves (disk I/O)

---

## Conclusion

Fallout D&D uses a layered architecture with clear separation of concerns:
- **TUI** for presentation
- **Game** for logic
- **AI** for storytelling
- **Persistence** for state management

Design patterns (State Machine, Repository, Cache-Aside, Template Method) provide structure and maintainability. Performance optimizations (mimalloc, SmallVec, Cow, Moka) ensure smooth gameplay even on modest hardware.

The architecture is designed for extensibility, testability, and performance while maintaining code clarity and Rust idioms.
