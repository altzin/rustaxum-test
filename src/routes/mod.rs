use axum::{Router, routing::get};
use sqlx::PgPool;
use utoipa::OpenApi;

mod health;
mod items;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_handler,
        // Add other annotated routes here
    ),
    tags(
        (name = "Health", description = "Service health checks")
    )
)]
pub struct ApiDoc;

pub fn create_router() -> Router<PgPool> {
    Router::new()
        .route("/healthz", get(health::health_handler))
        .route("/items", get(items::items_handler))
}
