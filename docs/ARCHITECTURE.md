# 🏗️ LLM Pool Architecture

## System Overview

The LLM Pool is a smart orchestration layer that sits between your application and multiple LLM models, providing intelligent routing, caching, and ensemble strategies.

## Request Flow

```
┌─────────────┐
│   Client    │
│ Application │
└──────┬──────┘
       │
       │ HTTP POST /v1/infer
       │ {task, prompt, strategy}
       │
       ▼
┌──────────────────────────────────────────┐
│         LLM Pool Service                 │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  1. HTTP/gRPC Server               │ │
│  │     - Parse request                │ │
│  │     - Extract task & strategy      │ │
│  └────────────┬───────────────────────┘ │
│               │                          │
│  ┌────────────▼───────────────────────┐ │
│  │  2. Orchestrator                   │ │
│  │     - Validate request             │ │
│  │     - Check cache                  │ │
│  │     - Select providers for task    │ │
│  └────────────┬───────────────────────┘ │
│               │                          │
│               │ Cache miss?              │
│               │                          │
│  ┌────────────▼───────────────────────┐ │
│  │  3. Ensemble Engine                │ │
│  │                                    │ │
│  │  ┌──────────────────────────────┐ │ │
│  │  │ Strategy: FASTEST            │ │ │
│  │  │ - Fire first provider        │ │ │
│  │  │ - Hedge after 300ms          │ │ │
│  │  │ - Return first response      │ │ │
│  │  └──────────────────────────────┘ │ │
│  │                                    │ │
│  │  ┌──────────────────────────────┐ │ │
│  │  │ Strategy: VOTING             │ │ │
│  │  │ - Query all providers        │ │ │
│  │  │ - Compare responses          │ │ │
│  │  │ - Pick consensus             │ │ │
│  │  └──────────────────────────────┘ │ │
│  │                                    │ │
│  │  ┌──────────────────────────────┐ │ │
│  │  │ Strategy: JUDGE              │ │ │
│  │  │ - Get multiple candidates    │ │ │
│  │  │ - Use judge model to select  │ │ │
│  │  │ - Return best                │ │ │
│  │  └──────────────────────────────┘ │ │
│  └────────────┬───────────────────────┘ │
│               │                          │
│  ┌────────────▼───────────────────────┐ │
│  │  4. Provider Pool                  │ │
│  │     - Route to Ollama              │ │
│  │     - Handle timeouts              │ │
│  │     - Track health                 │ │
│  └────────────┬───────────────────────┘ │
└───────────────┼──────────────────────────┘
                │
    ┌───────────┼───────────┐
    │           │           │
    ▼           ▼           ▼
┌────────┐  ┌────────┐  ┌────────┐
│ Phi-3  │  │ Llama  │  │ Gemma  │
│  4B    │  │ 3.1 8B │  │  2 9B  │
│        │  │        │  │        │
│ Fast   │  │ Judge  │  │ Rerank │
│ Light  │  │ Smart  │  │ Refine │
└────┬───┘  └────┬───┘  └────┬───┘
     │           │           │
     └───────────┼───────────┘
                 │
                 ▼
        ┌────────────────┐
        │     Ollama     │
        │ localhost:11434│
        └────────────────┘
```

## Component Details

### 1. API Layer (Server)

**Responsibilities:**
- Accept HTTP and gRPC requests
- Parse and validate input
- Convert between protocols
- Return structured responses

**Files:**
- `src/server/http.rs` - REST API with Axum
- `src/server/grpc.rs` - gRPC with Tonic

**Endpoints:**
- `GET /health` - Service health
- `POST /v1/infer` - Inference request

### 2. Orchestrator

**Responsibilities:**
- Request validation (size, deadline, tokens)
- Cache lookup and storage
- Provider selection based on task
- Strategy determination
- Response formatting

**Files:**
- `src/orchestrator.rs`

