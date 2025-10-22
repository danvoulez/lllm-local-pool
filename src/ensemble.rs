use crate::config::Config;
use crate::errors::{LLMPoolError, Result};
use crate::providers::{Provider, ProviderResponse};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone)]
pub enum Strategy {
    Fastest,
    Voting,
    Weighted,
    Consensus,
    Judge,
}

impl Strategy {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "FASTEST" => Strategy::Fastest,
            "VOTING" => Strategy::Voting,
            "WEIGHTED" => Strategy::Weighted,
            "CONSENSUS" => Strategy::Consensus,
            "JUDGE" => Strategy::Judge,
            _ => Strategy::Fastest,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EnsembleResult {
    pub response: ProviderResponse,
    pub strategy_used: Strategy,
    pub models_queried: Vec<String>,
    pub model_scores: Vec<f32>,
    pub reason: String,
}

pub struct Ensemble {
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl Ensemble {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
    
    pub async fn execute(
        &self,
        strategy: Strategy,
        providers: Vec<Arc<dyn Provider>>,
        prompt: &str,
        max_tokens: i32,
        deadline_ms: i32,
    ) -> Result<EnsembleResult> {
        if providers.is_empty() {
            return Err(LLMPoolError::EnsembleError("No providers available".to_string()));
        }
        
        match strategy {
            Strategy::Fastest => self.fastest(providers, prompt, max_tokens, deadline_ms).await,
            Strategy::Voting => self.voting(providers, prompt, max_tokens, deadline_ms).await,
            Strategy::Weighted => self.weighted(providers, prompt, max_tokens, deadline_ms).await,
            Strategy::Consensus => self.consensus(providers, prompt, max_tokens, deadline_ms).await,
            Strategy::Judge => self.judge(providers, prompt, max_tokens, deadline_ms).await,
        }
    }
    
    async fn fastest(
        &self,
        providers: Vec<Arc<dyn Provider>>,
        prompt: &str,
        max_tokens: i32,
        deadline_ms: i32,
    ) -> Result<EnsembleResult> {
        info!("🏃 FASTEST strategy with {} providers", providers.len());
        
        // For now, just use the first provider
        // TODO: Implement hedged requests and race condition
        let provider = &providers[0];
        let response = provider.infer(prompt, max_tokens, deadline_ms).await?;
        
        Ok(EnsembleResult {
            response: response.clone(),
            strategy_used: Strategy::Fastest,
            models_queried: vec![response.model.clone()],
            model_scores: vec![1.0],
            reason: "First provider to respond".to_string(),
        })
    }
    
    async fn voting(
        &self,
        providers: Vec<Arc<dyn Provider>>,
        prompt: &str,
        max_tokens: i32,
        deadline_ms: i32,
    ) -> Result<EnsembleResult> {
        info!("🗳️  VOTING strategy with {} providers", providers.len());
        
        // Query all providers in parallel
        let mut tasks = Vec::new();
        for provider in providers.iter() {
            let p = provider.clone();
            let prompt = prompt.to_string();
            tasks.push(tokio::spawn(async move {
                p.infer(&prompt, max_tokens, deadline_ms).await
            }));
        }
        
        let mut responses = Vec::new();
        for task in tasks {
            if let Ok(Ok(resp)) = task.await {
                responses.push(resp);
            }
        }
        
        if responses.is_empty() {
            return Err(LLMPoolError::EnsembleError("All providers failed".to_string()));
        }
        
        // Simple voting: pick the most common response (or first for now)
        let winner = responses[0].clone();
        
        Ok(EnsembleResult {
            response: winner.clone(),
            strategy_used: Strategy::Voting,
            models_queried: responses.iter().map(|r| r.model.clone()).collect(),
            model_scores: vec![1.0; responses.len()],
            reason: "Voting consensus".to_string(),
        })
    }
    
    async fn weighted(
        &self,
        providers: Vec<Arc<dyn Provider>>,
        prompt: &str,
        max_tokens: i32,
        deadline_ms: i32,
    ) -> Result<EnsembleResult> {
        info!("⚖️  WEIGHTED strategy with {} providers", providers.len());
        // For now, fallback to fastest
        self.fastest(providers, prompt, max_tokens, deadline_ms).await
    }
    
    async fn consensus(
        &self,
        providers: Vec<Arc<dyn Provider>>,
        prompt: &str,
        max_tokens: i32,
        deadline_ms: i32,
    ) -> Result<EnsembleResult> {
        info!("🤝 CONSENSUS strategy with {} providers", providers.len());
        // For now, fallback to voting
        self.voting(providers, prompt, max_tokens, deadline_ms).await
    }
    
    async fn judge(
        &self,
        providers: Vec<Arc<dyn Provider>>,
        prompt: &str,
        max_tokens: i32,
        deadline_ms: i32,
    ) -> Result<EnsembleResult> {
        info!("⚖️  JUDGE strategy with {} providers", providers.len());
        
        // Get multiple candidates
        let voting_result = self.voting(providers, prompt, max_tokens, deadline_ms).await?;
        
        // TODO: Implement actual judge logic with a separate model
        // For now, return the voting result
        Ok(EnsembleResult {
            strategy_used: Strategy::Judge,
            ..voting_result
        })
    }
}
