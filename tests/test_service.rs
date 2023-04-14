#[cfg(test)]
mod tests {
    use crate::util;
    use crate::util::test_config;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_portfolio_get_not_found() {
        let dao = util::init_dao().await;
        let app = test::init_service(App::new().configure(move |cfg| {
            test_config(cfg, dao);
        }))
        .await;

        util::put_portfolio(1, "valid.csv", &app).await;

        // assert the state
        let get_request = test::TestRequest::get().uri("/portfolio/2").to_request();
        let get_response = test::call_service(&app, get_request).await;
        assert_eq!(404, get_response.status().as_u16());
    }

    #[actix_web::test]
    async fn test_portfolio_put_with_valid_then_get() {
        let dao = util::init_dao().await;
        let app = test::init_service(App::new().configure(move |cfg| {
            test_config(cfg, dao);
        }))
        .await;

        util::put_portfolio(1, "valid.csv", &app).await;
        let resp = util::get_portfolio(1, &app).await;
        assert_eq!(&util::expected_valid_portfolio(1), &resp);
    }

    #[actix_web::test]
    async fn test_portfolio_update_existing() {
        let dao = util::init_dao().await;
        let app = test::init_service(App::new().configure(move |cfg| {
            test_config(cfg, dao);
        }))
        .await;

        util::put_portfolio(1, "valid.csv", &app).await;
        let resp = util::get_portfolio(1, &app).await;
        assert_eq!(&util::expected_valid_portfolio(1), &resp);

        util::put_portfolio(1, "valid_2.csv", &app).await;
        let resp = util::get_portfolio(1, &app).await;
        assert_eq!(&util::expected_valid_2_portfolio(1), &resp);
    }

    #[actix_web::test]
    async fn test_portfolio_put_multiple() {
        let dao = util::init_dao().await;
        let app = test::init_service(App::new().configure(move |cfg| {
            test_config(cfg, dao);
        }))
        .await;

        util::put_portfolio(1, "valid.csv", &app).await;
        util::put_portfolio(2, "valid_2.csv", &app).await;
        let resp_1 = util::get_portfolio(1, &app).await;
        assert_eq!(&util::expected_valid_portfolio(1), &resp_1);

        let resp = util::get_portfolio(2, &app).await;
        assert_eq!(&util::expected_valid_2_portfolio(2), &resp);
    }
}

mod util {
    use actix_http::Request;
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::test;
    use std::env::VarError;
    use std::path::PathBuf;
    use test_util::resource;

    use actix_web::web::{Bytes, Data, ServiceConfig};
    use chrono::NaiveDate;
    use mongodb::Client;
    use portools::dao::local::InMemoryDao;
    use rust_decimal::Decimal;

    use portools::dao::{Dao, mongo};
    use portools::dao::mongo::MongoDao;
    use portools::model::{Currency, Lot, Portfolio};
    use portools::service;
    use portools::service::state::State;

    const DATE_FORMAT: &'static str = "%Y/%m/%d";

    pub async fn put_portfolio(
        id: u32,
        csv_file: &str,
        app: &(impl Service<Request, Response = ServiceResponse, Error = actix_web::error::Error>
              + Sized),
    ) {
        let csv = load_bytes(csv_file);
        let put_request = test::TestRequest::put()
            .uri(&format!("/portfolio/{id}"))
            .append_header(("Content-Length", csv.len()))
            .set_payload(csv)
            .to_request();
        let put_response = test::call_service(&app, put_request).await;
        assert_eq!(200, put_response.status().as_u16());
    }

    pub async fn get_portfolio(
        id: u32,
        app: &(impl Service<Request, Response = ServiceResponse, Error = actix_web::error::Error>
              + Sized),
    ) -> Portfolio {
        let req = test::TestRequest::get()
            .uri(&format!("/portfolio/{id}"))
            .to_request();
        let resp: Portfolio = test::call_and_read_body_json(&app, req).await;
        resp
    }

    pub async fn init_dao() -> Box<dyn Dao> {
        match std::env::var("MONGODB_URI") {
            Ok(uri) => {
                println!("using Mongo DAO with URI {uri}");
                let client = Client::with_uri_str(uri).await.expect("failed to connect");
                mongo::drop_and_create_collections_and_indexes(&client).await;
                Box::new(MongoDao::new(client))
            }
            Err(VarError::NotUnicode(_)) => {
                panic!("MONGODB_URI environment variable was not unicode string");
            }
            Err(VarError::NotPresent) => {
                println!("using in-memory DAO");
                let dao: InMemoryDao = Default::default();
                Box::new(dao)
            }
        }
    }

    pub fn expected_valid_portfolio(id: u32) -> Portfolio {
        Portfolio {
            id,
            lots: vec![
                new_lot("Taxable", "VOO", "2023/03/27", 1, 100.47),
                new_lot("IRA", "BND", "2023/03/28", 2, 200.26),
                new_lot("IRA", "BND", "2023/03/29", 3, 300.23),
            ],
        }
    }

    pub fn expected_valid_2_portfolio(id: u32) -> Portfolio {
        Portfolio {
            id,
            lots: vec![
                new_lot("Taxable", "VTEB", "2023/03/27", 5, 55.55),
                new_lot("IRA", "VTI", "2023/03/28", 4, 222.22),
            ],
        }
    }

    pub fn test_config(cfg: &mut ServiceConfig, dao: Box<dyn Dao>) {
        let app_state = Data::new(State::new(dao));
        service::config(cfg, &app_state);
    }

    pub fn load_bytes(resource: &str) -> Bytes {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resource/test/service/");
        path.push(resource);
        resource::load_bytes(path.to_str().unwrap())
    }

    pub fn new_lot(
        account: &str,
        symbol: &str,
        date: &str,
        quantity: u32,
        cost_basis_usd: f64,
    ) -> Lot {
        let cost_basis = Currency::new(cost_basis_usd.to_string().parse().unwrap(), "USD").unwrap();
        Lot::new(
            account,
            symbol,
            NaiveDate::parse_from_str(date, DATE_FORMAT).unwrap(),
            Decimal::from(quantity),
            cost_basis,
        )
        .unwrap()
    }
}
