use axum::extract::FromRef;
use opentelemetry::metrics::{Counter, Histogram};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub items_created_counter: Counter<u64>,
    pub http_requests_total: Counter<u64>,
    pub http_request_duration_seconds: Histogram<f64>,
}

// Allows `State<PgPool>` in handlers
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
