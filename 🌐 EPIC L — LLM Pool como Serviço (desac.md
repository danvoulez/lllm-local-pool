🌐 EPIC L — LLM Pool como Serviço (desacoplado + seguro)
L1 — Core Service (API, servidor e config)

Objetivo: Serviço stateless com gRPC e HTTP compatíveis, hot-reload de config.

Tarefas

 Contrato gRPC

 proto/vvtv/llmpool/v1/llmpool.proto com Query, Answer, Task, Strategy, EnsembleDecision, Health.

 request_id, tenant_id, project_id, deadline_ms, max_tokens, meta.

 Servidor

 llm-pool/src/server/grpc.rs (tonic) e http.rs (axum/actix) espelhando o proto.

 router.rs: roteia /v1/infer → orchestrator::infer().

 config.rs: carregar llm-pool.toml com hot-reload (notify).

 Orquestrador

 orchestrator.rs: valida Query, enriquece deadline, chama ensemble, formata Answer.

 Enforce limites: max_deadline_ms, max_prompt_bytes, max_tokens.

Arquivos

proto/vvtv/llmpool/v1/llmpool.proto

llm-pool/src/server/{grpc.rs,http.rs,router.rs}

llm-pool/src/{config.rs,orchestrator.rs,errors.rs}

Testes

 Round-trip gRPC/HTTP equivalentes.

 Validação de bounds (deadline acima do limite → 400/INVALID_ARGUMENT).

 Health check OK.

CI

 Job gera stubs (Rust/Python/TS) e verifica drift do proto.

Docs

 docs/LLMPOOL_API.md (exemplos gRPC/HTTP).

L2 — Providers (drivers plugáveis)

Objetivo: Falar com engines reais: Ollama, vLLM, HTTP genérico.

Tarefas

 Trait Provider

 providers/mod.rs: Provider { name(), supports(task), infer(query) }.

 Ollama

 provider_ollama.rs: POST /api/generate, mapear model, prompt, max_tokens.

 vLLM

 provider_vllm.rs: POST /generate (OpenAI-like).

 HTTP genérico

 provider_http.rs: schema configurável (mapeia campos via template).

 Healthcheck & backoff

 health.rs: ping periódico, marca provider degraded/unhealthy.

Arquivos

llm-pool/src/providers/{mod.rs,ollama.rs,vllm.rs,http.rs,health.rs}

Testes

 Mocks dos 3 providers.

 Health flapping → backoff exponencial.

CI

 Contract tests (respostas sintéticas por task).

Docs

 docs/PROVIDERS.md (campos suportados, exemplos de config).

L3 — Ensemble & QoS (latência, hedge, breaker)

Objetivo: Estratégias FASTEST/VOTING/WEIGHTED/CONSENSUS/JUDGE, hedged requests, circuit breaker.

Tarefas

 Ensemble

 ensemble.rs: implementar 5 estratégias.

 Similaridade (VOTING/CONSENSUS): minhash/levenshtein simples, opcional embed.

 WEIGHTED: pesos do config + win-rate histórico.

 JUDGE: concatenar candidatos, prompt curto de julgamento, prazo menor; fallback → VOTING.

 Hedged requests

 Disparar 2ª tentativa (outro provider) após hedge_after_ms no FASTEST.

 Circuit Breaker

 Por provider e por task (janela 50, fail rate >10% abre 5min).

 Timeout orchestration

 Dividir deadline_ms entre providers e judge com margem (e.g., 70/30).

Arquivos

llm-pool/src/{ensemble.rs,qos/{hedge.rs,breaker.rs}}

Testes

 Fallback chain: JUDGE→VOTING→WEIGHTED→FASTEST.

 Hedged reduz p95 em cenário com um provider lento.

 Breaker abre/fecha corretamente.

CI

 Tests de latência simulada, asserts de p95.

Docs

 docs/ENSEMBLE.md (quando usar cada estratégia; custos/latência).

L4 — Tenancy, Auth & Rate-Limit

Objetivo: Multi-tenant seguro, com API keys (HMAC) e/ou JWT, rate-limit e quotas.

Tarefas

 Auth HMAC

 Middleware HTTP/gRPC: valida X-Api-Key, X-Timestamp, X-Nonce, X-Signature.

 Proteção de replay (timestamp ±5min, nonce em Redis/mem 10min).

 JWT (opcional)

 Validação JWKS, claims tenant_id, project_id, scopes:["infer"].

 Rate-limit

 Por {tenant_id,project_id}, RPS + burst (token bucket).

 Quotas (opcional)

 Contabilizar tokens/dia por tenant (se providers reportarem).

Arquivos

llm-pool/src/security/{hmac.rs,jwt.rs,ratelimit.rs,nonce_cache.rs}

Testes

 HMAC válido/inválido, replay detectado.

 429 ao estourar rate-limit.

 JWT expirado/sem escopo.

CI

 Security tests rodam isolados; lint de headers obrigatórios.

Docs

 docs/SECURITY.md (como gerar chaves, rotacionar, exemplos de assinatura).

L5 — Configuração & Hot-Reload

Objetivo: Arquivo único (llm-pool.toml) governando providers, ensemble, QoS, tenants.

Tarefas

 Schema

 [server], [qos], [ensemble], [breaker], [cache], [tenancy.*], [[providers]], [judge].

 Hot-reload

 Watcher aplica mudanças sem reiniciar; validação e rollback se inválido.

 Defaults seguros

 max_deadline_ms, hedge_after_ms, fail_rate, window_size.

Arquivos

llm-pool/src/config.rs

configs/llm-pool.example.toml

Testes

 Reload preserva conexões em andamento.

 Rejeita config inválida com log claro.

Docs

 docs/CONFIG.md (todos campos + exemplos).

L6 — Cache & Idempotência

Objetivo: Reduzir latência/custos com cache por {task,prompt,max_tokens} e idempotência por request_id.

Tarefas

 Cache

 Driver memória/Redis; TTL por task; “do not cache” opcional por meta.

 Idempotência

 Memória/Redis guarda Answer por request_id (curto prazo, p.ex. 10–15 min).

Arquivos

llm-pool/src/cache/{mod.rs,redis.rs,memory.rs}

Testes

 Cache hit ratio sobe em prompts repetidos.

 Idempotente: mesma resposta para request_id repetido.

Docs

 docs/CACHE.md (chave, TTLs, invalidação).

L7 — Observabilidade (métricas, logs, tracing)

Objetivo: Visibilidade total multi-tenant.

Tarefas

 Métricas Prometheus

 llmpool_infer_duration_ms{tenant,project,task,strategy,provider}

 llmpool_timeouts_total{…}

 llmpool_ensemble_decision_total{strategy}

 llmpool_provider_win_rate{provider,task}

 llmpool_cache_hit_ratio

 llmpool_breaker_open_total

 Logs estruturados

 JSON com request_id, tenant, project, task, strategy, winner_model, duration_ms, from_cache.

 Tracing

 OpenTelemetry; propagar trace_id do cliente.

Arquivos

llm-pool/src/telemetry/{metrics.rs,logging.rs,tracing.rs}

grafana/dashboards/llm-pool.json

Testes

 Smoke: endpoints de /metrics carregam; painel renderiza.

Docs

 docs/OBSERVABILITY.md (painéis, KPIs, alertas recomendados).

L8 — SDKs (Rust, Python, TypeScript)

Objetivo: Clientes leves e idiomáticos.

Tarefas

 Rust: llmpool_sdk

 Client::infer(Query) com retries leves e mapeamento gRPC/HTTP.

 Python: llmpool (httpx/grpc)

 Client.infer(task, strategy, prompt, ...) + HMAC helper.

 TypeScript: @vvtv/llmpool-client

 Fetch com HMAC/JWT; tipagens a partir do proto.

Arquivos

sdk/rust/llmpool_sdk/*

sdk/python/llmpool/*

sdk/ts/llmpool-client/*

Testes

 Cada SDK com server fake (contract tests).

 Exemplos prontos (planner/curator samples).

Docs

 docs/SDKs.md (uso, autenticação, exemplos por task).

L9 — Deploy & Gateway (exposição segura na internet)

Objetivo: Publicar o pool com chaves e WAF sem expor o backend cru.

Tarefas

 Container & Helm

 Dockerfile multi-stage; charts/llm-pool (replicas, HPA, liveness, readiness).

 Gateway/WAF

 Kong/Cloudflare/NGINX: TLS, rate-limit, validação de HMAC/JWT, mTLS gateway↔pool.

 (Opcional) IP allowlist; limitar size de payload.

 Secrets & rotação

 Duas chaves ativas por cliente (A/B); runbook de rotação.

 SRE runbook

 Quedas de provider, breaker aberto, saturação, emergência (kill switch).

Arquivos

deploy/Dockerfile

deploy/charts/llm-pool/*

gateway/kong.yaml ou cloudflare/terraform/*

RUNBOOK_llmpool.md

Testes

 E2E público (latência p95, rate-limit, HMAC válido/ inválido).

 mTLS habilitado entre gateway↔pool.

Docs

 docs/EXPOSURE.md (topologia, headers, exemplos de assinatura).

📎 Extras úteis (Qualidade de vida)

 Feature flag no cliente: backend = "local" | "pool".

 Kill-switch por task/tenant (desabilitar JUDGE ou RECOVERY_PLAN rápido).

 Policy de custos por tenant: max_tokens, max_deadline, max_parallel_providers.

🔢 Ordem sugerida (com paralelismo)

L1 (Core) + L5 (Config) + L7 (básico de métricas)

L2 (Providers)

L3 (Ensemble & QoS)

L4 (Auth/Rate-limit)

L6 (Cache/Idempotência)

L8 (SDKs)

L9 (Deploy+Gateway)

✅ Critérios de aceite finais

p95 Infer (FASTEST, 2 providers, hedge 350ms) ≤ 900ms em rede local/gateway.

Timeout rate < 10% por task/tenant (breaker atuando).

Cache hit ≥ 30% em prompts repetidos (workload real).

Determinismo em VOTING/CONSENSUS/JUDGE para a mesma entrada.

Segurança: HMAC/JWT verificados, rate-limit ativo, replay bloqueado.

Observabilidade: métricas e logs com tenant/project/task/strategy/provider.

SDKs publicados e exemplos rodando (planner/curator/autopilot).

Se quiser, converto esse plano em issues do GitHub (com labels, dependências) e já te mando os scaffolds de pastas/arquivos para começar a abrir PRs.

Você disse:
Qgora a lista de quais modelos usar e como, quando
O ChatGPT disse:
Pensou por 27s

Perfeito, Dan. Abaixo está a curadoria dos modelos para o seu LLM Pool — quais usar, quando, como e por quê — já pensando no seu SLA (respostas <1,2s nos hooks online) e nos modos batch (auditoria/relatórios).

🎛️ Matriz “Tarefa → Modelo → Como usar”

Regras gerais
• Hook on-line (≤1200 ms): priorize 8–14B quantizado (INT4/8) com hedged requests.
• Judge/Rerank de alta precisão (até 2,5s): suba para 14–32B (ou MoE).
• Batch noturno (sem pressa): modelos >30B/MoE.
• Servir via vLLM (OpenAI-compatible) ou Ollama como fallback local. 
GitHub
+1

1) expand_queries (ampliar termos de busca)

Primário (rápido, barato): Phi-3 Medium 14B (ou 4–7B se GPU apertada). Excelente em síntese/expansão curta; roda liso quantizado. 
Ollama

Alternativa com mais contexto: Llama 3.1 8B (ou 70B em batch). Boa compreensão geral e janelas longas. 
Ollama
+1

Quando usar: todo ciclo de procura/planejamento; TTL de cache alta (mesma query se repete).

2) site_tactics (estratégia de navegação/scrape humano-simulada)

Primário (robusto a instruções longas): Llama 3.1 8B/70B (on-line 8B; 70B para gerar “playbooks” em batch). 
Ollama
+1

Alternativa com raciocínio tático: Qwen3 14–32B (boa leitura de páginas e instruções multi-etapas; Apache 2.0 nas variantes abertas). Use em batch para gerar macro-estratégias por domínio. 
Wikipedia

3) rerank_candidates (ordenar 10–12 candidatos)

Primário (on-line ≤1,2s): Gemma 2 9B/27B — ótimos “rerankers” gerais; 9B para on-line, 27B para prime time (2ª passada). 
Ollama

“Taste engine” em batch: Llama 3.1 70B ou Qwen3 32B para rerank com justificativa (logs do Curador). 
The Guardian
+1

4) judge (LLM-as-judge entre respostas)

Primário (curto e assertivo): Llama 3.1 8B/70B com prompt de julgamento enxuto; limite max_tokens ≤ 128. 
Ollama
+1

Juiz “esperto” para raciocínio: Qwen3 14–32B (modo batch em revisões diárias). 
Wikipedia

5) recovery_plan (quando algo falha em runtime)

Primário (resposta sucinta): Phi-3 Medium 14B (foca em plano de ação curto). 
Ollama

Secundário (cenários complexos): Gemma 2 27B (mais “calmo” em instruções estruturadas). 
Ollama

6) enrich_metadata (títulos/tags/resumos)

Primário (latência baixa): Phi-3 Mini/Medium para tagging rápido. 
Ollama

Alternativa criativa controlada: Llama 3.1 8B (ou 70B em lote para copy premium do dia). 
Ollama
+1

Observação sobre Qwen (família 2.5/3): versões Qwen3 8/14/32B são fortes em raciocínio e disponíveis sob Apache 2.0 para self-hosting, boas para judge, rerank e site tactics em batch. 
Wikipedia

⚙️ Perfis do Pool (o “cardápio”)
Perfil	Estratégia	Modelos (ordem de hedge)	Uso
FAST-LOW	FASTEST + hedge 300 ms	Phi-3 14B → Llama 3.1 8B	hooks on-line (expand/enrich)
JUDGE-MID	JUDGE → VOTING	Llama 3.1 8B (judge) + Gemma 2 9B (candidatos)	rerank e judge on-line
QUALITY-HIGH	CONSENSUS	Llama 3.1 70B + Qwen3 32B	rerank/judge em batch (curadoria)
COST-SAVER	WEIGHTED	Phi-3 14B (peso 0.6) + Gemma 2 9B (0.4)	tarefas massivas (tags)
ROBUST	VOTING → JUDGE	Gemma 2 27B + Llama 3.1 8B + Phi-3 14B	quando variância de resposta é alta

Servidores: preferir vLLM (alto throughput, batching contínuo, quantizações INT4/8, API OpenAI-like). Manter Ollama como dev/local fallback e prototipagem. 
GitHub
+1

🖥️ Hardware & Deployment

Nós on-line (baixa latência):

1× A4000/RTX 6000 Ada 48–96 GB → Llama 3.1 8B / Gemma 2 9B / Phi-3 14B INT4/8.

2× GPUs + vLLM com continuous batching para p95 estável. 
GitHub

Nós batch (noite):

1× A100 80GB / H100 para Llama 3.1 70B / Qwen3 32B e Mixtral-class (se adotado).

Quantização: INT4/8 (AWQ/GPTQ/FP8 onde suportado no vLLM). 
GitHub

🧭 Roteamento (quando escolher cada um)

Entrada curta (<1k tokens), prazo ≤1200 ms:
→ FAST-LOW (Phi-3 14B → hedge Llama 3.1 8B).

Disputa entre 3–5 candidatos:
→ JUDGE-MID (Llama 3.1 8B como juiz).

Especial “prime time”/coleções do dia (offline):
→ QUALITY-HIGH (Llama 3.1 70B + Qwen3 32B consenso).

Milhares de metadados:
→ COST-SAVER (Phi-3 14B + Gemma 2 9B, pesados no cache).

Quando um provedor degrada (timeouts >10%):
→ Circuit breaker abre; cair para Ollama local do mesmo tamanho (ou provider alternativo via vLLM). 
GitHub

🧪 Prompting e limites práticos

JUDGE: prompt fixo, few-shot 1–2 linhas, max_tokens ≤128 (corta latência).

RERANK: passe apenas features essenciais (duração, tags, resumo 1–2 frases).

EXPAND: delimite output schema (lista JSON de termos + pesos).

RECOVERY: peça até 3 passos com owner checks (evita delírio).

ENRICH: templates com campos curtos (titulo ≤60 chars, 4–8 tags).

🧰 Licenças & praticidades

Llama 3.1 (Meta) — open p/ uso local; tamanhos 8B–405B disponíveis via Ollama/vLLM. 
Ollama
+1

Gemma 2/3 (Google) — boa para 1 GPU; atenção a licença (restringe alguns usos comerciais; verifique clausulado). 
theverge.com

Phi-3 (Microsoft) — small/medium muito eficientes. 
Ollama

Qwen3 (Alibaba/Qwen) — ampla família Apache 2.0 em 2025 (8B–32B); forte em raciocínio. 
Wikipedia

Observação: modelos e licenças evoluem rápido; manter inventário mensal e bench interno em seu tráfego (latência, win-rate por tarefa).

🧩 Exemplo de pool profile (llm-pool.toml)
[qos]
max_deadline_ms = 1500
hedge_after_ms  = 300

[ensemble]
default = "FASTEST"

[[providers]]
name = "vllm-phi3-14b"
type = "vllm"
model = "microsoft/phi-3-medium-128k-instruct"
tasks = ["expand_queries","enrich_metadata","recovery_plan"]

[[providers]]
name = "vllm-llama31-8b"
type = "vllm"
model = "meta-llama/llama-3.1-8b-instruct"
tasks = ["judge","rerank_candidates","site_tactics","expand_queries"]

[[providers]]
name = "vllm-gemma2-9b"
type = "vllm"
model = "google/gemma-2-9b-it"
tasks = ["rerank_candidates","enrich_metadata"]

[[providers]]
name = "vllm-qwen3-32b"
type = "vllm"
model = "qwen/qwen3-32b-instruct"
tasks = ["judge","rerank_candidates"]         # batch

[judge]
strategy = "JUDGE"
fallback = "VOTING"

TL;DR (escolhas rápidas)

On-line: Phi-3 14B + Llama 3.1 8B (rápidos); Gemma 2 9B para rerank. 
Ollama

Batch qualidade: Llama 3.1 70B + Qwen3 32B (consenso/judge). 
The Guardian
+1

Infra: servir por vLLM (OpenAI-like API, batching, quantizações); Ollama para dev e fallback. 
GitHub
+1

Se quiser, já te entrego os arquivos de config iniciais com esses perfis e um script de bench interno (latência/win-rate por tarefa) pra validar no seu tráfego real.

Você disse:
Decide pra mim o que fazerm so lembrando que tem que rodar no macmini e nao em 2 gpu
O ChatGPT disse:
Pensou por 5s

Perfeito — decisão fechada para rodar 100% no Mac mini (Apple Silicon, 16–32 GB RAM, sem GPU NVIDIA) usando Ollama (Metal) como runtime. Foquei em latência <1,2 s nos hooks on-line, footprint de memória razoável e qualidade suficiente para judge/rerank.

✅ Escolha final (pronta pra usar)
Pool on-line (tempo-real)

FAST: Phi-3 Mini 4B (Instruct) — super leve e rápido; ótimo p/ expandir buscas, enriquecer metadados e planos de recuperação curtinhos.

GENERAL: Llama 3.1 8B Instruct — “cérebro geral” do pool, bom equilíbrio em quase tudo; ideal p/ judge curto e site tactics sucintas.

RERANK/TASTE: Gemma 2 9B It (ou Qwen 2.5/3 7B Instruct se preferir Apache 2.0) — melhora rerank e refinamento editorial com pouco custo extra.

Por que esses?
• Todos rodam bem no Mac mini via Metal/llama.cpp (Ollama) com quantização Q4_K_M.
• 4B + 8B + ~9B dá diversidade pro ensemble sem matar RAM.

Estratégias por tarefa
Tarefa (hook)	Estratégia	Modelos	Observações
expand_queries	FASTEST + hedge 300 ms	Phi-3 4B → Llama 3.1 8B	cache alto (TTL 10–30 min)
site_tactics	JUDGE → VOTING	Llama 3.1 8B (juiz) + Gemma 2 9B	prompts curtos, max_tokens ≤128
rerank_candidates	JUDGE	Llama 3.1 8B (juiz)	passe só features essenciais
recovery_plan	FASTEST	Phi-3 4B → Llama 3.1 8B	3 passos no máx.
enrich_metadata	FASTEST	Phi-3 4B	títulos ≤60 chars, 4–8 tags

Batch noturno (opcional, mesmo Mac mini): usar Llama 3.1 8B e Gemma 2 9B com deadline maior (2–3 s) para gerar memos/editoriais. Não recomendo modelos >10–13B no Mac mini — custo/latência sobem muito.

📦 Instalação (Ollama)
# 1) Instalar Ollama (macOS)
brew install ollama
ollama serve &

# 2) Baixar modelos (quantização default do Ollama já é ótima no Mac)
ollama pull phi3:mini        # microsoft/phi-3-mini
ollama pull llama3.1:8b      # meta-llama/llama-3.1-8b-instruct
ollama pull gemma2:9b        # google/gemma-2-9b-it
# (opcional) alternativa Apache 2.0:
# ollama pull qwen2.5:7b-instruct


Dica RAM: se seu Mac mini for 16 GB, rode 2 modelos ativos (Phi-3 4B + Llama 3.1 8B) e carregue o 3º on-demand (Gemma/Qwen) só para rerank/judge.

⚙️ llm-pool.toml (pronto pro seu serviço)
[server]
listen_addr = "0.0.0.0:7070"
enable_grpc = true
enable_http = true

[qos]
max_deadline_ms = 1500
hedge_after_ms  = 300
max_prompt_bytes = 16384
max_tokens_default = 256

[ensemble]
default_strategy = "FASTEST"
strategy_by_task.expand_queries   = "FASTEST"
strategy_by_task.site_tactics     = "JUDGE"
strategy_by_task.rerank_candidates= "JUDGE"
strategy_by_task.recovery_plan    = "FASTEST"
strategy_by_task.enrich_metadata  = "FASTEST"

[breaker]
fail_rate   = 0.10
window_size = 50
open_cooldown_ms = 300000

[cache]
enabled = true
driver  = "memory"
ttl_seconds = 900
key_fields = ["task","prompt","max_tokens"]

[[providers]]
name   = "ollama-phi3-mini"
driver = "ollama"
base_url = "http://127.0.0.1:11434"
model  = "phi3:mini"
tasks  = ["expand_queries","enrich_metadata","recovery_plan"]
weight = 0.6

[[providers]]
name   = "ollama-llama31-8b"
driver = "ollama"
base_url = "http://127.0.0.1:11434"
model  = "llama3.1:8b"
tasks  = ["judge","site_tactics","rerank_candidates","expand_queries","recovery_plan"]
weight = 1.0

[[providers]]
name   = "ollama-gemma2-9b"
driver = "ollama"
base_url = "http://127.0.0.1:11434"
model  = "gemma2:9b"
tasks  = ["rerank_candidates","judge"]
weight = 0.8

[judge]
model_provider = "ollama-llama31-8b"
max_tokens = 128
deadline_ms = 700
fallback_strategy = "VOTING"

🔧 Prompts & limites (para segurar p95)

JUDGE/RE-RANK: prompt fixo ultra curto + schema de saída (JSON com winner_idx, why), max_tokens ≤128.

EXPAND: peça lista JSON de termos + pesos (no máx. 16 termos).

RECOVERY: “diga até 3 passos”, sempre com guardrails.

ENRICH: templates (título ≤60, 4–8 tags).

🧪 Sanity check (targets no Mac mini)

expand/enrich: 200–700 ms (Phi-3 4B)

judge/rerank simples: 700–1200 ms (Llama 3.1 8B)

Batch noturno (memos/editoriais): 1,5–3,0 s (Llama 8B / Gemma 9B)

TL;DR

Use Phi-3 4B (rápido), Llama 3.1 8B (juiz geral) e Gemma 2 9B (rerank fino). Tudo via Ollama no Mac mini, com estratégias FASTEST/JUDGE, prompts curtos e cache. Se preferir licença Apache full, troque Gemma 2 9B por Qwen 2.5/3 7B.

Se quiser, te mando também os prompts prontos (JUDGE/RERANK/EXPAND/RECOVERY/ENRICH) em .md pra colar no repositório.

Você disse:
Manda
O ChatGPT disse:

maravilha — seguem os 5 prompts prontos (em arquivos .md) para colar no seu repositório (/prompts). Estão otimizados para rodar no Mac mini com Ollama: curtos, determinísticos, com schemas JSON simples (fáceis de parsear) e guardrails fortes.

Dica: mantenha max_tokens em 128 (JUDGE/RERANK) e 256 (EXPAND/RECOVERY/ENRICH).
Todos os prompts exigem saída estritamente em JSON (sem texto extra).

/prompts/judge.md
# JUDGE — seleção determinística do melhor candidato

Você é um **árbitro conciso**. Recebe 3–12 candidatos e retorna **apenas 1 vencedor** com justificativa breve.

## Regras
- Siga o **critério** informado em `decision_criterion`.
- **Nunca** invente fatos, números, títulos ou URLs.
- **Recuse** candidatos que aparentem violar políticas (ex.: conteúdo que sugira menores, coerção, ilegalidade). Se todos violarem, retorne `"winner_id": null` com `"reason"`.
- Priorize **qualidade visual** (≥720p), **duração-alvo** (ex.: ~10min), **variedade estética** e **coerência** com o slot.
- Saída **somente JSON**, sem texto adicional.

## Entrada (JSON)
```json
{
  "decision_criterion": "choose best for prime-time balance of novelty + duration ~10min + visual quality",
  "slot_context": {"slot_id": "86", "recent_themes": ["studio","warm-pastel"], "target_duration_sec": 600},
  "candidates": [
    {
      "id":"c1",
      "title":"…",
      "tags":["…"],
      "duration_sec": 604,
      "resolution":"1080p",
      "novelty_score":0.61,
      "palette":["warm","pastel"],
      "qc":{"vmaf":91,"lufs":-14.2},
      "policy_flags":{"suspected_minor":false,"violence":false,"drm_locked":false}
    }
    // … c2..cN
  ]
}

Saída (JSON)
{
  "winner_id": "c1",
  "confidence": 0.74,
  "reason": "1080p, ~10min, contrasta temas recentes; melhor vmaf; variedade estética.",
  "quality_notes": ["bom áudio (-14 LUFS)", "paleta distinta do histórico"],
  "policy": {"action": "allow", "notes": []}
}

Diretrizes de avaliação (resumo)

Aderência ao critério do slot; 2) Qualidade técnica (resolução/VMAF/LUFS); 3) Variedade vs. últimos slots; 4) Duração próxima da meta; 5) Tags coerentes; 6) Sinalizações de política (reprovar se suspeito).


---

## `/prompts/rerank.md`

```md
# RERANK — reordenar 10–12 candidatos para o slot

Você recebe uma lista de candidatos e deve retornar **apenas a ordem final** e um racional curto.

## Regras
- Otimize para **ritmo** (evitar repetição), **duração** (~10min), **qualidade técnica** e **variedade de paleta/tema**.
- **Não remova** itens; **reordene**. Se algum for inviável, marque em `policy.flags`.
- Saída **somente JSON**.

## Entrada (JSON)
```json
{
  "slot_context": {
    "slot_id":"86",
    "recent_slots":[
      {"id":"r11","palette":["warm","pastel"],"tags":["studio"],"duration_sec":602},
      {"id":"r12","palette":["warm"],"tags":["studio","closeup"],"duration_sec":593}
    ],
    "target_duration_sec": 600
  },
  "candidates":[
    {"id":"c1","duration_sec":604,"resolution":"1080p","palette":["cool"],"tags":["cinematic"],"qc":{"vmaf":90}},
    {"id":"c2","duration_sec":720,"resolution":"720p","palette":["warm"],"tags":["studio"],"qc":{"vmaf":87}}
    // … c3..c12
  ]
}

Saída (JSON)
{
  "ranking": ["c1","c5","c3","c2","c7","c8","c6","c11","c10","c4","c9","c12"],
  "diversity_gain": 0.07,
  "reasons": [
    "abre com paleta fria p/ contrastar histórico quente",
    "dur. próximas de 10min nas 3 primeiras posições",
    "evita 2 'studio warm' consecutivos"
  ],
  "policy": {
    "flags": [],
    "notes": []
  }
}

Heurística (aplique em silêncio)

Penalize sequências com mesma paleta dos últimos 2–3 slots.

Bonifique 1080p e VMAF ≥ 90.

Prefira 600±45s; durations muito fora vão para baixo.


---

## `/prompts/expand_queries.md`

```md
# EXPAND_QUERIES — ampliar termos de busca (vídeos e músicas)

Gere **listas curtas e objetivas** para busca web. Foco em **diversidade controlada** e **baixo custo**.

## Regras
- Não use termos explícitos; prefira **descrições neutras/estéticas**.
- Inclua **negativos** para evitar duplicatas, baixa qualidade e termos proibidos.
- Sugira **sites genéricos** (quando aplicável), nunca invente URLs.
- Saída **somente JSON**.

## Entrada (JSON)
```json
{
  "seed_terms": ["cinematic lounge", "ambient slow burn"],
  "media": "video|music",
  "lang": ["pt","en"],
  "duration_target_sec": 600,
  "negatives_hint": ["cam rip","compilation lowres","duplicate"],
  "constraints": {"min_resolution":"720p","license_hint":"non-drm"}
}

Saída (JSON)
{
  "must_include": ["cinematic lounge", "ambient slow burn"],
  "optional": ["downtempo", "moody color grading", "studio soft light", "bokeh"],
  "negatives": ["cam rip","compilation","blurry","watermark","reaction","shorts"],
  "duration_range_sec": [540, 720],
  "site_hints": ["site:archive.org", "site:soundcloud.com", "site:bandcamp.com", "site:vimeo.com"],
  "language_mix": {"pt": 0.4, "en": 0.6},
  "notes": ["priorize '1080p' nas queries de vídeo", "adicionar -shorts -tiktok"]
}

Observação

Reutilize resultados com TTL de 10–30 min no cache do pool.


---

## `/prompts/recovery_plan.md`

```md
# RECOVERY_PLAN — plano de ação em até 3 passos

Quando algo falhar (ex.: baixa qualidade técnica, buffer crítico, fonte sem HD), descreva **um plano sucinto e seguro**.

## Regras
- Máximo **3 passos**. Sem gambiarras, sem burlar ToS, sem instruções invasivas.
- Se o cenário indicar DRM/ABR indisponível, **recomende abortar** com razão.
- Saída **somente JSON**.

## Entrada (JSON)
```json
{
  "incident": "low_vmaf_playout",
  "context": {
    "slot_id":"86",
    "current_asset":{"id":"a17","vmaf":81,"resolution":"720p","bitrate_kbps":1200},
    "buffer_minutes": 0.7,
    "alternatives":[{"id":"b02","vmaf":92,"resolution":"1080p"}]
  }
}

Saída (JSON)
{
  "severity": "medium",
  "steps": [
    {"action":"swap_asset","args":{"use_id":"b02"},"guardrails":["verify vmaf>=90","duration within 540..720"]},
    {"action":"bump_music_ratio","args":{"next_slots":2,"mood":"downtempo"},"guardrails":["keep queue FIFO within bucket"]},
    {"action":"note_incident","args":{"tag":"slow-HD-source"},"guardrails":["do not blacklist domain without audit"]}
  ],
  "rollback": {"if":"swap fails","do":"revert_to a17 and inject emergency loop 1 item"},
  "eta_min": 3,
  "reason": "asset b02 atende 1080p/VMAF92; troca rápida reduz risco de churn"
}

Notas

Use preferência por alternativas já baixadas.

Se tudo abaixo do limiar, recomende emergency loop temporário.


---

## `/prompts/enrich_metadata.md`

```md
# ENRICH_METADATA — título, tags e resumo curto

Crie metadados **claros e neutros**, focados em estética/ritmo (sem termos explícitos).

## Regras
- Título ≤ **60** caracteres; resumo ≤ **120** caracteres.
- 4–8 **tags** objetivas (cores, clima, estilo).
- Se suspeitar de violação de política, sinalize em `content_flags`.
- Saída **somente JSON**.

## Entrada (JSON)
```json
{
  "asset": {
    "id":"c1",
    "features": {"palette":["cool","night"],"style":["cinematic","slow burn"],"duration_sec": 604, "resolution":"1080p"},
    "audio":{"lufs": -14.1},
    "tags":["studio","bokeh"]
  }
}

Saída (JSON)
{
  "title": "Cinematic Night — slow burn em luz fria",
  "one_liner": "Texturas frias e ritmo suave com bokeh discreto e acabamento 1080p.",
  "tags": ["cinematic","cool","night","slow-burn","bokeh","1080p","studio"],
  "mood": "downtempo",
  "content_flags": [],
  "notes": ["mantém coerência com duração alvo ~10min"]
}

Observações

Evite palavras sensacionalistas; privilegie estética e técnica.


---

### Como usar (resumo rápido)

- **JUDGE/RERANK**: chame com `max_tokens=128`, `temperature≈0.2`, **deadline 700–1200ms**.  
- **EXPAND/RECOVERY/ENRICH**: `max_tokens=256`, `temperature≈0.3`, **deadline 800–1500ms**.  
- Sempre force **saída JSON** — o pool já valida e rejeita fora do schema.

Se quiser, eu também te mando uma **versão .zip** com esses arquivos já nomeados e uma **pasta `examples/`** com entradas/saídas de teste (posso gerar aqui mesmo).
::contentReference[oaicite:0]{index=0}