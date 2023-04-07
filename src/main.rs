mod actix_util;
mod csv_digester;
mod dao;
mod model;
mod util;

use crate::csv_digester::csv_to_lot;
use crate::model::Lot;
use actix_util::ContentLengthHeaderError;
use actix_util::ContentLengthHeaderError::Malformed;
use actix_web::{
    error, get, put, web,
    web::{Data, Json},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use mongodb::Client;
use std::io;
use std::sync::Mutex;

struct AppLimits {
    max_file_size: usize,
    max_num_lots: usize,
}

// TODO: allow limits to be configurable
const APP_LIMITS: AppLimits = AppLimits {
    max_file_size: 10_000,
    max_num_lots: 10_000,
};

#[get("/lots")]
async fn get_lots(data: Data<AppState>) -> actix_web::Result<Json<Vec<Lot>>> {
    match dao::get_lots(&data.client).await {
        Ok(lots) => Ok(Json(lots)),
        Err(e) => {
            println!("get_lots error: {e}");
            Err(error::ErrorInternalServerError(e))
        }
    }
}

#[put("/lots")]
async fn put_lots(csv: web::Bytes, req: HttpRequest, data: Data<AppState>) -> impl Responder {
    let content_length = actix_util::get_content_length_header(&req);
    if content_length.is_err() {
        return match content_length.unwrap_err() {
            Malformed(message) => {
                println!("bad request: {message}");
                HttpResponse::BadRequest()
            }
            ContentLengthHeaderError::Missing => HttpResponse::LengthRequired(),
        };
    }
    let content_length = content_length.unwrap();
    if content_length > APP_LIMITS.max_file_size {
        return HttpResponse::PayloadTooLarge();
    }
    match csv_to_lot(csv) {
        Ok(ref lots) => {
            if lots.len() > APP_LIMITS.max_num_lots {
                return HttpResponse::PayloadTooLarge();
            }
            match dao::put_lots(&data.client, lots).await {
                Ok(_) => HttpResponse::Ok(),
                Err(e) => {
                    println!("get_lots error: {e}");
                    HttpResponse::InternalServerError()
                }
            }
        }
        Err(e) => {
            println!("Invalid upload: {:?}", e);
            HttpResponse::BadRequest()
        }
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // todo: refine so that docker and non-Docker flows are seamless
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    dao::create_lots_index(&client).await;

    let app_state = Data::new(AppState::new(client));
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(get_lots)
            .service(put_lots)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

struct AppState {
    // todo: abstract as Dao, move Mutex and Client into respective dao implementations
    lots: Mutex<Vec<Lot>>,
    client: Client,
}

impl AppState {
    fn new(client: Client) -> AppState {
        AppState {
            lots: Mutex::new(Vec::new()),
            client,
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