**Validation Rules:**
- `deadline_ms` ≤ `max_deadline_ms` (1500ms)
- `prompt.len()` ≤ `max_prompt_bytes` (16KB)
- `max_tokens` > 0

### 3. Ensemble Engine

**Responsibilities:**
- Execute selected strategy
- Coordinate multiple providers
- Handle failures and fallbacks
- Aggregate results

**Files:**
- `src/ensemble.rs`

**Strategies:**

#### FASTEST
```
Provider A ──────────────────────> Response (450ms) ✓
                                   Return this!
Provider B ────────────────────────────> (800ms) ✗
                                         Cancelled
```

#### VOTING
```
Provider A ──> "Option 1" ──┐
                            ├──> Compare ──> "Option 1" (2 votes) ✓
Provider B ──> "Option 1" ──┤
                            │
Provider C ──> "Option 2" ──┘
```

#### JUDGE
```
Provider A ──> Candidate 1 ──┐
                             ├──> Judge Model ──> "Best: Candidate 2" ✓
Provider B ──> Candidate 2 ──┤
                             │
Provider C ──> Candidate 3 ──┘
```

### 4. Provider Pool

**Responsibilities:**
- Manage provider instances
- Route requests to Ollama
- Health monitoring
- Timeout handling

**Files:**
- `src/providers/mod.rs` - Pool management
- `src/providers/ollama.rs` - Ollama client

**Provider Configuration:**
```toml
[[providers]]
name = "ollama-phi3-mini"
model = "phi3:mini"
tasks = ["expand_queries", "enrich_metadata"]
weight = 0.6
```

### 5. Cache Layer

**Responsibilities:**
- Store inference results
- Generate cache keys
- Manage TTL
- Eviction policy

**Files:**
- `src/cache.rs`

**Cache Key:**
```
SHA256(task + prompt + max_tokens)
```

**Example:**
```
task: "expand_queries"
prompt: "Generate terms for ambient videos"
max_tokens: 256
→ Key: "a3f5c8d9e2b1..."
→ TTL: 900 seconds (15 min)
```

## Data Flow Example

### Request: Expand Queries (FASTEST)

```
1. Client Request
   POST /v1/infer
   {
     "task": "expand_queries",
     "prompt": "ambient cinematic",
     "max_tokens": 256
   }

2. Orchestrator
   ✓ Validate: OK
   ✓ Cache: MISS
   ✓ Providers: [phi3-mini, llama31-8b]
   ✓ Strategy: FASTEST (from config)

3. Ensemble (FASTEST)
   → Fire phi3-mini immediately
   → Set hedge timer: 300ms
   → phi3-mini responds in 450ms
   → Return phi3-mini result
   → Cancel hedge

4. Cache
   → Store result with TTL 900s

5. Response
   {
     "request_id": "uuid",
     "content": "{\"must_include\": [...]}",
     "winner_model": "phi3:mini",
     "duration_ms": 450,
     "from_cache": false,
     "strategy_used": "FASTEST"
   }
```

### Request: Judge (JUDGE Strategy)

```
1. Client Request
   POST /v1/infer
   {
     "task": "judge",
     "prompt": "Select best: [candidates...]",
     "strategy": "JUDGE"
   }

2. Orchestrator
   ✓ Providers: [llama31-8b, gemma2-9b]
   ✓ Strategy: JUDGE (explicit)

3. Ensemble (JUDGE)
   → Query llama31-8b & gemma2-9b in parallel
   → Get 2 candidates
   → Use llama31-8b as judge
   → Judge selects best candidate
   → Return winner

4. Response
   {
     "winner_model": "llama3.1:8b",
     "duration_ms": 1150,
     "strategy_used": "JUDGE",
     "models_queried": ["llama3.1:8b", "gemma2:9b"]
   }
```

## Task-to-Model Mapping

