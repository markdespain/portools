use actix_web::{web::Data, App, HttpServer};
use mongodb::Client;
use std::io;

use portools::dao::mongo;
use portools::dao::mongo::MongoDao;
use portools::service;
use portools::service::state::State;

use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, filter::EnvFilter, Registry};

#[actix_web::main]
async fn main() -> io::Result<()> {
    init_logging();

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(&uri)
        .await
        .unwrap_or_else(|_| panic!("should be able to connect to {}", uri));
    mongo::create_collections_and_indexes(&client)
        .await
        .expect("should be able create collections and indexes");

    let app_state = Data::new(State::new(Box::new(MongoDao::new(client))));
    HttpServer::new(move || App::new().configure(|cfg| service::config(cfg, &app_state)))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

fn init_logging() {
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "portools".into(), io::stdout
    );
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
}
