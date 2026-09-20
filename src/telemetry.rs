use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithHttpConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use std::env;
use std::time::Duration;

// src/telemetry.rs
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use std::collections::HashMap;

pub fn init_tracer(
    auth_header: &str,
    service_name: &str,
) -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), auth_header.to_string());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_headers(headers)
        .build()?;

    let resource = Resource::builder()
        .with_attribute(KeyValue::new("service.name", service_name.to_string()))
        .build();

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::AlwaysOn)
        .build();

    global::set_tracer_provider(provider.clone());

    Ok(provider)
}

pub fn init_metrics() -> Result<Option<SdkMeterProvider>, Box<dyn std::error::Error>> {
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    if app_env == "prod" {
        // 1. Define standard resource attributes for OpenObserve
        let resource = Resource::builder()
            .with_attributes(vec![
                KeyValue::new(
                    "service.name",
                    env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "axum-test".into()),
                ),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("deployment.environment", "production"),
                KeyValue::new(
                    "host.name",
                    gethostname::gethostname().to_string_lossy().to_string(),
                ),
            ])
            .build();

        // 2. Build the remote OTLP exporter for OpenObserve
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .build()?;

        let reader = PeriodicReader::builder(exporter)
            .with_interval(Duration::from_secs(5))
            .build();

        let provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(reader)
            .build();

        global::set_meter_provider(provider.clone());
        tracing::info!("Telemetry: OpenObserve OTLP metrics pipeline active (prod)");

        Ok(Some(provider))
    } else {
        // Dev: Print metric updates to stdout or use an in-memory/no-op provider
        let stdout_exporter = opentelemetry_stdout::MetricExporter::default();
        let reader = PeriodicReader::builder(stdout_exporter)
            .with_interval(Duration::from_secs(30))
            .build();

        let provider = SdkMeterProvider::builder().with_reader(reader).build();

        global::set_meter_provider(provider.clone());
        tracing::info!("Telemetry: Local stdout metrics active (dev)");

        Ok(Some(provider))
    }
}
