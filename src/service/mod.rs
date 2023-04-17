use crate::digest::csv_to_lot;
use crate::model::Portfolio;
use crate::service::state::State;
use crate::service::util::ContentLengthHeaderError;
use crate::service::util::ContentLengthHeaderError::Malformed;
use actix_web::web::{Data, Json, Path};
use actix_web::{error, get, put, web, HttpRequest, HttpResponse, Responder};
use tracing;
use tracing::Instrument;
use uuid::Uuid;
use ContentLengthHeaderError::Missing;

pub mod state;
pub(crate) mod util;

pub fn config(cfg: &mut web::ServiceConfig, state: &Data<State>) {
    cfg.service(put_portfolio)
        .service(get_portfolio)
        .app_data(state.clone());
}

#[get("/portfolio/{portfolio_id}")]
pub async fn get_portfolio(
    path: Path<u32>,
    data: Data<State>,
) -> actix_web::Result<Json<Portfolio>> {
    let request_id = Uuid::new_v4();
    let portfolio_id = path.into_inner();
    let span = tracing::info_span!("get_portfolio", %request_id, %portfolio_id);
    let _guard = span.enter();

    match data
        .dao
        .get_portfolio(portfolio_id)
        .instrument(tracing::debug_span!("dao.get_portfolio"))
        .await
    {
        Ok(Some(portfolio)) => Ok(Json(portfolio)),
        Ok(None) => Err(error::ErrorNotFound("portfolio not found")),
        Err(e) => {
            tracing::error!("dao.get_portfolio error: {e}");
            Err(error::ErrorInternalServerError(e))
        }
    }
}

#[put("/portfolio/{portfolio_id}")]
pub async fn put_portfolio(
    path: Path<u32>,
    csv: web::Bytes,
    req: HttpRequest,
    data: Data<State>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let portfolio_id = path.into_inner();
    let span = tracing::info_span!("put_portfolio", %request_id, %portfolio_id);
    let _guard = span.enter();

    let content_length = util::get_content_length_header(&req);
    if content_length.is_err() {
        return match content_length.unwrap_err() {
            Malformed(message) => {
                tracing::debug!("bad request: {message}");
                HttpResponse::BadRequest()
            }
            Missing => HttpResponse::LengthRequired(),
        };
    }
    let content_length = content_length.unwrap();
    if content_length > data.limits.max_file_size {
        return HttpResponse::PayloadTooLarge();
    }
    match csv_to_lot(csv) {
        Ok(lots) => {
            if lots.len() > data.limits.max_num_lots {
                return HttpResponse::PayloadTooLarge();
            }
            let portfolio = Portfolio {
                id: portfolio_id,
                lots,
            };
            match data
                .dao
                .put_portfolio(&portfolio)
                .instrument(tracing::debug_span!("dao.put_portfolio"))
                .await
            {
                Ok(_) => HttpResponse::Ok(),
                Err(e) => {
                    tracing::error!("get_lots error: {e}");
                    HttpResponse::InternalServerError()
                }
            }
        }
        Err(e) => {
            tracing::debug!("Invalid upload: {:?}", e);
            HttpResponse::BadRequest()
        }
    }
}
