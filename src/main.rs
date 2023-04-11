mod csv_digester;
mod dao;
mod model;
mod service;
mod test_util;
mod validate;

use actix_web::{web, web::Data, App, HttpServer};
use dao::mongo;
use mongodb::Client;
use std::io;

use service::state::State;

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(service::get_lots)
        .service(service::put_lots);
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    mongo::create_lots_index(&client).await;

    let app_state = Data::new(State::new(client));
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
