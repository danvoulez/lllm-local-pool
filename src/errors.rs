use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLMPoolError {
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Deadline exceeded: {0}ms")]
    DeadlineExceeded(i32),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Ensemble error: {0}")]
    EnsembleError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Circuit breaker open for provider: {0}")]
    CircuitBreakerOpen(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<LLMPoolError> for tonic::Status {
    fn from(err: LLMPoolError) -> Self {
        match err {
            LLMPoolError::InvalidQuery(msg) => {
                tonic::Status::invalid_argument(msg)
            }
            LLMPoolError::DeadlineExceeded(ms) => {
                tonic::Status::deadline_exceeded(format!("Deadline {}ms exceeded", ms))
            }
            LLMPoolError::AuthError(msg) => {
                tonic::Status::unauthenticated(msg)
            }
            LLMPoolError::RateLimitExceeded => {
                tonic::Status::resource_exhausted("Rate limit exceeded")
            }
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, LLMPoolError>;
