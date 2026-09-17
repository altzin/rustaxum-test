use tracing::{info, instrument};

#[instrument(name = "GET /healthz")]
pub async fn health_handler() -> &'static str {
    info!("Health check was called!");
    "OK"
}
