mod actix_util;
mod portfolio;
mod util;

use crate::portfolio::{Currency, Lot};
use actix_util::ContentLengthHeaderError;
use actix_util::ContentLengthHeaderError::MalformedContentLengthHeader;
use actix_web::{
    get, put, web,
    web::{Data, Json},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::naive::NaiveDate;
use rust_decimal::Decimal;
use std::io;
use std::sync::Mutex;

const MAX_FILE_SIZE: usize = 10_000;

#[get("/lots")]
async fn get_lots(data: Data<AppState>) -> impl Responder {
    let lots = data.get_lots();
    Json(lots)
}

#[put("/lots")]
async fn put_lots(csv: web::Bytes, req: HttpRequest, data: Data<AppState>) -> impl Responder {
    let content_length = actix_util::get_content_length_header(&req);
    if content_length.is_err() {
        return match content_length.unwrap_err() {
            MalformedContentLengthHeader(message) => {
                println!("bad request: {message}");
                HttpResponse::BadRequest()
            }
            ContentLengthHeaderError::NoContentLengthHeader => HttpResponse::LengthRequired(),
        };
    }
    let content_length = content_length.unwrap();
    if content_length > MAX_FILE_SIZE {
        return HttpResponse::PayloadTooLarge();
    }
    match Lot::from_csv(csv) {
        Ok(lots) => {
            data.set_lots(lots);
            HttpResponse::Ok()
        }
        Err(e) => {
            println!("Invalid upload: {:?}", e);
            HttpResponse::BadRequest()
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let app_state = AppState::new();
    let lots = vec![Lot::new(
        String::from("Joint Taxable"),
        String::from("VOO"),
        NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
        6,
        Currency::new(
            Decimal::from_str_exact("123.45").unwrap(),
            String::from("USD"),
        )
        .unwrap(),
    )
    .unwrap()];
    app_state.set_lots(lots);

    let app_state = Data::new(app_state);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(get_lots)
            .service(put_lots)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

struct AppState {
    lots: Mutex<Vec<Lot>>,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            lots: Mutex::new(Vec::new()),
        }
    }

    fn set_lots(&self, new_lots: Vec<Lot>) {
        let mut l = self.lots.lock().unwrap();
        *l = new_lots;
    }

    fn get_lots(&self) -> Vec<Lot> {
        let l = self.lots.lock().unwrap();
        l.to_vec()
    }
}
