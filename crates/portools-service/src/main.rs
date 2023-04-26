use actix_web::{web::Data, App, HttpServer};
use mongodb::Client;
use std::io;

use portools_common::dao::mongo;
use portools_common::dao::mongo::MongoDao;
use portools_common::log;

use portools_service::config::Limits;
use portools_service::service;
use portools_service::service::state::State;

const APP_NAME: &str = "portools-service";

#[actix_web::main]
async fn main() -> io::Result<()> {
    log::init(APP_NAME)
        .unwrap_or_else(|error| panic!("failed initialize logging. cause: {:?}", error));

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(&uri)
        .await
        .unwrap_or_else(|error| panic!("should be able to connect to {uri}. error: {error}"));
    mongo::create_collections_and_indexes(&client)
        .await
        .unwrap_or_else(|error| {
            panic!("should be able create collections and indexes. error: {error}")
        });

    let limits: Limits = confy::load(APP_NAME, None).unwrap_or_else(|error| {
        panic!(
            "should be able to load configuration from {:?}. error: {error}",
            confy::get_configuration_file_path(APP_NAME, None)
        )
    });
    tracing::info!("using limits: {:?}", limits);
    let app_state = Data::new(State {
        limits,
        dao: Box::new(MongoDao::new(client)),
    });
    HttpServer::new(move || App::new().configure(|cfg| service::config(cfg, &app_state)))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
