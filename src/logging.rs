use color_eyre::Result;
use color_eyre::eyre::Context;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(
    service_name: &str,
    jaeger_endpoint: Option<&str>,
    tracing_level: &str,
) -> Result<Option<SdkTracerProvider>> {
    let resource = Resource::builder()
        .with_attributes(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            service_name.to_string(),
        )])
        .build();

    let (telemetry_layer, tracer_provider) = if let Some(jaeger_endpoint) = jaeger_endpoint {
        // Initialize OTLP exporter using gRPC (Tonic)
        let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(jaeger_endpoint)
            .build()
            .wrap_err("Failed to create OTLP span exporter")?;

        // Create a tracer provider with the exporter
        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(otlp_exporter)
            .with_resource(resource)
            .build();

        // Set it as the global provider
        opentelemetry::global::set_tracer_provider(tracer_provider.clone());

        let tracer = opentelemetry::global::tracer("music-manager");

        // Get a tracer from the global provider
        (
            Some(tracing_opentelemetry::layer().with_tracer(tracer)),
            Some(tracer_provider),
        )
    } else {
        (None, None)
    };

    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    let filter_layer =
        EnvFilter::try_new(tracing_level).wrap_err("Failed to create tracing filter")?;

    if let Some(telemetry_layer) = telemetry_layer {
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(telemetry_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    }

    Ok(tracer_provider)
}
