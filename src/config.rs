use crate::errors::{LLMPoolError, Result};
use notify::{Watcher, RecursiveMode, Event};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub qos: QoSConfig,
    pub ensemble: EnsembleConfig,
    pub breaker: BreakerConfig,
    pub cache: CacheConfig,
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub judge: JudgeConfig,
    #[serde(default)]
    pub tenancy: HashMap<String, TenantConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_grpc_addr")]
    pub grpc_addr: String,
    #[serde(default = "default_http_addr")]
    pub http_addr: String,
    #[serde(default = "default_true")]
    pub enable_grpc: bool,
    #[serde(default = "default_true")]
    pub enable_http: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QoSConfig {
    #[serde(default = "default_max_deadline")]
    pub max_deadline_ms: i32,
    #[serde(default = "default_hedge_after")]
    pub hedge_after_ms: i32,
    #[serde(default = "default_max_prompt_bytes")]
    pub max_prompt_bytes: usize,
    #[serde(default = "default_max_tokens")]
    pub max_tokens_default: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnsembleConfig {
    #[serde(default = "default_strategy")]
    pub default_strategy: String,
    #[serde(default)]
    pub strategy_by_task: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BreakerConfig {
    #[serde(default = "default_fail_rate")]
    pub fail_rate: f32,
    #[serde(default = "default_window_size")]
    pub window_size: usize,
    #[serde(default = "default_cooldown")]
    pub open_cooldown_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_cache_driver")]
    pub driver: String,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
    #[serde(default)]
    pub key_fields: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub name: String,
    pub driver: String,
    pub base_url: String,
    pub model: String,
    pub tasks: Vec<String>,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub timeout_ms: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct JudgeConfig {
    pub model_provider: Option<String>,
    #[serde(default = "default_judge_max_tokens")]
    pub max_tokens: i32,
    #[serde(default = "default_judge_deadline")]
    pub deadline_ms: i32,
    #[serde(default = "default_fallback_strategy")]
    pub fallback_strategy: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TenantConfig {
    pub api_key: String,
    pub rate_limit_rps: u32,
    pub rate_limit_burst: u32,
}

// Defaults
fn default_grpc_addr() -> String { "0.0.0.0:7070".to_string() }
fn default_http_addr() -> String { "0.0.0.0:7071".to_string() }
fn default_true() -> bool { true }
fn default_max_deadline() -> i32 { 1500 }
fn default_hedge_after() -> i32 { 300 }
fn default_max_prompt_bytes() -> usize { 16384 }
fn default_max_tokens() -> i32 { 256 }
fn default_strategy() -> String { "FASTEST".to_string() }
fn default_fail_rate() -> f32 { 0.10 }
fn default_window_size() -> usize { 50 }
fn default_cooldown() -> u64 { 300000 }
fn default_cache_driver() -> String { "memory".to_string() }
fn default_ttl() -> u64 { 900 }
fn default_weight() -> f32 { 1.0 }
fn default_judge_max_tokens() -> i32 { 128 }
fn default_judge_deadline() -> i32 { 700 }
fn default_fallback_strategy() -> String { "VOTING".to_string() }

pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| LLMPoolError::ConfigError(format!("Failed to read config: {}", e)))?;
    
    let config: Config = toml::from_str(&content)
        .map_err(|e| LLMPoolError::ConfigError(format!("Failed to parse config: {}", e)))?;
    
    validate(&config)?;
    Ok(config)
}

fn validate(config: &Config) -> Result<()> {
    if config.providers.is_empty() {
        return Err(LLMPoolError::ConfigError("No providers configured".to_string()));
    }
    
    if config.qos.max_deadline_ms <= 0 {
        return Err(LLMPoolError::ConfigError("max_deadline_ms must be positive".to_string()));
    }
    
    Ok(())
}

pub fn watch<P: AsRef<Path>>(path: P, initial: Config) -> Result<tokio::task::JoinHandle<()>> {
    let path = path.as_ref().to_path_buf();
    let config = Arc::new(RwLock::new(initial));
    
    let handle = tokio::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        }).expect("Failed to create watcher");
        
        watcher.watch(&path, RecursiveMode::NonRecursive)
            .expect("Failed to watch config file");
        
        info!("👀 Watching config file for changes: {:?}", path);
        
        while let Some(_event) = rx.recv().await {
            match load(&path) {
                Ok(new_config) => {
                    let mut cfg = config.write().await;
                    *cfg = new_config;
                    info!("♻️  Configuration reloaded successfully");
                }
                Err(e) => {
                    error!("❌ Failed to reload config: {}", e);
                }
            }
        }
    });
    
    Ok(handle)
}
