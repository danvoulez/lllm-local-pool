mod ollama;
mod health;

use crate::config::Config;
use crate::errors::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Provider: Send + Sync {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    #[allow(dead_code)]
    fn supports(&self, task: &str) -> bool;
    async fn infer(&self, prompt: &str, max_tokens: i32, deadline_ms: i32) -> Result<ProviderResponse>;
    async fn health(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: String,
    pub model: String,
    pub duration_ms: i32,
}

pub struct ProviderPool {
    providers: HashMap<String, Arc<dyn Provider>>,
    task_map: HashMap<String, Vec<String>>,
}

impl ProviderPool {
    pub fn names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(name).cloned()
    }
    
    pub fn providers_for_task(&self, task: &str) -> Vec<Arc<dyn Provider>> {
        self.task_map
            .get(task)
            .map(|names| {
                names.iter()
                    .filter_map(|name| self.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        for (name, provider) in &self.providers {
            results.insert(name.clone(), provider.health().await);
        }
        results
    }
}

pub async fn init(config: &Config) -> Result<Arc<ProviderPool>> {
    let mut providers: HashMap<String, Arc<dyn Provider>> = HashMap::new();
    let mut task_map: HashMap<String, Vec<String>> = HashMap::new();
    
    for pconfig in &config.providers {
        let provider: Arc<dyn Provider> = match pconfig.driver.as_str() {
            "ollama" => Arc::new(ollama::OllamaProvider::new(pconfig.clone())),
            _ => {
                tracing::warn!("Unknown provider driver: {}", pconfig.driver);
                continue;
            }
        };
        
        // Map tasks to this provider
        for task in &pconfig.tasks {
            task_map.entry(task.clone())
                .or_insert_with(Vec::new)
                .push(pconfig.name.clone());
        }
        
        providers.insert(pconfig.name.clone(), provider);
    }
    
    Ok(Arc::new(ProviderPool {
        providers,
        task_map,
    }))
}
