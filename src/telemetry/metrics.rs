use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{PeriodicReader, SdkMeterProvider},
};
use std::collections::HashMap;
use std::time::Duration;

use crate::config::OtlpConfig;

#[derive(Clone)]
pub struct AppMetrics {
    pub http_requests_total: Counter<u64>,
    pub http_request_duration_seconds: Histogram<f64>,
}

impl AppMetrics {
    pub fn new() -> Self {
        // Obtains a meter from the globally registered SdkMeterProvider
        let meter = global::meter("axum-test");

        Self {
            http_requests_total: meter
                .u64_counter("http_requests_total")
                .with_description("Total incoming HTTP requests")
                .build(),
            http_request_duration_seconds: meter
                .f64_histogram("http_request_duration_seconds")
                .with_description("HTTP request latency in seconds")
                .with_unit("s")
                .build(),
        }
    }
}
pub fn init_metrics(
    config: Option<&OtlpConfig>,
    service_name: &str,
) -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let resource = Resource::builder()
        .with_attribute(KeyValue::new("service.name", service_name.to_string()))
        .build();

    let mut builder = SdkMeterProvider::builder().with_resource(resource);

    if let Some(cfg) = config {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), cfg.auth_header.clone());

        // Use the dynamically generated base URL
        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(cfg.base_url() + "/v1/metrics")
            .with_headers(headers)
            .build()?;

        let reader = PeriodicReader::builder(exporter)
            .with_interval(Duration::from_secs(5))
            .build();

        builder = builder.with_reader(reader);
        println!(
            "Metrics: remote OpenObserve exporter active at {}",
            cfg.base_url()
        );
    } else {
        println!("Metrics: running in local mode (no remote exporter)");
    }

    let provider = builder.build();
    global::set_meter_provider(provider.clone());

    Ok(provider)
}
