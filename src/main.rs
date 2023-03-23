use opentelemetry::{
    global,
    sdk::{
        export::trace::stdout,
        trace::{self, Tracer},
    },
};
use opentelemetry_otlp::WithExportConfig;
use tracing::{info, info_span, Instrument};
use tracing_forest::ForestLayer;
use tracing_subscriber::{prelude::*, EnvFilter};

#[allow(dead_code)]
fn otel_jaeger_tracer() -> Tracer {
    opentelemetry_jaeger::new_collector_pipeline()
        .with_endpoint("http://localhost:14268/api/traces")
        .with_service_name("service_jaeger_1")
        .with_trace_config(trace::config().with_sampler(trace::Sampler::AlwaysOn))
        .with_reqwest()
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("pipeline install error")
}

#[allow(dead_code)]
fn otel_otlp_tracer() -> Tracer {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::config().with_resource(opentelemetry::sdk::Resource::new(vec![
                opentelemetry::KeyValue::new("service.name", "service_otlp_1"),
            ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("pipeline install error")
}

#[allow(dead_code)]
fn otel_stdout_tracer() -> Tracer {
    stdout::new_pipeline().install_simple()
}

#[allow(dead_code)]
fn init_oltp_tracing() {
    let otel_otlp_tracer = otel_otlp_tracer();
    let otel = tracing_opentelemetry::layer().with_tracer(otel_otlp_tracer);
    let stdout = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::Registry::default()
        .with(otel)
        .with(stdout)
        .with(EnvFilter::from_default_env())
        .init();
}

#[allow(dead_code)]
fn init_jaeger_tracing() {
    let otel_jaeger_tracer = otel_jaeger_tracer();
    let otel = tracing_opentelemetry::layer().with_tracer(otel_jaeger_tracer);
    // let stdout = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::Registry::default()
        .with(ForestLayer::default())
        .with(otel)
        // .with(stdout)
        .with(EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() {
    // init_oltp_tracing();
    init_jaeger_tracing();

    let _root_span = info_span!("root").entered();
    info!("Starting the application...");

    let tasks = (0..=5).into_iter().map(|i| {
        tokio::spawn(
            async move {
                info!(target: "task", "Doing task {}...", i);
                do_task_1(i);
            }
            .in_current_span(),
        )
    });

    futures::future::join_all(tasks).await;

    // Gracefully shut down the OpenTelemetry pipeline
    global::shutdown_tracer_provider();
}

#[tracing::instrument]
fn do_task_1(val: i8) {
    info!("Calling do_task_1 with argument {}", val);
    do_task_2(val + 1);
}

#[tracing::instrument]
fn do_task_2(val: i8) {
    info!("Calling do_task_2 with argument {}", val);
    do_task_3(val + 1);
}

#[tracing::instrument]
fn do_task_3(val: i8) {
    info!("Calling do_task_3 with argument {}", val);
}
