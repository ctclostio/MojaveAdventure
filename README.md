# Fallout DnD - AI Dungeon Master

A terminal-based RPG set in the Fallout universe, powered by AI dungeon master using llama.cpp.

## Features

- **SPECIAL System**: Character creation with Fallout's iconic stats (Strength, Perception, Endurance, Charisma, Intelligence, Agility, Luck)
- **Turn-based Combat**: Action Points, critical hits, and tactical combat
- **Inventory System**: Weapons, armor, consumables, and more
- **AI Dungeon Master**: Dynamic storytelling powered by local LLMs via llama.cpp
- **Save/Load System**: Persistent game states
- **Terminal UI**: Retro Fallout-style interface with colors and ASCII art

## Prerequisites

1. **Rust** (if not installed):
   ```bash
   # Windows (via rustup-init.exe)
   # Download from: https://rustup.rs/

   # Linux/Mac
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **llama.cpp** with server mode:
   ```bash
   # Clone llama.cpp
   git clone https://github.com/ggerganov/llama.cpp
   cd llama.cpp

   # Build (Windows with CMake)
   cmake -B build
   cmake --build build --config Release

   # Or build (Linux/Mac)
   make
   ```

3. **A GGUF Model** - Download a suitable model:
   - Recommended: `Mistral-7B-Instruct` or `Llama-2-7B` for good performance
   - Download from: https://huggingface.co/models?search=gguf
   - Smaller models (3B-7B) work well for DM tasks

## Setup

### 1. Install Rust Dependencies

The first time you run the project, cargo will download all dependencies automatically.

### 2. Configure llama.cpp Settings

Edit `config.toml` to match your setup:

```toml
[llama]
server_url = "http://localhost:8080"  # Change if using different port
temperature = 0.8                      # Higher = more creative (0.0-1.0)
max_tokens = 512                       # Max response length

[game]
starting_level = 1
starting_caps = 500
permadeath = false
autosave_interval = 5  # minutes (0 to disable)
```

### 3. Start llama.cpp Server

**Windows:**
```bash
cd path\to\llama.cpp\build\bin\Release
llama-server.exe -m path\to\your\model.gguf --port 8080 -c 2048 --threads 4
```

**Linux/Mac:**
```bash
cd path/to/llama.cpp
./llama-server -m path/to/your/model.gguf --port 8080 -c 2048 --threads 4
```

**Important flags:**
- `-m <model>` - Path to your GGUF model file
- `--port 8080` - Port to run on (must match config.toml)
- `-c 2048` - Context size (higher = more memory of conversation)
- `--threads 4` - CPU threads (adjust based on your system)
- `--n-gpu-layers 32` - (Optional) Offload layers to GPU if you have one

### 4. Run the Game

```bash
# From the FalloutDnD directory
cargo run --release
```

The first build will take a few minutes to compile dependencies.

## How to Play

### Main Menu
- **New Game**: Create a character and start a new adventure
- **Load Game**: Continue from a saved game
- **Exit**: Quit the game

### Character Creation
1. Enter your character name
2. Distribute 28 SPECIAL points across 7 attributes (min 1, max 10 each)
3. Skills are automatically calculated based on your SPECIAL stats

### In-Game Commands

**Exploration Mode:**
- Type natural language to interact with the world
  - Examples: "I search the room", "Talk to the merchant", "Head north"
- `inventory` - View your items
- `stats` - View detailed character stats
- `save` - Save your game
- `quit` - Exit to main menu

**Combat Mode:**
- `attack <number>` - Attack enemy by number (e.g., "attack 1")
- `use <item>` - Use an item from inventory
- `run` - Attempt to flee combat

### Tips

1. **SPECIAL Build Guide:**
   - **Strength**: Increases melee damage and carry weight
   - **Perception**: Improves accuracy and awareness
   - **Endurance**: More HP and radiation resistance
   - **Charisma**: Better at speech and barter
   - **Intelligence**: More skill points, better hacking
   - **Agility**: More Action Points, better accuracy
   - **Luck**: Critical hit chance, general fortune

2. **Combat Strategy:**
   - Each action costs Action Points (AP)
   - AP refills each turn
   - Critical hits deal double damage
   - Use cover and tactics described by the DM

3. **Save Often:**
   - The wasteland is dangerous
   - Auto-save is available but manual saves are recommended

## Troubleshooting

### "Cannot connect to llama.cpp"
- Ensure llama-server is running: check for "HTTP server listening" message
- Verify the port matches config.toml
- Test manually: visit http://localhost:8080/health in browser

### llama.cpp is slow
- Use a smaller model (3B-7B parameters)
- Reduce context size (`-c` flag)
- Enable GPU offloading if available (`--n-gpu-layers`)
- Reduce `max_tokens` in config.toml

### Cargo build errors
- Update Rust: `rustup update`
- Clean and rebuild: `cargo clean && cargo build --release`

### Game won't start
- Check if `saves/` directory exists (created automatically)
- Ensure config.toml is valid TOML format
- Run with `RUST_BACKTRACE=1 cargo run` for detailed errors

## Project Structure

```
FalloutDnD/
├── src/
│   ├── main.rs           # Main game loop
│   ├── config.rs         # Configuration management
│   ├── ui/               # Terminal UI components
│   ├── game/
│   │   ├── character.rs  # SPECIAL, skills, character stats
│   │   ├── combat.rs     # Combat system and dice rolling
│   │   ├── items.rs      # Items, weapons, armor
│   │   └── mod.rs        # Game state management
│   └── ai/
│       └── mod.rs        # llama.cpp integration
├── saves/                # Save game files
├── config.toml           # Game and AI configuration
└── Cargo.toml            # Rust dependencies
```

## Customization

### Modify AI Personality
Edit the `system_prompt` in `config.toml` to change DM behavior:
```toml
system_prompt = """You are a gritty, hardcore DM.
Make the wasteland dangerous and unforgiving..."""
```

### Add Custom Items
Edit `src/game/items.rs` and add to `get_starting_items()` function.

### Adjust Difficulty
In `config.toml`:
```toml
[game]
starting_level = 5        # Start at higher level
starting_caps = 5000      # More money
permadeath = true         # Hardcore mode
```

## Advanced: Using Different LLM Backends

This game works with any llama.cpp compatible server. You can also use:
- **Ollama**: Change server_url to `http://localhost:11434/api/generate`
- **LM Studio**: Use their local server URL
- **Text generation web UI**: Compatible endpoint

Just update `config.toml` with the appropriate URL.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

**Quick Start for Contributors:**

1. Install git hooks (prevents formatting issues):
   ```bash
   # Unix/macOS
   ./scripts/install-hooks.sh

   # Windows (PowerShell)
   .\scripts\install-hooks.ps1
   ```

2. Before committing, ensure:
   ```bash
   cargo fmt --all      # Format code
   cargo clippy         # Check for issues
   cargo test           # Run tests
   ```

**Project Ideas:**
- Add more Fallout-specific items and creatures
- Implement settlement building
- Add companion system
- Create quest system
- Improve combat mechanics (cover, different weapon types)
- Add radiation and status effects

## License

MIT License - Feel free to use and modify!

## Credits

- Built with Rust, llama.cpp, and love for Fallout
- SPECIAL system © Bethesda/Interplay
- Inspired by classic Fallout RPGs

---

**War... war never changes. But your adventure starts now!**
