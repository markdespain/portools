use crate::csv_digester::csv_to_lot;
use crate::model::Portfolio;
use crate::service::limits::LIMITS;
use crate::service::state::State;
use crate::service::util::ContentLengthHeaderError;
use crate::service::util::ContentLengthHeaderError::Malformed;
use actix_web::web::{Data, Json, Path};
use actix_web::{error, get, put, web, HttpRequest, HttpResponse, Responder};
use ContentLengthHeaderError::Missing;

pub(crate) mod limits;
pub mod state;
pub(crate) mod util;

pub fn config(cfg: &mut web::ServiceConfig, state: &Data<State>) {
    cfg
        .service(put_portfolio)
        .service(get_portfolio)
        .app_data(state.clone());
}

#[get("/portfolio/{portfolio_id}")]
pub async fn get_portfolio(path: Path<u32>, data: Data<State>) -> actix_web::Result<Json<Portfolio>> {
    let portfolio_id = path.into_inner();
    match data.dao.get_portfolio(portfolio_id).await {
        Ok(Some(portfolio)) => Ok(Json(portfolio.to_owned())),
        Ok(None) => Err(error::ErrorNotFound("portfolio not found")),
        Err(e) => {
            println!("get_portfolio error: {e}");
            Err(error::ErrorInternalServerError(e))
        }
    }
}

#[put("/portfolio/{portfolio_id}")]
pub async fn put_portfolio(path: Path<u32>, csv: web::Bytes, req: HttpRequest, data: Data<State>) -> impl Responder {
    let portfolio_id = path.into_inner();
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
        Ok(lots) => {
            if lots.len() > LIMITS.max_num_lots {
                return HttpResponse::PayloadTooLarge();
            }
            let portfolio = Portfolio{
                id : portfolio_id,
                lots
            };
            match data.dao.put_portfolio(&portfolio).await {
                Ok(_) => return HttpResponse::Ok(),
                Err(e) => {
                    println!("get_lots error: {e}");
                    return HttpResponse::InternalServerError();
                }
            }
        }
        Err(e) => {
            println!("Invalid upload: {:?}", e);
            return HttpResponse::BadRequest();
        }
    }
}
