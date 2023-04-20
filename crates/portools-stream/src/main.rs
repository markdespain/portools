use allocation::AllocationService;
use mongodb::change_stream::event::OperationType;
use mongodb::Client;
use portools_common::model::Portfolio;
use std::io;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Registry};

const APP_NAME: &str = "portools";

const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";

use portools_common::dao::mongo::MongoDao;
use portools_stream::allocation;

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
        .await
    {
        Ok(stream) => stream,
        Err(_error) => panic!("failed to get stream: {_error}"),
    };

    let service = AllocationService {
        dao: Box::new(MongoDao::new(client)),
    };

    //let mut resume_token = None;
    while change_stream.is_alive() {
        match change_stream.next_if_any().await {
            Ok(Some(event)) => {
                // process event
                match event.operation_type {
                    OperationType::Insert | OperationType::Replace => {
                        if let Some(ref portfolio) = event.full_document {
                            match service.update_allocation(portfolio).await {
                                Err(_error) => {
                                    tracing::error!("failed to update portfolio allocation")
                                }
                                Ok(_) => {
                                    tracing::info!("updated portfolio allocation")
                                }
                            }
                        }
                    }
                    _ => {
                        tracing::warn!(
                            "unsupported operation performed: {:?}, document: {:?}",
                            event.operation_type,
                            event.full_document
                        );
                    }
                }
            }
            Ok(None) => {
                tracing::trace!("got none");
            }
            Err(error) => {
                tracing::error!("got error from stream: {error}")
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
