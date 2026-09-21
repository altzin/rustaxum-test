// src/main.rs
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

mod config;
mod middleware;
mod routes;
mod state;
mod telemetry;

pub mod error;

use middleware::RouterExt;

use crate::{config::AppConfig, state::AppState, telemetry::metrics};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let config = AppConfig::from_env();

    let (tracer_provider, meter_provider) =
        telemetry::init_observability(config.otlp.as_ref(), &config.service_name, config.is_prod)?;

    let app_metrics = metrics::AppMetrics::new();

    tracing::info!(
        "Starting service '{}' in {} mode",
        config.service_name,
        if config.is_prod {
            "production"
        } else {
            "development"
        }
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.db_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .inspect(|_| info!("Database migrations executed successfully"))
        .inspect_err(|e| error!(error = %e, "Database migration failed"))?;

    let state = AppState {
        db: pool,
        metrics: app_metrics,
    };

    let app = Router::new()
        .merge(routes::create_router(config.is_prod))
        // Attach the state to the router BEFORE adding the middleware
        .with_state(state.clone())
        // Wrap all routes with our metrics tracker
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::track_metrics,
        ))
        .with_base_middleware();

    // 5. Server with graceful shutdown
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    info!("Server listening on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    let _ = tracer_provider.shutdown();
    let _ = meter_provider.shutdown();

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C signal handler");
    info!("Shutdown signal received, draining connections...");
}
