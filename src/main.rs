mod portfolio;
mod util;

use crate::portfolio::{Currency, Lot};
use actix_web::{
    get,
    web::{Data, Json},
    App, HttpServer, Responder, Result,
};
use chrono::naive::NaiveDate;
use uuid::Uuid;

#[get("/lots")]
async fn get_lots(data: Data<AppState>) -> Result<impl Responder> {
    let lots = data.get_lots();
    Ok(Json(lots))
}

// #[post("/lots")]
// async fn post_lots(req_body: String) -> impl Responder {
//     HttpResponse::Ok().body(req_body)
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    HttpServer::new(move || App::new().app_data(app_state.clone()).service(get_lots))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

use std::sync::Mutex;

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
