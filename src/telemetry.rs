use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers into `tracing`'s subscriber
pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    // Print all spans at += INFO level if the RUST_LOG variable is not set
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    // Output the span to stdout
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    // The `with` method is provided by `SubscriberExt`, an extension trait for `Subscriber exposted by `tracing_subscriber`
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a global default to process span data
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all of `log`'s events to the subscriber
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
