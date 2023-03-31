mod portfolio;
mod util;
mod actix_util;

use crate::portfolio::{Currency, Lot};
use actix_multipart::Multipart;
use actix_web::{
    App,
    get,
    HttpRequest,
    HttpResponse, HttpServer, post, Responder, web::{Data, Json},
};
use chrono::naive::NaiveDate;
use std::io;
use std::sync::Mutex;
use uuid::Uuid;
use actix_util::ContentLengthHeaderError;
use actix_util::ContentLengthHeaderError::MalformedContentLengthHeader;

const MAX_FILE_SIZE: usize = 10_000;

#[get("/lots")]
async fn get_lots(data: Data<AppState>) -> impl Responder {
    let lots = data.get_lots();
    Json(lots)
}

#[post("/lots")]
async fn post_lots(mut payload: Multipart, req: HttpRequest) -> impl Responder {
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
    let csv = actix_util::multipart_to_vec(&mut payload, content_length).await;
    if csv.is_err() {
        println!("upload error: {:?}", csv.unwrap_err());
        return HttpResponse::BadRequest();
    }
    match String::from_utf8(csv.unwrap()) {
        Ok(result) => {
            println!("uploaded: {result}");
            HttpResponse::Ok()
        }
        Err(e) => {
            println!("failed to convert uploaded bytes to utf8: {e}");
            HttpResponse::BadRequest()
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let mut app_state = AppState::new();
    let lots = vec![Lot::new(
        Uuid::new_v4(),
        String::from("Joint Taxable"),
        String::from("VOO"),
        NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
        6,
        Currency::new(30064, String::from("USD")).unwrap(),
    )
    .unwrap()];
    app_state.set_lots(lots);

    let app_state = Data::new(app_state);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(get_lots)
            .service(post_lots)
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

    fn set_lots(&mut self, new_lots: Vec<Lot>) {
        let mut l = self.lots.lock().unwrap();
        *l = new_lots;
    }

    fn get_lots(&self) -> Vec<Lot> {
        let l = self.lots.lock().unwrap();
        l.to_vec()
    }
}
