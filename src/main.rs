use actix_web::{web::Data, App, HttpServer};
use mongodb::Client;
use std::io;

use portools::dao::mongo;
use portools::dao::mongo::MongoDao;
use portools::service;
use portools::service::state::State;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    mongo::create_lots_index(&client).await;

    let app_state = Data::new(State::new(Box::new(MongoDao { client })));
    HttpServer::new(move || App::new().configure(|cfg| service::config(cfg, &app_state)))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
