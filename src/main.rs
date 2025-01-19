use opentelemetry::{global, trace::TraceError};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    resource::{EnvResourceDetector, OsResourceDetector, ProcessResourceDetector, 
              SdkProvidedResourceDetector, TelemetryResourceDetector},
    runtime::Tokio,
    trace as sdktrace,
};
use std::{env, time::Duration};
use tracing::{info, info_span, Instrument};
use opentelemetry_otlp::WithExportConfig;

fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    // Set global propagator for distributed tracing context
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    // Initialize resource detectors with security-relevant metadata
    let os_resource = OsResourceDetector.detect(Duration::from_secs(0));
    let process_resource = ProcessResourceDetector.detect(Duration::from_secs(0));
    let sdk_resource = SdkProvidedResourceDetector.detect(Duration::from_secs(0));
    let env_resource = EnvResourceDetector::new().detect(Duration::from_secs(0));
    let telemetry_resource = TelemetryResourceDetector.detect(Duration::from_secs(0));

    // Configure and install OTLP exporter with secure defaults
    let endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(3))
        .into_tracer_with_trace_config(
            sdktrace::config()
                .with_resource(
                    os_resource
                        .merge(&process_resource)
                        .merge(&sdk_resource)
                        .merge(&env_resource)
                        .merge(&telemetry_resource),
                )
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize the tracer
    let _tracer = init_tracer()?;
    
    // Create a span for the main operation
    let root_span = info_span!(
        "hello_world_operation",
        service.name = "hello-world-demo",
        service.version = env!("CARGO_PKG_VERSION"),
        security.context = "demo_context"
    );

    // Perform the main operation within the span
    async {
        info!("Starting hello world application");
        
        // Simulate some work
        let message = "Hello, OpenTelemetry!";
        
        // Add an event to the span
        info!(
            message = message,
            timestamp = chrono::Utc::now().timestamp(),
            security.level = "INFO"
        );
        
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        info!("Application completed successfully");
    }
    .instrument(root_span)
    .await;

    // Ensure all spans are exported
    global::shutdown_tracer_provider();

    Ok(())
}