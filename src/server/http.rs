use crate::cache::Cache;
use crate::config::Config;
use crate::orchestrator::{InferRequest, Orchestrator};
use crate::providers::ProviderPool;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    orchestrator: Arc<Orchestrator>,
    providers: Arc<ProviderPool>,
}

#[derive(Deserialize)]
struct InferHttpRequest {
    request_id: Option<String>,
    tenant_id: Option<String>,
    project_id: Option<String>,
    task: String,
    prompt: String,
    max_tokens: Option<i32>,
    deadline_ms: Option<i32>,
    strategy: Option<String>,
}

#[derive(Serialize)]
struct InferHttpResponse {
    request_id: String,
    content: String,
    winner_model: String,
    duration_ms: i32,
    from_cache: bool,
    strategy_used: String,
}

#[derive(Serialize)]
struct HealthHttpResponse {
    healthy: bool,
    providers: std::collections::HashMap<String, String>,
    version: String,
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let health_status = state.providers.health_check().await;
    let all_healthy = health_status.values().all(|&v| v);
    
    let providers: std::collections::HashMap<String, String> = health_status
        .into_iter()
        .map(|(k, v)| (k, if v { "healthy".to_string() } else { "unhealthy".to_string() }))
        .collect();
    
    Json(HealthHttpResponse {
        healthy: all_healthy,
        providers,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn infer_handler(
    State(state): State<AppState>,
    Json(payload): Json<InferHttpRequest>,
) -> Result<Json<InferHttpResponse>, (StatusCode, String)> {
    let request_id = payload.request_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    info!("📥 HTTP Infer request: {}", request_id);
    
    let infer_req = InferRequest {
        request_id: request_id.clone(),
        tenant_id: payload.tenant_id.unwrap_or_default(),
        project_id: payload.project_id.unwrap_or_default(),
        task: payload.task,
        prompt: payload.prompt,
        max_tokens: payload.max_tokens.unwrap_or(256),
        deadline_ms: payload.deadline_ms.unwrap_or(1500),
        strategy: payload.strategy,
    };
    
    match state.orchestrator.infer(infer_req).await {
        Ok(result) => Ok(Json(InferHttpResponse {
            request_id: result.request_id,
            content: result.content,
            winner_model: result.winner_model,
            duration_ms: result.duration_ms,
            from_cache: result.from_cache,
            strategy_used: result.strategy_used,
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn serve(
    config: Arc<Config>,
    providers: Arc<ProviderPool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cache = Arc::new(Cache::new(config.cache.ttl_seconds, 10000));
    let orchestrator = Arc::new(Orchestrator::new(config.clone(), providers.clone(), cache));
    
    let state = AppState {
        orchestrator,
        providers,
    };
    
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/infer", post(infer_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    let addr: std::net::SocketAddr = config.server.http_addr.parse()
        .map_err(|e| format!("Invalid address: {}", e))?;
    
    info!("🚀 HTTP server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
