use mongodb::Client;
use portools_common::model::Portfolio;
use std::io;
use tokio;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Registry};

const APP_NAME: &str = "portools";

const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";

#[tokio::main]
async fn main() {
    init_logging();
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(&uri)
        .await
        .unwrap_or_else(|_| panic!("should be able to connect to {}", uri));
    let mut change_stream = match client
        .database(DB_NAME)
        .collection::<Portfolio>(COLL_PORTFOLIO)
        .watch(None, None)
        .await {
        Ok(stream) => stream,
        Err(e) => panic!("failed to get stream: {e}")
    };
    //let mut resume_token = None;
    while change_stream.is_alive() {
        match change_stream.next_if_any().await {
            Ok(Some(ref event)) =>  {
                // process event
                println!("operation performed: {:?}, document: {:?}", event.operation_type, event.full_document);
            }
            Ok(None) => {
                println!("got none");
            }
            Err(e) => {
                panic!("got error from stream: {e}")
            }
        }
        //resume_token = change_stream.resume_token();
    }
    println!("change stream is no longer alive");
}

// todo: extract into lib
fn init_logging() {
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(APP_NAME.into(), io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
}
