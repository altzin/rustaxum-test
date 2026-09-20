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

use crate::metrics::AppMetrics;
use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
};
use opentelemetry::KeyValue;
use std::time::Instant;

pub async fn track_metrics(
    State(metrics): State<AppMetrics>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();

    // Extract the matched route template (e.g., "/users/:id") or fallback to the raw path
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|mp| mp.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_owned());

    // Execute the actual route handler
    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        KeyValue::new("http.method", method),
        KeyValue::new("http.route", path),
        KeyValue::new("http.status_code", status),
    ];

    // Record to OpenTelemetry
    metrics.http_requests_total.add(1, &labels);
    metrics
        .http_request_duration_seconds
        .record(latency, &labels);

    response
}
