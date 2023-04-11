use crate::csv_digester::csv_to_lot;
use crate::model::Lot;
use crate::service::limits::LIMITS;
use crate::service::state::State;
use actix_web::web::{Data, Json};
use actix_web::{error, get, put, web, HttpRequest, HttpResponse, Responder};
use ContentLengthHeaderError::Missing;
use crate::service::util::ContentLengthHeaderError;
use crate::service::util::ContentLengthHeaderError::Malformed;

pub(crate) mod limits;
pub(crate) mod state;
pub(crate) mod util;

#[get("/lots")]
pub async fn get_lots(data: Data<State>) -> actix_web::Result<Json<Vec<Lot>>> {
    match data.dao.get_lots().await {
        Ok(lots) => Ok(Json(lots)),
        Err(e) => {
            println!("get_lots error: {e}");
            Err(error::ErrorInternalServerError(e))
        }
    }
}

#[put("/lots")]
pub async fn put_lots(csv: web::Bytes, req: HttpRequest, data: Data<State>) -> impl Responder {
    let content_length = util::get_content_length_header(&req);
    if content_length.is_err() {
        return match content_length.unwrap_err() {
            Malformed(message) => {
                println!("bad request: {message}");
                HttpResponse::BadRequest()
            }
            Missing => HttpResponse::LengthRequired(),
        };
    }
    let content_length = content_length.unwrap();
    if content_length > LIMITS.max_file_size {
        return HttpResponse::PayloadTooLarge();
    }
    match csv_to_lot(csv) {
        Ok(ref lots) => {
            if lots.len() > LIMITS.max_num_lots {
                return HttpResponse::PayloadTooLarge();
            }
            match data.dao.put_lots(lots).await {
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
