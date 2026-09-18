use tracing::{info, instrument};

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    ),
    tag = "Health"
)]
#[instrument(name = "GET /healthz")]
pub async fn health_handler() -> &'static str {
    info!("Health check was called!");
    "OK"
}
