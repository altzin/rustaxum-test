use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{
    Resource,
    trace::{Sampler, SdkTracerProvider},
};
use std::{collections::HashMap, env};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let endpoint = env::var("OTEL_ENDPOINT").expect("OTEL_ENDPOINT must be set");
    let auth_value = env::var("OTEL_AUTH_HEADER").expect("OTEL_AUTH_HEADER must be set");
    let service_name = env::var("SERVICE_NAME").expect("Must set service name");

    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), auth_value);

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .with_headers(headers)
        .build()
        .expect("Failed to build exporter");

    let resource = Resource::builder()
        .with_attribute(KeyValue::new("service.name", service_name))
        .build();

    // Because 'rt-tokio' is in Cargo.toml, this now natively uses Tokio
    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::AlwaysOn)
        .build();

    global::set_tracer_provider(provider.clone());
    let tracer = global::tracer("axum-test");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry)
        .init();

    let app = routes::create_router()
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(listener, app).await;

    let _ = provider.force_flush();
}
