# Quick Setup Guide for llama.cpp

## Step 1: Download llama.cpp

### Option A: Pre-built Release (Easiest for Windows)
1. Go to https://github.com/ggerganov/llama.cpp/releases
2. Download the latest Windows release (e.g., `llama-xxx-bin-win-avx2-x64.zip`)
3. Extract to a folder like `C:\llama.cpp`

### Option B: Build from Source
```bash
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp

# Windows (requires CMake and Visual Studio)
cmake -B build
cmake --build build --config Release

# Linux/Mac
make
```

## Step 2: Download a Model

### Recommended Models for RPG DM:

**Small & Fast (3-4 GB RAM):**
- Mistral-7B-Instruct: https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF
  - Download: `mistral-7b-instruct-v0.2.Q4_K_M.gguf`

**Balanced (8 GB RAM):**
- Llama-2-7B-Chat: https://huggingface.co/TheBloke/Llama-2-7B-Chat-GGUF
  - Download: `llama-2-7b-chat.Q5_K_M.gguf`

**Large (16+ GB RAM):**
- Llama-2-13B-Chat: https://huggingface.co/TheBloke/Llama-2-13B-Chat-GGUF
  - Download: `llama-2-13b-chat.Q4_K_M.gguf`

Save to: `C:\llama.cpp\models\` (or anywhere you prefer)

## Step 3: Start llama.cpp Server

### Windows (PowerShell):
```powershell
# Navigate to llama.cpp directory
cd C:\llama.cpp

# Run the server (adjust paths as needed)
.\llama-server.exe -m ".\models\mistral-7b-instruct-v0.2.Q4_K_M.gguf" --port 8080 -c 2048 --threads 4

# With GPU (if you have NVIDIA GPU and CUDA-enabled build):
.\llama-server.exe -m ".\models\mistral-7b-instruct-v0.2.Q4_K_M.gguf" --port 8080 -c 2048 --n-gpu-layers 35
```

### Linux/Mac:
```bash
cd ~/llama.cpp
./llama-server -m ./models/mistral-7b-instruct-v0.2.Q4_K_M.gguf --port 8080 -c 2048 --threads 4
```

## Step 4: Verify Server is Running

Open a browser and go to: http://localhost:8080/

You should see a simple web interface. If this works, you're ready!

## Step 5: Configure Fallout DnD

Edit `config.toml` in your Fallout DnD directory:

```toml
[llama]
server_url = "http://localhost:8080"  # Should match the port above
```

## Step 6: Run the Game!

```bash
cd FalloutDnD
cargo run --release
```

## Troubleshooting

### "Address already in use"
Another program is using port 8080. Change to a different port:
```bash
# Use port 8081 instead
llama-server.exe ... --port 8081

# Update config.toml:
server_url = "http://localhost:8081"
```

### Server crashes/OOM
Your model is too large for available RAM:
- Download a smaller quantization (Q4_K_M instead of Q5_K_M)
- Reduce context size: `-c 1024` instead of `-c 2048`
- Use a smaller model (7B instead of 13B)

### Slow responses (>10 seconds)
- Reduce `max_tokens` in config.toml to 256
- Use more CPU threads: `--threads 8`
- Enable GPU offload: `--n-gpu-layers 35`
- Use a quantized model (Q4 instead of Q5/Q6)

### GPU not being used
- Ensure you have CUDA/ROCm/Metal-enabled build
- Check with: `llama-server.exe --help | findstr gpu`
- Download GPU-enabled release or rebuild with GPU support

## Performance Tips

### For Best DM Experience:
```bash
# Balanced - Good quality, reasonable speed
llama-server.exe -m mistral-7b-instruct.Q4_K_M.gguf --port 8080 -c 2048 --threads 8

# Fast - Lower quality, very fast
llama-server.exe -m mistral-7b-instruct.Q4_K_S.gguf --port 8080 -c 1536 --threads 8

# Quality - Best responses, slower
llama-server.exe -m mistral-7b-instruct.Q6_K.gguf --port 8080 -c 2048 --threads 8 --n-gpu-layers 35
```

### Recommended config.toml settings:

**For Creative/Fun DM:**
```toml
temperature = 0.9
top_p = 0.95
max_tokens = 400
```

**For Consistent/Serious DM:**
```toml
temperature = 0.6
top_p = 0.9
max_tokens = 300
```

## Testing Your Setup

1. Start llama-server
2. Visit http://localhost:8080/ in browser
3. Type a message in the web interface
4. If you get a response, it's working!
5. Run the Fallout DnD game

## Alternative: Using Ollama (Even Easier!)

If llama.cpp is too complex, use Ollama:

1. Download from: https://ollama.ai/
2. Install and run: `ollama run mistral`
3. In another terminal: `ollama serve`
4. The game should auto-detect Ollama!

Note: You'll need to adjust the AI module for Ollama's API format, or use llama.cpp which is directly supported.
