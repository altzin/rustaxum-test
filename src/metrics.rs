use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{KeyValue, global};
use opentelemetry_sdk::{
    Resource,
    metrics::{PeriodicReader, SdkMeterProvider},
};
use std::collections::HashMap;
use std::time::Duration;

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
    endpoint: Option<String>,
    auth_header: Option<String>,
    service_name: &str,
) -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let resource = Resource::builder()
        .with_attribute(KeyValue::new("service.name", service_name.to_string()))
        .build();

    let mut builder = SdkMeterProvider::builder().with_resource(resource);
    let ep = endpoint.as_deref();
    let auth = auth_header.as_deref();

    if let (Some(ep), Some(auth)) = (endpoint, auth_header) {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), auth.to_string());

        let exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(ep)
            .with_headers(headers)
            .build()?;

        let reader = PeriodicReader::builder(exporter)
            .with_interval(Duration::from_secs(5))
            .build();

        builder = builder.with_reader(reader);
        tracing::info!("Metrics: remote OpenObserve exporter active");
    }

    let provider = builder.build();
    global::set_meter_provider(provider.clone());

    Ok(provider)
}
