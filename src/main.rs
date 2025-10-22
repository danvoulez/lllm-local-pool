mod config;
mod errors;
mod orchestrator;
mod providers;
mod server;
mod ensemble;
mod qos;
mod security;
mod cache;
mod telemetry;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize telemetry
    telemetry::init()?;

    info!("🚀 Starting LLM Pool Service");

    // Load configuration
    let config = Arc::new(config::load("llm-pool.toml")?);
    info!("✅ Configuration loaded");

    // Start config hot-reload watcher
    let config_handle = config::watch("llm-pool.toml", (*config).clone())?;

    // Initialize providers
    let providers = providers::init(&config).await?;
    info!("✅ Providers initialized: {:?}", providers.names());

    // Start servers
    let grpc_config = config.clone();
    let grpc_providers = providers.clone();
    let grpc_handle = tokio::spawn(async move {
        let _ = server::grpc::serve(grpc_config, grpc_providers).await;
    });
    
    let http_config = config.clone();
    let http_providers = providers.clone();
    let http_handle = tokio::spawn(async move {
        let _ = server::http::serve(http_config, http_providers).await;
    });

    info!("✅ gRPC server listening on {}", config.server.grpc_addr);
    info!("✅ HTTP server listening on {}", config.server.http_addr);

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Shutdown signal received");
        }
        _ = grpc_handle => {},
        _ = http_handle => {},
    }

    config_handle.abort();
    info!("👋 LLM Pool Service stopped");
    Ok(())
}
