mod portfolio;
mod util;

use crate::portfolio::{Currency, Lot};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use chrono::naive::NaiveDate;

#[get("/lot")]
async fn get_lot() -> Result<impl Responder> {
    let lot = Lot::new(
        String::from("Joint Taxable"),
        String::from("VOO"),
        NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
        6,
        Currency::new(30064, String::from("USD")).unwrap(),
    )
    .unwrap();
    Ok(web::Json(lot))
}

// #[post("/lot")]
// async fn echo(req_body: String) -> impl Responder {
//     HttpResponse::Ok().body(req_body)
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(get_lot))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
