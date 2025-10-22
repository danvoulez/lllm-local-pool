use crate::cache::Cache;
use crate::config::Config;
use crate::orchestrator::{InferRequest, Orchestrator};
use crate::providers::ProviderPool;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

// Include generated proto code
pub mod proto {
    tonic::include_proto!("vvtv.llmpool.v1");
}

use proto::{
    llm_pool_server::{LlmPool, LlmPoolServer},
    Answer, EnsembleDecision, HealthRequest, HealthResponse, Query, Strategy as ProtoStrategy,
    Task as ProtoTask,
};

pub struct LLMPoolService {
    orchestrator: Arc<Orchestrator>,
    providers: Arc<ProviderPool>,
}

#[tonic::async_trait]
impl LlmPool for LLMPoolService {
    async fn infer(&self, request: Request<Query>) -> Result<Response<Answer>, Status> {
        let query = request.into_inner();
        
        info!("📥 gRPC Infer request: {}", query.request_id);
        
        let task_str = task_to_string(query.task());
        let strategy_str = strategy_to_string(query.strategy());
        
        let infer_req = InferRequest {
            request_id: query.request_id.clone(),
            tenant_id: query.tenant_id,
            project_id: query.project_id,
            task: task_str,
            prompt: query.prompt,
            max_tokens: query.max_tokens,
            deadline_ms: query.deadline_ms,
            strategy: Some(strategy_str),
        };
        
        let result = self.orchestrator.infer(infer_req).await?;
        
        let answer = Answer {
            request_id: result.request_id,
            content: result.content,
            winner_model: result.winner_model,
            duration_ms: result.duration_ms,
            from_cache: result.from_cache,
            decision: Some(EnsembleDecision {
                strategy_used: string_to_strategy(&result.strategy_used) as i32,
                models_queried: result.models_queried,
                model_scores: vec![],
                reason: "Ensemble decision".to_string(),
            }),
            meta: std::collections::HashMap::new(),
        };
        
        Ok(Response::new(answer))
    }
    
    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        let health_status = self.providers.health_check().await;
        
        let all_healthy = health_status.values().all(|&v| v);
        
        let provider_status: std::collections::HashMap<String, String> = health_status
            .into_iter()
            .map(|(k, v)| (k, if v { "healthy".to_string() } else { "unhealthy".to_string() }))
            .collect();
        
        Ok(Response::new(HealthResponse {
            healthy: all_healthy,
            provider_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }
}

pub async fn serve(config: Arc<Config>, providers: Arc<ProviderPool>) -> Result<(), Box<dyn std::error::Error>> {
    let cache = Arc::new(Cache::new(config.cache.ttl_seconds, 10000));
    let orchestrator = Arc::new(Orchestrator::new(config.clone(), providers.clone(), cache));
    
    let service = LLMPoolService {
        orchestrator,
        providers,
    };
    
    let addr: std::net::SocketAddr = config.server.grpc_addr.parse()
        .map_err(|e| format!("Invalid address: {}", e))?;
    
    info!("🚀 gRPC server starting on {}", addr);
    
    Server::builder()
        .add_service(LlmPoolServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}

fn task_to_string(task: ProtoTask) -> String {
    match task {
        ProtoTask::ExpandQueries => "expand_queries",
        ProtoTask::SiteTactics => "site_tactics",
        ProtoTask::RerankCandidates => "rerank_candidates",
        ProtoTask::Judge => "judge",
        ProtoTask::RecoveryPlan => "recovery_plan",
        ProtoTask::EnrichMetadata => "enrich_metadata",
        _ => "unknown",
    }
    .to_string()
}

fn strategy_to_string(strategy: ProtoStrategy) -> String {
    match strategy {
        ProtoStrategy::Fastest => "FASTEST",
        ProtoStrategy::Voting => "VOTING",
        ProtoStrategy::Weighted => "WEIGHTED",
        ProtoStrategy::Consensus => "CONSENSUS",
        ProtoStrategy::Judge => "JUDGE",
        _ => "FASTEST",
    }
    .to_string()
}

fn string_to_strategy(s: &str) -> ProtoStrategy {
    match s.to_uppercase().as_str() {
        "FASTEST" => ProtoStrategy::Fastest,
        "VOTING" => ProtoStrategy::Voting,
        "WEIGHTED" => ProtoStrategy::Weighted,
        "CONSENSUS" => ProtoStrategy::Consensus,
        "JUDGE" => ProtoStrategy::Judge,
        _ => ProtoStrategy::Unspecified,
    }
}
