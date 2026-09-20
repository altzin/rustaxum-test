use axum::Router;
use axum::http::StatusCode;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use std::time::Duration;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

pub trait RouterExt {
    fn with_base_middleware(self) -> Self;
}

impl RouterExt for Router {
    fn with_base_middleware(self) -> Self {
        self.layer(OtelInResponseLayer)
            .layer(OtelAxumLayer::default())
            .layer(CorsLayer::permissive())
            .layer(RequestBodyLimitLayer::new(1024 * 1024))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(10),
            ))
    }
}
