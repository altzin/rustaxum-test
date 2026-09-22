// src/main.rs
use opentelemetry::global;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

mod config;
mod middleware;
mod routes;
mod state;
mod telemetry;

pub mod error;

use middleware::RouterExt;

use crate::{config::AppConfig, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let config = AppConfig::from_env();

    let telemetry_guards =
        telemetry::init_telemetry(&config.service_name, config.is_prod, config.otlp.as_ref());

    tracing::info!(
        "Starting service '{}' in {} mode",
        config.service_name,
        if config.is_prod {
            "production"
        } else {
            "development"
        }
    );

    let meter = global::meter("axum-test");
    let items_created_counter = meter
        .u64_counter("items.created")
        .with_description("Total number of items successfully created")
        .build();

    let http_requests_total = meter
        .u64_counter("http.requests.total")
        .with_description("Total number of HTTP requests")
        .build();

    let http_request_duration_seconds = meter
        .f64_histogram("http.request.duration.seconds")
        .with_description("HTTP request duration in seconds")
        .build();

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
        items_created_counter,
        http_requests_total,
        http_request_duration_seconds,
    };

    let app = routes::create_router(config.is_prod)
        // Wrap all routes with our metrics tracker BEFORE providing the state
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::track_metrics,
        ))
        .with_base_middleware()
        // Provide the state last so it satisfies the Router<AppState> requirement
        .with_state(state);

    // 5. Server with graceful shutdown
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    info!("Server listening on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    telemetry_guards.shutdown();

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C signal handler");
    info!("Shutdown signal received, draining connections...");
}
