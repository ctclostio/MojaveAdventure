# Quick Start Guide

Get up and running in 5 minutes!

## Step 1: Install Rust (if needed)

**Windows:**
- Download and run: https://win.rustup.rs/x86_64

**Linux/Mac:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Step 2: Setup llama.cpp

### Option A: Quick Test (Without AI)
You can run the game immediately without AI DM:
```bash
cargo run --release
```
The game will warn about missing AI but you can still play with manual combat and exploration.

### Option B: Full AI Experience

1. **Download llama.cpp** (Windows pre-built):
   - https://github.com/ggerganov/llama.cpp/releases
   - Extract to `C:\llama.cpp`

2. **Download a model**:
   - Go to: https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF
   - Download: `mistral-7b-instruct-v0.2.Q4_K_M.gguf` (4 GB)
   - Save to: `C:\llama.cpp\models\`

3. **Start llama server**:
   ```powershell
   cd C:\llama.cpp
   .\llama-server.exe -m models\mistral-7b-instruct-v0.2.Q4_K_M.gguf --port 8080
   ```

4. **In a new terminal, run the game**:
   ```bash
   cd FalloutDnD
   cargo run --release
   ```

## Step 3: Play!

### First Launch
1. Choose "New Game"
2. Create your character:
   - Enter name
   - Distribute 28 SPECIAL points
3. Start your adventure!

### Example Actions
```
> I search the abandoned building for supplies
> Talk to the merchant about rumors
> Attack 1
> inventory
> save
```

## Recommended SPECIAL Builds

**Gunslinger:**
- S:4 P:8 E:5 C:3 I:5 A:8 L:5
- High accuracy, good AP for multiple shots

**Tank:**
- S:8 P:4 E:9 C:3 I:4 A:4 L:6
- Lots of HP, strong melee

**Smooth Talker:**
- S:3 P:6 E:4 C:9 I:7 A:5 L:4
- Avoid fights, negotiate, high speech

**Balanced:**
- S:5 P:5 E:6 C:5 I:6 A:5 L:6
- Good at everything, master of none

## Next Steps

- Read `setup_llama.md` for detailed llama.cpp configuration
- Read `README.md` for full documentation
- Edit `config.toml` to customize AI personality and game settings

## Troubleshooting One-Liners

**"Cannot connect to llama.cpp"**
→ Start llama-server first, or play without AI

**Cargo not found**
→ Install Rust, then restart terminal

**Too slow**
→ Use smaller model (Q4 instead of Q6) or reduce max_tokens in config.toml

**Build errors**
→ Run: `rustup update && cargo clean && cargo build --release`

## Have Fun!

The wasteland awaits. Remember: save often, trust no one, and may your SPECIAL stats guide you well!
