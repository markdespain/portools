use std::error::Error;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Registry};

#[derive(Debug)]
pub struct LogInitError {
    pub message: String,
    pub cause: Box<dyn Error>,
}

pub fn init(name: &str) -> Result<(), LogInitError> {
    LogTracer::init().map_err(|cause| LogInitError {
        message: "failed to init LogTracer".into(),
        cause: Box::new(cause),
    })?;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(name.into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).map_err(|cause| LogInitError {
        message: "set global default subscriber".into(),
        cause: Box::new(cause),
    })?;
    Ok(())
}
