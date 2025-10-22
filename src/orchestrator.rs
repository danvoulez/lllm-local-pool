use crate::cache::Cache;
use crate::config::Config;
use crate::ensemble::{Ensemble, Strategy};
use crate::errors::{LLMPoolError, Result};
use crate::providers::ProviderPool;
use std::sync::Arc;
use tracing::info;

pub struct Orchestrator {
    config: Arc<Config>,
    providers: Arc<ProviderPool>,
    ensemble: Ensemble,
    cache: Arc<Cache>,
}

#[derive(Debug, Clone)]
pub struct InferRequest {
    pub request_id: String,
    #[allow(dead_code)]
    pub tenant_id: String,
    #[allow(dead_code)]
    pub project_id: String,
    pub task: String,
    pub prompt: String,
    pub max_tokens: i32,
    pub deadline_ms: i32,
    pub strategy: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InferResponse {
    pub request_id: String,
    pub content: String,
    pub winner_model: String,
    pub duration_ms: i32,
    pub from_cache: bool,
    pub strategy_used: String,
    pub models_queried: Vec<String>,
}

impl Orchestrator {
    pub fn new(
        config: Arc<Config>,
        providers: Arc<ProviderPool>,
        cache: Arc<Cache>,
    ) -> Self {
        let ensemble = Ensemble::new(config.clone());
        Self {
            config,
            providers,
            ensemble,
            cache,
        }
    }
    
    pub async fn infer(&self, req: InferRequest) -> Result<InferResponse> {
        info!("🎯 Orchestrating request: {} (task: {})", req.request_id, req.task);
        
        // Validate request
        self.validate(&req)?;
        
        // Check cache
        if self.config.cache.enabled {
            if let Some(cached) = self.cache.get(&req.task, &req.prompt, req.max_tokens).await {
                info!("💾 Cache hit for request: {}", req.request_id);
                return Ok(InferResponse {
                    request_id: req.request_id,
                    content: cached.content,
                    winner_model: cached.model,
                    duration_ms: 0,
                    from_cache: true,
                    strategy_used: "CACHE".to_string(),
                    models_queried: vec![],
                });
            }
        }
        
        // Get providers for this task
        let providers = self.providers.providers_for_task(&req.task);
        if providers.is_empty() {
            return Err(LLMPoolError::EnsembleError(
                format!("No providers available for task: {}", req.task)
            ));
        }
        
        // Determine strategy
        let strategy_name = req.strategy
            .or_else(|| self.config.ensemble.strategy_by_task.get(&req.task).cloned())
            .unwrap_or_else(|| self.config.ensemble.default_strategy.clone());
        
        let strategy = Strategy::from_str(&strategy_name);
        
        // Execute ensemble
        let result = self.ensemble.execute(
            strategy,
            providers,
            &req.prompt,
            req.max_tokens,
            req.deadline_ms,
        ).await?;
        
        // Cache the result
        if self.config.cache.enabled {
            self.cache.set(
                &req.task,
                &req.prompt,
                req.max_tokens,
                &result.response.content,
                &result.response.model,
            ).await;
        }
        
        Ok(InferResponse {
            request_id: req.request_id,
            content: result.response.content,
            winner_model: result.response.model,
            duration_ms: result.response.duration_ms,
            from_cache: false,
            strategy_used: strategy_name,
            models_queried: result.models_queried,
        })
    }
    
    fn validate(&self, req: &InferRequest) -> Result<()> {
        // Check deadline
        if req.deadline_ms > self.config.qos.max_deadline_ms {
            return Err(LLMPoolError::InvalidQuery(
                format!("Deadline {}ms exceeds max {}ms", 
                    req.deadline_ms, self.config.qos.max_deadline_ms)
            ));
        }
        
        // Check prompt size
        if req.prompt.len() > self.config.qos.max_prompt_bytes {
            return Err(LLMPoolError::InvalidQuery(
                format!("Prompt size {} exceeds max {} bytes",
                    req.prompt.len(), self.config.qos.max_prompt_bytes)
            ));
        }
        
        // Check max_tokens
        if req.max_tokens <= 0 {
            return Err(LLMPoolError::InvalidQuery(
                "max_tokens must be positive".to_string()
            ));
        }
        
        Ok(())
    }
}
