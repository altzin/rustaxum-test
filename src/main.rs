// src/main.rs
use axum::Router;
use opentelemetry::global;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod metrics;
mod middleware;
mod routes;
mod state;
mod telemetry;

use middleware::RouterExt;

use crate::{config::OtlpConfig, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    // let endpoint = env::var("OTEL_ENDPOINT").expect("OTEL_ENDPOINT must be set");
    // let auth_value = env::var("OTEL_AUTH_HEADER").expect("OTEL_AUTH_HEADER must be set");
    let service_name = env::var("SERVICE_NAME").expect("SERVICE_NAME must be set");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let is_prod = env::var("APP_ENV").map(|v| v == "prod").unwrap_or(false);
    // Only build the OtlpConfig if the required auth header is present
    let otlp_config = env::var("OTEL_AUTH_HEADER").ok().map(|auth| OtlpConfig {
        host: env::var("OTEL_HOST").unwrap_or_else(|_| "localhost".to_string()),
        org: env::var("OTEL_ORG").unwrap_or_else(|_| "default".to_string()),
        port: env::var("OTEL_PORT")
            .unwrap_or_else(|_| "5080".to_string())
            .parse()
            .unwrap_or(5080),
        auth_header: auth,
    });

    // Pass the config by reference to both providers
    // (You would apply the exact same signature change to telemetry::init_tracer)
    let meter_provider = metrics::init_metrics(otlp_config.as_ref(), &service_name)?;
    let tracer_provider = telemetry::init_tracer(otlp_config.as_ref(), &service_name, is_prod)?;

    let app_metrics = metrics::AppMetrics::new();

    // 2. Set up logging & span bridging
    let tracer = global::tracer(service_name.clone());
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer)
        .init();

    tracing::info!(
        "Starting service '{}' in {} mode",
        service_name,
        if is_prod { "production" } else { "development" }
    );

    // 3. Database & Migrations
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
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

    let swagger =
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", routes::ApiDoc::openapi());

    let app = Router::new()
        .merge(routes::create_router())
        .merge(swagger)
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

    // 6. Flush remaining spans & metrics to OpenObserve on exit
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
