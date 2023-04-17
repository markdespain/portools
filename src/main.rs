use actix_web::{web::Data, App, HttpServer};
use mongodb::Client;
use std::io;
use env_logger::Env;

use portools::dao::mongo;
use portools::dao::mongo::MongoDao;
use portools::service;
use portools::service::state::State;

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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
