
use opentelemetry::{
    global,
    trace::{TraceError, TracerProvider as _},
};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    resource::{ResourceDetector, EnvResourceDetector, OsResourceDetector, ProcessResourceDetector, 
              SdkProvidedResourceDetector, TelemetryResourceDetector},
    trace as sdktrace,
    trace::TracerProvider,
};
use std::{env, time::Duration};
use tracing::{info, info_span, Instrument};

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

    // Build the tracer provider
    let trace_config = sdktrace::config()
        .with_resource(
            os_resource
                .merge(&process_resource)
                .merge(&sdk_resource)
                .merge(&env_resource)
                .merge(&telemetry_resource),
        )
        .with_sampler(sdktrace::Sampler::AlwaysOn);

    // Create a tracer provider
    let provider = TracerProvider::builder()
        .with_config(trace_config)
        .build();

    // Initialize the tracer and return it
    Ok(provider.versioned_tracer(
        "hello-world-demo",
        Some(env!("CARGO_PKG_VERSION")),
        Some("https://opentelemetry.io/schemas/1.4.0"),
        None,
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize the tracer
    let _tracer = init_tracer()?;
    
    // Create a span for the main operation
    let root_span = info_span!(
        "hello_world_operation",
        "service.name" = "hello-world-demo",
        "service.version" = env!("CARGO_PKG_VERSION"),
        "security.context" = "demo_context"
    );

    // Perform the main operation within the span
    async {
        info!("Starting hello world application");
        
        // Simulate some work
        let message = "Hello, OpenTelemetry!";
        
        // Add an event to the span with security context
        info!(
            event.type = "application_start",
            message = %message,
            timestamp = %chrono::Utc::now().timestamp(),
        );
        
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        info!(
            event.type = "application_end",
            message = "Application completed successfully"
        );
    }
    .instrument(root_span)
    .await;

    // Ensure all spans are exported
    global::shutdown_tracer_provider();

    Ok(())
}