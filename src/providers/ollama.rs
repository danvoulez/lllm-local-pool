use super::{Provider, ProviderResponse};
use crate::config::ProviderConfig;
use crate::errors::{LLMPoolError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub struct OllamaProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: i32,
    temperature: f32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(
                config.timeout_ms.unwrap_or(5000) as u64
            ))
            .build()
            .expect("Failed to build HTTP client");
        
        Self { config, client }
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn supports(&self, task: &str) -> bool {
        self.config.tasks.iter().any(|t| t == task)
    }
    
    async fn infer(&self, prompt: &str, max_tokens: i32, _deadline_ms: i32) -> Result<ProviderResponse> {
        let start = Instant::now();
        
        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                num_predict: max_tokens,
                temperature: 0.3,
            },
        };
        
        let url = format!("{}/api/generate", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMPoolError::ProviderError(format!("Ollama request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(LLMPoolError::ProviderError(
                format!("Ollama returned status: {}", response.status())
            ));
        }
        
        let ollama_resp: OllamaResponse = response
            .json()
            .await
            .map_err(|e| LLMPoolError::ProviderError(format!("Failed to parse Ollama response: {}", e)))?;
        
        let duration_ms = start.elapsed().as_millis() as i32;
        
        Ok(ProviderResponse {
            content: ollama_resp.response,
            model: self.config.model.clone(),
            duration_ms,
        })
    }
    
    async fn health(&self) -> bool {
        let url = format!("{}/api/tags", self.config.base_url);
        self.client.get(&url).send().await.is_ok()
    }
}
