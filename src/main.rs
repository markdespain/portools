mod portfolio;
mod util;

use crate::portfolio::{Currency, Lot};
use actix_multipart::{Field, Multipart};
use actix_web::{
    get,
    http::header::CONTENT_LENGTH,
    post,
    web::{Data, Json},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use chrono::naive::NaiveDate;
use futures_util::TryStreamExt as _;
use std::io;
use std::sync::Mutex;
use uuid::Uuid;
use ContentLengthHeaderError::MalformedContentLengthHeader;

const MAX_FILE_SIZE: usize = 10_000;

#[get("/lots")]
async fn get_lots(data: Data<AppState>) -> impl Responder {
    let lots = data.get_lots();
    Json(lots)
}

#[post("/lots")]
async fn post_lots(mut payload: Multipart, req: HttpRequest) -> impl Responder {
    let content_length = get_content_length_header(&req);
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
    let csv = multipart_to_vec(&mut payload, content_length).await;
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

fn get_content_length_header(req: &HttpRequest) -> Result<usize, ContentLengthHeaderError> {
    let header_value = req
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or(ContentLengthHeaderError::NoContentLengthHeader)?;
    let header_str = header_value.to_str().map_err(|e| {
        MalformedContentLengthHeader(format!("failed to convert content-length to a str: {e}"))
    })?;
    let content_length = header_str.parse().map_err(|e| {
        MalformedContentLengthHeader(format!("failed to parse content-length to to a u8: {e}"))
    })?;
    if content_length > 0 {
        Ok(content_length)
    } else {
        Err(MalformedContentLengthHeader(format!(
            "content-length was not positive: {content_length}"
        )))
    }
}

#[derive(Debug)]
enum ContentLengthHeaderError {
    NoContentLengthHeader,
    MalformedContentLengthHeader(String),
}

#[derive(Debug)]
enum UploadError {
    NoFile,
    MaxSizeExceeded,
}

async fn multipart_to_vec(
    payload: &mut Multipart,
    max_num_bytes: usize,
) -> Result<Vec<u8>, UploadError> {
    if let Ok(Some(mut field)) = payload.try_next().await {
        return field_to_vec(&mut field, max_num_bytes).await;
    }
    Err(UploadError::NoFile)
}

async fn field_to_vec(field: &mut Field, max_num_bytes: usize) -> Result<Vec<u8>, UploadError> {
    let mut csv_bytes: Vec<u8> = Vec::new();
    while let Ok(Some(chunk)) = field.try_next().await {
        if csv_bytes.len() + max_num_bytes > max_num_bytes {
            return Err(UploadError::MaxSizeExceeded);
        }
        csv_bytes.append(&mut chunk.to_owned().to_vec());
    }
    Ok(csv_bytes)
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
