use crate::digest::csv_to_lot;
use portools_common::model::Portfolio;
use crate::service::state::State;
use crate::service::util::ContentLengthHeaderError;
use crate::service::util::ContentLengthHeaderError::Malformed;
use actix_web::web::{Data, Json, Path};
use actix_web::{error, web, HttpRequest, HttpResponse, Responder};
use tracing;
use tracing_actix_web::TracingLogger;
use ContentLengthHeaderError::Missing;

pub mod state;
pub(crate) mod util;

pub fn config(cfg: &mut web::ServiceConfig, state: &Data<State>) {
    cfg.service(
        // use a scope of "" in order to make use of wrap()
        // ref: https://github.com/actix/actix-web/issues/797
        web::scope("")
            .wrap(TracingLogger::default())
            .app_data(state.clone())
            .route("/portfolio/{portfolio_id}", web::get().to(get_portfolio))
            .route("/portfolio/{portfolio_id}", web::put().to(put_portfolio)),
    );
}

pub async fn get_portfolio(
    path: Path<u32>,
    data: Data<State>,
) -> actix_web::Result<Json<Portfolio>> {
    let portfolio_id = path.into_inner();
    match data.dao.get_portfolio(portfolio_id).await {
        Ok(Some(portfolio)) => Ok(Json(portfolio)),
        Ok(None) => Err(error::ErrorNotFound("portfolio not found")),
        Err(e) => {
            tracing::error!("dao.get_portfolio error: {e}");
            Err(error::ErrorInternalServerError(e))
        }
    }
}

pub async fn put_portfolio(
    path: Path<u32>,
    csv: web::Bytes,
    req: HttpRequest,
    data: Data<State>,
) -> impl Responder {
    let portfolio_id = path.into_inner();
    let content_length = util::get_content_length_header(&req);
    if content_length.is_err() {
        return match content_length.unwrap_err() {
            Malformed(message) => {
                tracing::debug!(%message, "malformed Content-Length header");
                HttpResponse::BadRequest()
            }
            Missing => {
                tracing::debug!("missing Content-Length header");
                HttpResponse::LengthRequired()
            }
        };
    }
    let content_length = content_length.unwrap();
    if content_length > data.limits.portfolio.max_file_size {
        tracing::debug!(
            content_length,
            max_content_length = data.limits.portfolio.max_file_size,
            "Content-Length header exceeds maximum",
        );
        return HttpResponse::PayloadTooLarge();
    }
    let lots = match csv_to_lot(csv) {
        Ok(csv_lots) => csv_lots,
        Err(error) => {
            tracing::debug!(?error, "failed to convert CSV to Lots");
            return HttpResponse::BadRequest();
        }
    };
    if lots.len() > data.limits.portfolio.max_num_lots {
        return HttpResponse::PayloadTooLarge();
    }
    let portfolio = Portfolio {
        id: portfolio_id,
        lots,
    };
    match data.dao.put_portfolio(&portfolio).await {
        Ok(_) => HttpResponse::Ok(),
        Err(error) => {
            tracing::error!(?error, "failed to persist portfolio");
            HttpResponse::InternalServerError()
        }
    }
}
