# 🚀 START HERE - LLM Pool Quick Setup

Welcome! This is your **LLM Pool service** - a smart orchestration layer for multiple LLM models running on your Mac mini.

## 📋 What You Have

A complete, production-ready service with:
- ✅ **3 LLM models**: Phi-3 (4B), Llama 3.1 (8B), Gemma 2 (9B)
- ✅ **Dual APIs**: gRPC + HTTP/REST
- ✅ **Smart routing**: Tasks automatically use the best model
- ✅ **Ensemble strategies**: FASTEST, VOTING, JUDGE
- ✅ **Caching**: Instant responses for repeated queries
- ✅ **Hot-reload config**: Change settings without restart

## ⚡ Quick Start (3 Steps)

### Step 1: Download Models (15-20 min)

One model is already downloaded. Get the other two:

```bash
./setup-models.sh
```

**Or manually:**
```bash
ollama pull llama3.1:8b    # ~4.7GB
ollama pull gemma2:9b      # ~5.4GB
```

### Step 2: Build & Run (5 min)

```bash
# Make sure Ollama is running
ollama serve &

# Build and start the service
cargo run --release
```

Wait for this output:
```
✅ gRPC server listening on 0.0.0.0:7070
✅ HTTP server listening on 0.0.0.0:7071
```

### Step 3: Test It

Open a new terminal:

```bash
# Health check
curl http://localhost:7071/health | jq .

# Your first AI request
curl -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "expand_queries",
    "prompt": "Generate search terms for ambient videos"
  }' | jq .
```

**First request takes 20-30s** (model loading). After that: <1 second! ⚡

## 📚 Documentation

Choose your path:

### 🎯 **I want to get it running NOW**
→ Follow [CHECKLIST.md](CHECKLIST.md) - Step-by-step with checkboxes

### 📖 **I want to understand what I'm building**
→ Read [SETUP_SUMMARY.md](SETUP_SUMMARY.md) - What was created and why

### 🏗️ **I want to understand the architecture**
→ See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - System design and flow

### 📘 **I want the full documentation**
→ Read [README.md](README.md) - Complete guide with all features

### 🎓 **I want to learn step-by-step**
→ Follow [QUICKSTART.md](QUICKSTART.md) - Detailed tutorial

## 🎯 What Can It Do?

### Task Examples

**1. Expand Search Queries** (Fast - Phi-3)
```bash
curl -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "expand_queries",
    "prompt": "cinematic lounge ambient",
    "max_tokens": 256
  }'
```

**2. Judge Best Option** (Smart - Llama 3.1)
```bash
curl -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "judge",
    "prompt": "Select best: A) 720p 8min studio, B) 1080p 10min cinematic",
    "strategy": "JUDGE"
  }'
```

**3. Enrich Metadata** (Fast - Phi-3)
```bash
curl -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "enrich_metadata",
    "prompt": "Create title and tags for a cinematic video with bokeh"
  }'
```

## 🎨 Available Tasks

| Task | Speed | Model | Use Case |
|------|-------|-------|----------|
| `expand_queries` | ⚡ Fast | Phi-3 | Generate search terms |
| `enrich_metadata` | ⚡ Fast | Phi-3 | Create titles/tags |
| `recovery_plan` | ⚡ Fast | Phi-3 | Error recovery steps |
| `judge` | 🎯 Smart | Llama | Select best option |
| `rerank_candidates` | 🎯 Smart | Llama + Gemma | Reorder results |
| `site_tactics` | 🎯 Smart | Llama + Gemma | Navigation strategy |

## 🔧 Configuration

Edit `llm-pool.toml` to customize:

```toml
[ensemble]
default_strategy = "FASTEST"  # or "VOTING", "JUDGE"

[qos]
max_deadline_ms = 1500        # Max request time
hedge_after_ms = 300          # When to fire backup

[cache]
enabled = true
ttl_seconds = 900             # 15 minutes
```

**The service auto-reloads!** No restart needed. 🔄

## 📊 Performance Expectations

| Metric | Value | Notes |
|--------|-------|-------|
| First request | 20-40s | Model loading |
| Fast tasks (Phi-3) | 200-700ms | expand, enrich |
| Smart tasks (Llama) | 700-1200ms | judge, rerank |
| Cache hits | <10ms | Instant! |
| Memory usage | 8-12GB | With 2-3 models |

## 🆘 Troubleshooting

### "Connection refused"
```bash
# Start Ollama
ollama serve &
```

### "Model not found"
```bash
# Download missing model
ollama pull phi3:mini
ollama pull llama3.1:8b
ollama pull gemma2:9b
```

### First request times out
**This is normal!** Models take 20-30s to load. Try again - next request will be fast.

### Out of memory
- Close other apps
- Use only 2 models (comment out one in config)

## 📁 Project Structure

```
llm-pool/
├── START_HERE.md          ← You are here!
├── CHECKLIST.md           ← Step-by-step setup
├── QUICKSTART.md          ← Detailed tutorial
├── README.md              ← Full documentation
├── SETUP_SUMMARY.md       ← What was built
│
├── llm-pool.toml          ← Configuration (edit this!)
├── prompts/               ← Task templates
│   ├── judge.md
│   ├── rerank.md
│   ├── expand_queries.md
│   ├── recovery_plan.md
│   └── enrich_metadata.md
│
├── src/                   ← Rust source code
│   ├── main.rs
│   ├── orchestrator.rs
│   ├── ensemble.rs
│   ├── providers/
│   └── server/
│
├── setup-models.sh        ← Download all models
└── test-http.sh           ← Test suite
```

## 🎯 Next Steps

1. ✅ **Get it running** - Follow Step 1-3 above
2. 📖 **Read the docs** - Pick from the list above
3. 🧪 **Try examples** - Run `./test-http.sh`
4. 🎨 **Explore tasks** - Try different task types
5. ⚙️ **Tune config** - Edit `llm-pool.toml`
6. 🚀 **Build something** - Integrate into your app!

## 💡 Pro Tips

- **Cache is your friend**: Repeated prompts are instant
- **Start with FASTEST**: It's the quickest strategy
- **Use JUDGE for quality**: When you need the best answer
- **Watch the logs**: See which models are selected
- **Hot-reload config**: Edit and save, no restart needed

## 🎉 Success Checklist

You're ready when you see:

- ✅ Service starts without errors
- ✅ Health check shows all providers healthy
- ✅ First request completes (even if slow)
- ✅ Second request is fast (<1s)
- ✅ Cache works (repeat = instant)

## 📞 Need Help?

1. Check [CHECKLIST.md](CHECKLIST.md) - Common issues
2. Read [QUICKSTART.md](QUICKSTART.md) - Detailed guide
3. Review logs - They're very descriptive
4. Check Ollama: `curl http://localhost:11434/api/tags`

---

## 🚀 Ready? Let's Go!

```bash
# 1. Download models
./setup-models.sh

# 2. Start service
cargo run --release

# 3. Test it (in new terminal)
curl http://localhost:7071/health | jq .
```

**Welcome to LLM Pool!** 🎉

For the full experience, continue to [CHECKLIST.md](CHECKLIST.md) →
