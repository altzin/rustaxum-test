use crate::metrics::AppMetrics;
use axum::extract::FromRef;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub metrics: AppMetrics,
}

// Allows `State<AppMetrics>` in middleware/handlers
impl FromRef<AppState> for AppMetrics {
    fn from_ref(state: &AppState) -> Self {
        state.metrics.clone()
    }
}

// Allows `State<PgPool>` in handlers
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
