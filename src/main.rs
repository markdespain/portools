mod actix_util;
mod portfolio;
mod util;

use crate::portfolio::{Currency, Lot};
use crate::util::ValidationError;
use actix_util::ContentLengthHeaderError;
use actix_util::ContentLengthHeaderError::MalformedContentLengthHeader;
use actix_web::web::Buf;
use actix_web::{
    get, put, web,
    web::{Data, Json},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::naive::NaiveDate;
use csv::StringRecord;
use rust_decimal::Decimal;
use std::collections::HashMap;
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
    let mut rdr = csv::Reader::from_reader(csv.reader());
    let mut field_to_index = HashMap::with_capacity(5);
    let headers = rdr.headers();
    if headers.is_err() {
        return HttpResponse::BadRequest();
    }
    let headers = headers.unwrap();
    for (i, header) in headers.iter().enumerate() {
        field_to_index.insert(header.to_owned(), usize::from(i));
    }
    let mut lots = Vec::new();
    for record in rdr.records() {
        match record {
            Ok(r) => match to_lot(&field_to_index, &r) {
                Ok(lot) => {
                    lots.push(lot);
                }
                Err(e) => {
                    println!("failed to convert record to Lot: {:?}", e);
                    return HttpResponse::BadRequest();
                }
            },
            Err(e) => {
                println!("failed to convert uploaded bytes to utf8: {e}");
                return HttpResponse::BadRequest();
            }
        }
    }
    data.set_lots(lots);
    HttpResponse::Ok()
}

fn to_lot(
    field_to_index: &HashMap<String, usize>,
    record: &StringRecord,
) -> Result<Lot, RecordError> {
    Lot::new_str(
        get_field("account", field_to_index, record)?,
        get_field("symbol", field_to_index, record)?,
        get_field("date_acquired", field_to_index, record)?,
        get_field("quantity", field_to_index, record)?,
        get_field("cost_per_share", field_to_index, record)?,
    )
    .map_err(|validation_error| RecordError::Invalid {
        error: validation_error,
    })
}

fn get_field(
    field: &str,
    field_to_index: &HashMap<String, usize>,
    record: &StringRecord,
) -> Result<String, RecordError> {
    let field_index = field_to_index
        .get(&field[..])
        .ok_or(RecordError::MissingHeader {
            field: field.to_string(),
        })?;
    let field_value = record.get(*field_index).ok_or(RecordError::MissingValue {
        field: field.to_string(),
    })?;
    Ok(String::from(field_value))
}

#[derive(Debug)]
enum RecordError {
    MissingHeader { field: String },
    MissingValue { field: String },
    Invalid { error: ValidationError },
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
