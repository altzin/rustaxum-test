pub mod metrics;
pub mod tracer;

use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_observability(
    config: Option<&crate::config::OtlpConfig>,
    service_name: &str,
    is_prod: bool,
) -> Result<(SdkTracerProvider, SdkMeterProvider), Box<dyn std::error::Error>> {
    let meter_provider = metrics::init_metrics(config, service_name)?;
    let tracer_provider = tracer::init_tracer(config, service_name, is_prod)?;

    let tracer = opentelemetry::global::tracer(service_name.to_string());
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,opentelemetry_otlp=error,reqwest=error".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer)
        .init();

    Ok((tracer_provider, meter_provider))
}
