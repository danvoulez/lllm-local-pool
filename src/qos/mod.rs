pub mod hedge;
pub mod breaker;

// QoS (Quality of Service) implementations
// - Hedged requests: send duplicate requests after timeout
// - Circuit breaker: prevent cascading failures
