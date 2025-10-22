use moka::future::Cache as MokaCache;
use std::time::Duration;

#[derive(Clone)]
pub struct CachedResponse {
    pub content: String,
    pub model: String,
}

pub struct Cache {
    store: MokaCache<String, CachedResponse>,
}

impl Cache {
    pub fn new(ttl_seconds: u64, max_capacity: u64) -> Self {
        let store = MokaCache::builder()
            .time_to_live(Duration::from_secs(ttl_seconds))
            .max_capacity(max_capacity)
            .build();
        
        Self { store }
    }
    
    pub async fn get(&self, task: &str, prompt: &str, max_tokens: i32) -> Option<CachedResponse> {
        let key = self.make_key(task, prompt, max_tokens);
        self.store.get(&key).await
    }
    
    pub async fn set(&self, task: &str, prompt: &str, max_tokens: i32, content: &str, model: &str) {
        let key = self.make_key(task, prompt, max_tokens);
        self.store.insert(key, CachedResponse {
            content: content.to_string(),
            model: model.to_string(),
        }).await;
    }
    
    fn make_key(&self, task: &str, prompt: &str, max_tokens: i32) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(task.as_bytes());
        hasher.update(prompt.as_bytes());
        hasher.update(max_tokens.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
