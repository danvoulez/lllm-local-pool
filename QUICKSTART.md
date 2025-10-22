# 🚀 Quick Start Guide

Follow these steps to get your LLM Pool service up and running.

## Step 1: Download Models

The first model (Phi-3 mini) is already downloaded. Now get the remaining two:

```bash
# Download Llama 3.1 8B (~4.7GB)
ollama pull llama3.1:8b

# Download Gemma 2 9B (~5.4GB)
ollama pull gemma2:9b
```

**Or use the setup script:**
```bash
./setup-models.sh
```

This will take 10-20 minutes depending on your internet connection.

## Step 2: Verify Ollama is Running

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# If not running, start it:
ollama serve &
```

## Step 3: Build the Service

```bash
# Build in release mode (optimized)
cargo build --release

# This will take a few minutes on first build
```

## Step 4: Start the Service

```bash
# Run the service
cargo run --release

# Or directly:
./target/release/llm-pool
```

You should see:
```
🚀 Starting LLM Pool Service
✅ Configuration loaded
✅ Providers initialized: ["ollama-phi3-mini", "ollama-llama31-8b", "ollama-gemma2-9b"]
✅ gRPC server listening on 0.0.0.0:7070
✅ HTTP server listening on 0.0.0.0:7071
👀 Watching config file for changes
```

## Step 5: Test the Service

Open a new terminal and run:

```bash
# Quick health check
curl http://localhost:7071/health | jq .

# Run full test suite
./test-http.sh
```

## Step 6: Make Your First Request

```bash
curl -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "expand_queries",
    "prompt": "Generate search terms for ambient cinematic videos",
    "max_tokens": 256
  }' | jq .
```

## Expected Response

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "{\n  \"must_include\": [\"ambient\", \"cinematic\"],\n  ...",
  "winner_model": "phi3:mini",
  "duration_ms": 450,
  "from_cache": false,
  "strategy_used": "FASTEST"
}
```

## Next Steps

1. **Explore Tasks**: Try different tasks (judge, rerank_candidates, enrich_metadata)
2. **Adjust Config**: Edit `llm-pool.toml` to tune strategies and weights
3. **Check Prompts**: Review `prompts/*.md` for task-specific templates
4. **Monitor Performance**: Watch logs for latency and model selection

## Troubleshooting

### Models not found
```bash
ollama list  # Check installed models
ollama pull phi3:mini  # Re-download if needed
```

### Port already in use
Edit `llm-pool.toml` and change the port numbers:
```toml
[server]
grpc_addr = "0.0.0.0:7072"
http_addr = "0.0.0.0:7073"
```

### Build errors
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Ollama not responding
```bash
# Restart Ollama
pkill ollama
ollama serve &
```

## Performance Tips

- **First request is slow**: Models need to load into memory (~30s)
- **Subsequent requests are fast**: Models stay loaded
- **Cache helps**: Identical prompts return instantly
- **Mac mini RAM**: 16GB can handle 2 models, 32GB handles all 3 comfortably

## What's Next?

- Read the full [README.md](README.md) for architecture details
- Check [prompts/](prompts/) for task-specific prompt engineering
- Explore the [configuration options](llm-pool.toml)
- Review the [EPIC document](🌐%20EPIC%20L%20—%20LLM%20Pool%20como%20Serviço%20(desac.md) for the full roadmap

Enjoy your LLM Pool! 🎉
