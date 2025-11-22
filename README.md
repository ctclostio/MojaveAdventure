# Fallout DnD - AI Dungeon Master

[![Code Coverage](https://img.shields.io/badge/coverage-45.11%25-yellow)](https://github.com/your-username/FalloutDnD/actions/workflows/rust.yml)

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
**Symptoms:** Error message "Failed to connect to llama.cpp server"

**Solutions:**
- Ensure llama-server is running: check for "HTTP server listening" message
- Verify the port matches config.toml (default: 8080)
- Test manually: visit http://localhost:8080/health in browser
- Check firewall settings (allow port 8080)
- Try different server URL: `http://127.0.0.1:8080` instead of `localhost`

**Windows-specific:**
```bash
# Check if port is in use
netstat -ano | findstr :8080

# Kill process if needed
taskkill /PID <process_id> /F
```

**Linux/Mac:**
```bash
# Check if port is in use
lsof -i :8080

# Kill process if needed
kill <process_id>
```

---

### llama.cpp is slow
**Symptoms:** AI responses take >5 seconds, game feels sluggish

**Solutions:**

1. **Use a smaller model** (most effective):
   - 3B models: Fast, good for gameplay (~1s response)
   - 7B models: Balanced performance (~2-3s response)
   - 13B+ models: Slow, not recommended (<5s response)

   Download from: https://huggingface.co/models?search=gguf

2. **Reduce context size**:
   ```bash
   llama-server -m model.gguf -c 1024  # Instead of 2048
   ```

3. **Enable GPU offloading** (if you have a GPU):
   ```bash
   # NVIDIA GPU
   llama-server -m model.gguf --n-gpu-layers 32

   # Apple Silicon (M1/M2)
   llama-server -m model.gguf --n-gpu-layers 1  # Uses Metal
   ```

4. **Adjust config.toml**:
   ```toml
   [llama]
   max_tokens = 256  # Reduce from 512
   temperature = 0.7  # Lower = faster, less creative
   ```

5. **Use quantized models**:
   - Q4_K_M: Good balance (recommended)
   - Q5_K_M: Better quality, slower
   - Q8_0: Highest quality, slowest

---

### Cargo build errors
**Symptoms:** Compilation fails with cryptic errors

**Solutions:**
1. Update Rust toolchain:
   ```bash
   rustup update stable
   rustup default stable
   ```

2. Clean and rebuild:
   ```bash
   cargo clean
   cargo build --release
   ```

3. Check Rust version (minimum: 1.70):
   ```bash
   rustc --version
   ```

4. Install missing dependencies (Linux):
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential pkg-config libssl-dev

   # Fedora
   sudo dnf install gcc pkg-config openssl-devel
   ```

---

### Game won't start
**Symptoms:** Crashes on launch, immediate exit

**Solutions:**
1. Check if `saves/` directory exists (should be created automatically)
2. Verify config.toml is valid:
   ```bash
   # Use a TOML validator online or check syntax
   ```

3. Run with debug output:
   ```bash
   RUST_BACKTRACE=1 cargo run --release
   ```

4. Check terminal size (minimum: 80x24):
   ```bash
   # Linux/Mac
   echo $COLUMNS x $LINES

   # Should be at least 80x24
   ```

5. Test with default config:
   ```bash
   mv config.toml config.toml.backup
   cargo run --release  # Will create default config
   ```

---

### Save/Load issues
**Symptoms:** Can't save game, save files corrupted

**Solutions:**
1. Check write permissions on `saves/` directory
2. Ensure disk space available (saves are small, ~100KB typically)
3. Verify save file format (should be valid JSON):
   ```bash
   # Pretty-print save file
   cat saves/my_save.json | python -m json.tool
   ```

4. Backup saves regularly:
   ```bash
   cp -r saves/ saves_backup/
   ```

---

### Performance issues
**Symptoms:** Low FPS, stuttering, high memory usage

**Solutions:**
1. Run in release mode (not debug):
   ```bash
   cargo run --release  # Much faster!
   ```

2. Check system resources:
   - RAM usage: Should be <100MB for game only
   - CPU: llama.cpp will use 100% during AI generation (normal)

3. Reduce terminal update rate (in future version):
   - Target 30 FPS instead of 60 FPS

4. Close other applications using GPU/CPU

---

### Terminal display issues
**Symptoms:** Garbled text, broken borders, wrong colors

**Solutions:**
1. Use a modern terminal:
   - Windows: Windows Terminal (recommended), ConEmu
   - Mac: iTerm2, built-in Terminal.app
   - Linux: Alacritty, Kitty, GNOME Terminal

2. Enable UTF-8 support:
   ```bash
   # Windows (PowerShell)
   [Console]::OutputEncoding = [System.Text.Encoding]::UTF8

   # Linux/Mac (add to ~/.bashrc or ~/.zshrc)
   export LANG=en_US.UTF-8
   export LC_ALL=en_US.UTF-8
   ```

3. Check terminal color support:
   ```bash
   echo $TERM
   # Should be: xterm-256color, screen-256color, or similar
   ```

4. Resize terminal (minimum 80x24, recommended 120x40)

---

### Common Error Messages

#### "Error: Invalid save name"
- Save name contains invalid characters
- Use only: letters, numbers, hyphens, underscores
- Max length: 255 characters

#### "Error: Model not loaded"
- llama.cpp couldn't load the model file
- Check model path in llama-server command
- Ensure model file exists and is readable

#### "Error: Out of memory"
- Model too large for available RAM
- Use smaller model or increase system RAM
- Reduce context size (`-c` flag)

#### "Error: Thread panicked"
- Usually a bug - please report!
- Run with `RUST_BACKTRACE=full` and share output
- Check GitHub issues: https://github.com/yourusername/FalloutDnD/issues

---

## Performance Tips

### Optimized for Speed
The game uses several performance optimizations:
- **mimalloc allocator**: 5-6x faster memory allocations
- **SmallVec**: Zero heap allocations for typical combat (≤8 enemies)
- **SmartString**: Stack allocation for short strings
- **Moka cache**: 10-50x speedup for repeated AI queries

See [PERFORMANCE.md](PERFORMANCE.md) for detailed optimization information.

### Expected Performance
- **Frame rate**: 60 FPS (16.67ms per frame)
- **Memory usage**: <100MB for game, varies for llama.cpp
- **Combat response**: <50ms per action
- **AI response**: 1-5 seconds (depends on model and hardware)

### Maximizing Performance

#### 1. Always Use Release Mode
```bash
# SLOW (debug mode)
cargo run

# FAST (release mode - 10-100x faster!)
cargo run --release
```

#### 2. Choose the Right Model
- **3B quantized (Q4_K_M)**: Best for gameplay, ~1-2s responses
- **7B quantized (Q4_K_M)**: Good balance, ~2-3s responses
- **13B+**: Slow, not recommended unless you have a powerful GPU

#### 3. Enable GPU Acceleration
If you have an NVIDIA GPU:
```bash
# Check GPU support
nvidia-smi

# Enable GPU layers
llama-server -m model.gguf --n-gpu-layers 32
```

For Apple Silicon (M1/M2/M3):
```bash
# Uses Metal automatically
llama-server -m model.gguf --n-gpu-layers 1
```

#### 4. Tune llama.cpp Settings
```bash
# Balanced settings
llama-server -m model.gguf \
  -c 1024 \              # Context size (lower = faster)
  --threads 4 \          # CPU threads (match your CPU)
  --n-gpu-layers 32 \    # GPU offloading (if available)
  --mlock                # Lock model in RAM (prevents swapping)
```

#### 5. Optimize config.toml
```toml
[llama]
server_url = "http://localhost:8080"
temperature = 0.7      # Lower = faster, less random
max_tokens = 256       # Shorter responses = faster
top_k = 40             # Reduce for faster sampling
```

#### 6. System Recommendations

**Minimum specs:**
- CPU: 4 cores, 2.0 GHz
- RAM: 8GB
- Storage: 5GB free (for model + game)

**Recommended specs:**
- CPU: 8+ cores, 3.0+ GHz
- RAM: 16GB
- GPU: NVIDIA GTX 1060 or better
- Storage: 10GB free (SSD preferred)

**Optimal specs:**
- CPU: 16+ cores, 3.5+ GHz
- RAM: 32GB+
- GPU: NVIDIA RTX 3060 or better
- Storage: 20GB free (NVMe SSD)

### Profiling Performance

If you experience performance issues:

1. **Profile with flamegraph**:
```bash
cargo install flamegraph
cargo flamegraph --bin fallout-dnd
# Play for 5-10 minutes, then exit
# Open flamegraph.svg in browser
```

2. **Run benchmarks**:
```bash
cargo bench
# Check results for performance regressions
```

3. **Monitor resources**:
```bash
# Linux
htop

# Windows
Task Manager (Ctrl+Shift+Esc)

# Mac
Activity Monitor
```

## Project Structure

```
FalloutDnD/
├── src/
│   ├── main.rs              # Main game loop and entry point
│   ├── lib.rs               # Library exports
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types and handling
│   ├── templates.rs         # Tera template definitions
│   ├── validation.rs        # Input validation
│   ├── validation_garde.rs  # Garde validation framework
│   ├── game/                # Core game logic
│   │   ├── mod.rs           # GameState and module exports
│   │   ├── character.rs     # SPECIAL, skills, character stats
│   │   ├── combat.rs        # Combat system and dice rolling
│   │   ├── items.rs         # Items, weapons, armor
│   │   ├── worldbook.rs     # Persistent world knowledge
│   │   ├── rolls.rs         # Skill checks and dice rolling
│   │   ├── story_manager.rs # Legacy story context (FIFO)
│   │   ├── conversation.rs  # Conversation manager (NEW)
│   │   ├── persistence.rs   # Save/load system
│   │   ├── handlers.rs      # Command handlers
│   │   ├── char_handlers.rs # Character command handlers
│   │   ├── combat_handlers.rs  # Combat command handlers
│   │   ├── stat_allocator.rs   # SPECIAL point allocation UI
│   │   └── tui_game_loop.rs    # TUI-specific game loop
│   ├── ai/                  # AI integration
│   │   ├── mod.rs           # AIDungeonMaster client
│   │   ├── extractor.rs     # Command extraction from AI
│   │   └── cache.rs         # Moka response cache
│   └── tui/                 # Terminal UI (Ratatui)
│       ├── mod.rs           # Terminal init/restore
│       ├── app.rs           # App state machine
│       ├── ui.rs            # Main UI renderer
│       ├── theme.rs         # Fallout-style theme
│       ├── events.rs        # Event handling
│       ├── animations.rs    # UI animations
│       ├── narrative.rs     # Story display
│       ├── combat_display.rs   # Combat UI
│       ├── worldbook_ui.rs     # Worldbook viewer
│       ├── worldbook_browser.rs # Worldbook browser
│       ├── settings_ui.rs      # Settings screen
│       └── settings_editor.rs  # Settings editor
├── tests/                   # Integration tests
│   ├── helpers.rs           # Test utilities
│   ├── character_tests.rs   # Character tests
│   ├── combat_tests.rs      # Combat tests
│   ├── worldbook_tests.rs   # Worldbook tests
│   ├── handlers_tests.rs    # Handler tests
│   ├── stat_allocator_tests.rs  # SPECIAL allocation tests
│   ├── property_tests.rs    # Property-based tests
│   ├── regression_tests.rs  # Regression tests
│   └── ... (15+ test files)
├── benches/                 # Performance benchmarks (divan)
│   ├── combat_benchmarks.rs    # Combat performance
│   ├── worldbook_benchmarks.rs # Worldbook performance
│   └── ai_benchmarks.rs        # AI performance
├── saves/                   # Save game files (JSON)
├── config.toml              # Game and AI configuration
├── Cargo.toml               # Rust dependencies
├── README.md                # This file
├── ARCHITECTURE.md          # System design overview
├── PERFORMANCE.md           # Performance optimizations
├── API.md                   # Public API documentation
├── TESTING.md               # Testing guide
└── CONTRIBUTING.md          # Contribution guidelines
```

### Key Directories

- **`src/game/`** - All game logic (character, combat, worldbook)
- **`src/ai/`** - AI integration with llama.cpp
- **`src/tui/`** - Terminal UI using Ratatui
- **`tests/`** - 150+ integration tests
- **`benches/`** - Performance benchmarks
- **`saves/`** - JSON save files

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed module relationships.

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

## Documentation

### For Users
- **README.md** (this file) - Getting started, setup, gameplay
- **QUICKSTART.md** - Quick setup guide
- **setup_llama.md** - Detailed llama.cpp setup

### For Developers
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design, modules, data flow, design patterns
- **[API.md](API.md)** - Public API documentation for all core types
- **[PERFORMANCE.md](PERFORMANCE.md)** - Performance optimizations, benchmarks, profiling
- **[TESTING.md](TESTING.md)** - Testing guide, infrastructure, running tests
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines, code style

### Quick Links
- **Character System**: See API.md → Character, Special, Skills
- **Combat Mechanics**: See API.md → CombatState, Enemy
- **AI Integration**: See API.md → AIDungeonMaster
- **Performance Tips**: See PERFORMANCE.md → Optimization Summary
- **Design Patterns**: See ARCHITECTURE.md → Design Patterns
- **Running Tests**: See TESTING.md → Running Tests

## License

MIT License - Feel free to use and modify!

## Credits

- Built with Rust, llama.cpp, and love for Fallout
- SPECIAL system © Bethesda/Interplay
- Inspired by classic Fallout RPGs

## Technologies

### Core
- **Rust** - Systems programming language
- **llama.cpp** - Local LLM inference
- **Ratatui** - Terminal UI framework
- **Tokio** - Async runtime

### Optimization
- **mimalloc** - High-performance allocator (5-6x faster)
- **SmallVec** - Stack-based vectors
- **SmartString** - Stack-based short strings
- **Moka** - Async cache (10-50x speedup)

### Testing
- **insta** - Snapshot testing
- **proptest** - Property-based testing
- **divan** - Fast benchmarking
- **serial_test** - Test serialization

### Other
- **Serde** - Serialization/deserialization
- **tiktoken-rs** - Token counting
- **Tera** - Template engine
- **Garde** - Validation framework
- **Miette** - Pretty error messages

---

**War... war never changes. But your adventure starts now!**

For questions, issues, or contributions, see [CONTRIBUTING.md](CONTRIBUTING.md).
