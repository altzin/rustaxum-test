use axum::{Router, routing::get};

mod health;
mod items;

pub fn create_router() -> Router {
    Router::new()
        .route("/healthz", get(health::health_handler))
        .route("/items", get(items::items_handler))
}