```
┌─────────────────────┬──────────────┬─────────────────┐
│ Task                │ Strategy     │ Models          │
├─────────────────────┼──────────────┼─────────────────┤
│ expand_queries      │ FASTEST      │ Phi-3 → Llama   │
│ enrich_metadata     │ FASTEST      │ Phi-3           │
│ recovery_plan       │ FASTEST      │ Phi-3 → Llama   │
│ judge               │ JUDGE        │ Llama (judge)   │
│ rerank_candidates   │ JUDGE        │ Llama + Gemma   │
│ site_tactics        │ JUDGE        │ Llama + Gemma   │
└─────────────────────┴──────────────┴─────────────────┘
```

## Configuration Hot-Reload

```
1. User edits llm-pool.toml
   [ensemble]
   default_strategy = "VOTING"  # Changed from FASTEST

2. File watcher detects change
   notify crate → event

3. Config loader
   → Parse new TOML
   → Validate structure
   → If valid: apply
   → If invalid: log error, keep old config

4. Service continues
   → No restart needed
   → Active requests complete with old config
   → New requests use new config
```

## Error Handling

```
Request
  │
  ├─> Validation Error
  │   └─> 400 Bad Request
  │
  ├─> Cache Error
  │   └─> Log warning, continue without cache
  │
  ├─> All Providers Fail
  │   └─> 503 Service Unavailable
  │
  ├─> Deadline Exceeded
  │   └─> 504 Gateway Timeout
  │
  └─> Success
      └─> 200 OK with Answer
```

## Performance Characteristics

### Latency Budget (1500ms total)

```
HTTP Parsing:        ~5ms
Validation:         ~1ms
Cache Lookup:       ~2ms
─────────────────────────
Subtotal:           ~8ms

Provider Call:    450-1200ms  (depends on model)
Ensemble Logic:    ~10-50ms   (depends on strategy)
─────────────────────────
Inference:       460-1250ms

Response Format:    ~5ms
─────────────────────────
Total:          473-1263ms  ✓ Under 1500ms budget
```

### Memory Usage

```
Service Base:        ~50MB
Phi-3 Mini (4B):   ~2.5GB
Llama 3.1 (8B):    ~5.0GB
Gemma 2 (9B):      ~5.5GB
Cache:             ~100MB
─────────────────────────
Total (3 models): ~13.1GB

Recommendation: 16GB RAM minimum, 32GB ideal
```

## Scalability Considerations

### Current (Single Instance)
- Handles: ~10-20 concurrent requests
- Bottleneck: Model inference time
- Throughput: ~5-10 req/sec

### Future (Horizontal Scaling)
- Multiple LLM Pool instances
- Load balancer in front
- Shared Redis cache
- Provider pool per instance

### Future (Vertical Scaling)
- More GPU memory → larger models
- More CPU cores → more concurrent requests
- SSD → faster model loading

## Security Layers (Planned)

```
Request
  │
  ├─> 1. TLS/HTTPS
  │      └─> Encrypted transport
  │
  ├─> 2. API Gateway
  │      └─> Rate limiting, WAF
  │
  ├─> 3. HMAC/JWT Auth
  │      └─> Verify signature
  │
  ├─> 4. Tenant Isolation
  │      └─> Separate quotas
  │
  └─> 5. LLM Pool
         └─> Process request
```

## Monitoring Points

```
┌─────────────────────────────────────────┐
│ Metrics to Track                        │
├─────────────────────────────────────────┤
│ • Request rate (req/sec)                │
│ • Latency (p50, p95, p99)               │
│ • Cache hit rate (%)                    │
│ • Provider health (up/down)             │
│ • Strategy usage (FASTEST/VOTING/...)   │
│ • Model selection frequency             │
│ • Error rate (%)                        │
│ • Timeout rate (%)                      │
└─────────────────────────────────────────┘
```

---

## Summary

The LLM Pool architecture provides:

1. **Flexibility**: Multiple strategies for different use cases
2. **Performance**: Caching and smart routing
3. **Reliability**: Health checks and fallbacks
4. **Observability**: Structured logging and metrics
5. **Maintainability**: Hot-reload and modular design

For implementation details, see the source code in `src/`.
