# 🌐 LLM Pool Service

[![CI](https://github.com/danvoulez/lllm-local-pool/workflows/CI/badge.svg)](https://github.com/danvoulez/lllm-local-pool/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

A high-performance, multi-model LLM orchestration service with ensemble strategies, intelligent routing, and caching. Built with Rust for maximum performance and reliability.

## Features

- **Multi-Model Support**: Phi-3 Mini (4B), Llama 3.1 (8B), Gemma 2 (9B)
- **Ensemble Strategies**: FASTEST, VOTING, WEIGHTED, CONSENSUS, JUDGE
- **Dual Protocol**: gRPC and HTTP/REST APIs
- **Smart Caching**: In-memory cache with configurable TTL
- **Hot-Reload Config**: Update configuration without restart
- **Task-Specific Routing**: Automatic model selection per task
- **Observability**: Structured logging and metrics (planned)

## Quick Start

### 1. Install Ollama and Models

```bash
# Install Ollama (if not already installed)
brew install ollama

# Start Ollama service
ollama serve &

# Download required models (this will take some time)
chmod +x setup-models.sh
./setup-models.sh
```

### 2. Build the Service

```bash
# Build the Rust project
cargo build --release

# Or run in development mode
cargo run
```

### 3. Start the Service

```bash
# Using the release build
./target/release/llm-pool

# Or with custom config
RUST_LOG=info ./target/release/llm-pool
```

The service will start:
- **gRPC** on `0.0.0.0:7070`
- **HTTP** on `0.0.0.0:7071`

### 4. Test the Service

```bash
# Health check
curl http://localhost:7071/health

# Inference request
curl -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "expand_queries",
    "prompt": "Generate search terms for cinematic lounge videos",
    "max_tokens": 256,
    "deadline_ms": 1500
  }'
```

## Configuration

Edit `llm-pool.toml` to customize:

- **Providers**: Add/remove models, adjust weights
- **Strategies**: Set default and per-task strategies
- **QoS**: Deadlines, hedging, circuit breakers
- **Cache**: TTL, capacity, key fields

The service will automatically reload when you save changes to the config file.

## Task Types

| Task | Description | Default Strategy | Models |
|------|-------------|------------------|--------|
| `expand_queries` | Expand search terms | FASTEST | Phi-3, Llama 3.1 |
| `site_tactics` | Navigation strategies | JUDGE | Llama 3.1, Gemma 2 |
| `rerank_candidates` | Reorder results | JUDGE | Llama 3.1, Gemma 2 |
| `judge` | Select best option | JUDGE | Llama 3.1, Gemma 2 |
| `recovery_plan` | Error recovery | FASTEST | Phi-3, Llama 3.1 |
| `enrich_metadata` | Generate metadata | FASTEST | Phi-3 |

## Ensemble Strategies

- **FASTEST**: Return first response (with optional hedging)
- **VOTING**: Query multiple models, pick consensus
- **WEIGHTED**: Weight responses by provider performance
- **CONSENSUS**: Require high agreement between models
- **JUDGE**: Use a judge model to select best response

## Prompts

Task-specific prompt templates are in the `prompts/` directory:
- `judge.md` - Selection criteria
- `rerank.md` - Reordering logic
- `expand_queries.md` - Query expansion
- `recovery_plan.md` - Error recovery
- `enrich_metadata.md` - Metadata generation

## Architecture

```
┌─────────────┐     ┌─────────────┐
│   gRPC      │     │    HTTP     │
│  :7070      │     │   :7071     │
└──────┬──────┘     └──────┬──────┘
       │                   │
       └───────┬───────────┘
               │
        ┌──────▼──────┐
        │ Orchestrator│
        └──────┬──────┘
               │
        ┌──────▼──────┐
        │  Ensemble   │
        │  Strategies │
        └──────┬──────┘
               │
    ┌──────────┼──────────┐
    │          │          │
┌───▼───┐  ┌──▼───┐  ┌──▼───┐
│Phi-3  │  │Llama │  │Gemma │
│ 4B    │  │3.1 8B│  │2 9B  │
└───────┘  └──────┘  └──────┘
```

## Development

### Project Structure

```
llm-pool/
├── src/
│   ├── main.rs           # Entry point
│   ├── config.rs         # Configuration & hot-reload
│   ├── orchestrator.rs   # Request orchestration
│   ├── ensemble.rs       # Ensemble strategies
│   ├── providers/        # Provider implementations
│   │   ├── mod.rs
│   │   ├── ollama.rs     # Ollama provider
│   │   └── health.rs     # Health checks
│   ├── server/           # API servers
│   │   ├── grpc.rs       # gRPC server
│   │   └── http.rs       # HTTP server
│   ├── cache.rs          # Caching layer
│   ├── qos/              # Quality of Service
│   ├── security/         # Auth & rate limiting
│   └── telemetry.rs      # Observability
├── proto/                # Protocol buffers
├── prompts/              # Task prompts
├── llm-pool.toml         # Configuration
└── Cargo.toml            # Dependencies
```

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release --target aarch64-apple-darwin
```

## Performance Targets

- **expand/enrich**: 200–700 ms (Phi-3 4B)
- **judge/rerank**: 700–1200 ms (Llama 3.1 8B)
- **batch operations**: 1.5–3.0 s (Llama 8B / Gemma 9B)

## Roadmap

- [x] L1: Core Service (API, config, hot-reload)
- [x] L2: Ollama Provider
- [x] L3: Basic Ensemble (FASTEST, VOTING)
- [ ] L3: Advanced Ensemble (WEIGHTED, CONSENSUS, JUDGE)
- [ ] L3: Hedged Requests & Circuit Breaker
- [ ] L4: HMAC/JWT Authentication
- [ ] L4: Rate Limiting
- [ ] L6: Redis Cache Support
- [ ] L7: Prometheus Metrics
- [ ] L7: OpenTelemetry Tracing
- [ ] L8: SDKs (Rust, Python, TypeScript)
- [ ] L9: Docker & Kubernetes Deployment

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
