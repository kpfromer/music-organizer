use std::path::PathBuf;
use std::time::SystemTime;

use color_eyre::Result;
use color_eyre::eyre::Context;
use fern::colors::Color;
use fern::colors::ColoredLevelConfig;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logging(
    console_level: log::LevelFilter,
    log_file: Option<PathBuf>,
    file_level: log::LevelFilter,
) -> Result<()> {
    // configure colors for the whole line
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::White)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

    // configure colors for the name of the level.
    // since almost all of them are the same as the color for the whole line, we
    // just clone `colors_line` and overwrite our changes
    let colors_level = colors_line.info(Color::Green);

    // Create base dispatch with no filtering (filtering happens in the chains)
    let mut base_dispatch = fern::Dispatch::new().level(log::LevelFilter::Trace);

    // Console output dispatch with colored formatting
    let console_dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date} {level} {target}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = humantime::format_rfc3339_seconds(SystemTime::now()),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        .level(console_level)
        .chain(std::io::stdout());

    base_dispatch = base_dispatch.chain(console_dispatch);

    // File output dispatch with plain text formatting (if log file is specified)
    if let Some(log_file_path) = log_file {
        let file_dispatch = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{date} {level} {target}] {message}",
                    date = humantime::format_rfc3339_seconds(SystemTime::now()),
                    target = record.target(),
                    level = record.level(),
                    message = message,
                ));
            })
            .level(file_level)
            .chain(fern::log_file(log_file_path)?);

        base_dispatch = base_dispatch.chain(file_dispatch);
    }

    base_dispatch.apply().wrap_err("Failed to setup logging")?;
    Ok(())
}

pub fn init_tracing(service_name: &str) -> Result<()> {
    let resource = Resource::builder()
        .with_attributes(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            service_name.to_string(),
        )])
        .build();

    // Initialize OTLP exporter using gRPC (Tonic)
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()
        .wrap_err("Failed to create OTLP span exporter")?;

    // Create a tracer provider with the exporter
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(resource)
        .build();

    // Set it as the global provider
    opentelemetry::global::set_tracer_provider(tracer_provider);

    // Get a tracer from the global provider
    let tracer = opentelemetry::global::tracer("music-manager");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(telemetry_layer)
        .init();

    Ok(())
}
