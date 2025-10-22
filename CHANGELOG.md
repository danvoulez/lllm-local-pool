# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-21

### Added
- Initial release of LLM Pool service
- **Core Service (L1)**
  - gRPC server with Tonic on port 7070
  - HTTP REST API with Axum on port 7071
  - Configuration management with hot-reload support
  - Request orchestration and validation
  - Protocol buffer definitions for service contract

- **Provider Integration (L2)**
  - Ollama provider implementation
  - Support for multiple models (Phi-3, Llama 3.1, Gemma 2)
  - Provider health checking
  - Provider pool management

- **Ensemble Strategies (L3)**
  - FASTEST strategy - first response wins
  - VOTING strategy - consensus-based selection
  - WEIGHTED strategy (basic implementation)
  - CONSENSUS strategy (basic implementation)
  - JUDGE strategy (basic implementation)
  - Task-specific model routing

- **Caching (L6)**
  - In-memory cache with Moka
  - SHA256-based cache key generation
  - Configurable TTL (default 15 minutes)
  - Cache hit/miss tracking

- **Observability (L7)**
  - Structured logging with tracing-subscriber
  - Request ID tracking with UUID
  - JSON log output
  - Performance metrics in responses

- **Configuration**
  - TOML-based configuration
  - Hot-reload support with file watching
  - Task-to-model mapping
  - QoS parameters (deadlines, hedging, circuit breaker)

- **Documentation**
  - Comprehensive README with examples
  - Quick start guide (QUICKSTART.md)
  - Setup checklist (CHECKLIST.md)
  - Architecture documentation (docs/ARCHITECTURE.md)
  - Project status tracking (PROJECT_STATUS.md)
  - Setup summary (SETUP_SUMMARY.md)

- **Prompt Templates**
  - Judge selection template
  - Reranking template
  - Query expansion template
  - Recovery plan template
  - Metadata enrichment template

- **Development Tools**
  - Model setup script (setup-models.sh)
  - HTTP API test suite (test-http.sh)
  - Build script for proto compilation
  - Git ignore configuration

### Performance
- First request: 20-40s (model loading)
- Fast tasks (Phi-3): 200-700ms
- Smart tasks (Llama 3.1): 700-1200ms
- Cache hits: <10ms (instant)
- Memory usage: 8-12GB with 2-3 models loaded

### Known Limitations
- Hedged requests not yet implemented (stub only)
- Circuit breaker not yet implemented (stub only)
- HMAC/JWT authentication not yet implemented (stub only)
- Rate limiting not yet implemented (stub only)
- Prometheus metrics not yet implemented (stub only)
- OpenTelemetry tracing not yet implemented (stub only)
- Redis cache backend not yet implemented
- Client SDKs not yet available

### Supported Platforms
- macOS (Apple Silicon and Intel)
- Linux (x86_64, ARM64)
- Requires Rust 1.70+ and Ollama

## [Unreleased]

### Planned Features
- Additional LLM providers (OpenAI, Anthropic, Cohere)
- Advanced ensemble strategies with win-rate tracking
- Redis cache backend for distributed deployments
- Full Prometheus metrics implementation
- OpenTelemetry distributed tracing
- HMAC and JWT authentication
- Per-tenant rate limiting
- Circuit breaker with failure tracking
- Hedged request implementation
- Client SDKs (Python, TypeScript, Rust, Go)
- Docker and Kubernetes deployment manifests
- Monitoring dashboards (Grafana)
- Performance benchmarks
- Load testing suite

---

## Version History

- **0.1.0** (2025-10-21) - Initial release with core functionality

[0.1.0]: https://github.com/yourusername/llm-pool/releases/tag/v0.1.0
[Unreleased]: https://github.com/yourusername/llm-pool/compare/v0.1.0...HEAD
