use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

pub fn create_router() -> Router {
    let swagger: Router =
        Into::into(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    Router::new()
        .route("/healthz", get(health::health_handler))
        .route("/items", get(items::items_handler))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(CorsLayer::permissive()) // TODO: should maybe put layers together
        .merge(swagger)
}
