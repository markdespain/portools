#[cfg(test)]
mod tests {
    use actix_web::{test, App};

    use portools::model::Portfolio;

    use crate::util;
    use crate::util::test_config;

    #[actix_web::test]
    async fn test_portfolio_get_not_found() {
        let dao = util::init_dao().await;
        let app = test::init_service(App::new().configure(move |cfg| {
            test_config(cfg, dao);
        }))
        .await;

        // put the CSV
        let csv = util::load_bytes("valid.csv");
        let put_request = test::TestRequest::put()
            .uri("/portfolio/1")
            .append_header(("Content-Length", csv.len()))
            .set_payload(csv)
            .to_request();
        let put_response = test::call_service(&app, put_request).await;
        assert_eq!(200, put_response.status().as_u16());

        // assert the state
        let get_request = test::TestRequest::get().uri("/portfolio/2").to_request();
        let get_response = test::call_service(&app, get_request).await;
        assert_eq!(404, get_response.status().as_u16());
    }

    #[actix_web::test]
    async fn test_portfolio_put_with_valid() {
        let dao = util::init_dao().await;
        let app = test::init_service(App::new().configure(move |cfg| {
            test_config(cfg, dao);
        }))
        .await;

        // put the CSV
        let csv = util::load_bytes("valid.csv");
        let put_request = test::TestRequest::put()
            .uri("/portfolio/1")
            .append_header(("Content-Length", csv.len()))
            .set_payload(csv)
            .to_request();
        let put_response = test::call_service(&app, put_request).await;
        assert_eq!(200, put_response.status().as_u16());

        // assert the state
        let req = test::TestRequest::get().uri("/portfolio/1").to_request();
        let resp: Portfolio = test::call_and_read_body_json(&app, req).await;
        let expected = Portfolio {
            id: 1,
            lots: vec![
                util::new_lot("Taxable", "VOO", "2023/03/27", 1, 100.47),
                util::new_lot("IRA", "BND", "2023/03/28", 2, 200.26),
                util::new_lot("IRA", "BND", "2023/03/29", 3, 300.23),
            ],
        };
        assert_eq!(&expected, &resp);
    }
}

mod util {
    use std::env::VarError;
    use std::path::PathBuf;
    use test_util::resource;

    use actix_web::web::{Bytes, Data, ServiceConfig};
    use chrono::NaiveDate;
    use mongodb::Client;
    use rust_decimal::Decimal;
    use portools::dao::local::InMemoryDao;

    use portools::dao::mongo;
    use portools::dao::mongo::MongoDao;
    use portools::model::{Currency, Lot};
    use portools::service;
    use portools::service::state::{State, StateDao};

    const DATE_FORMAT: &'static str = "%Y/%m/%d";

    pub async fn init_dao() -> Box<StateDao> {
        match std::env::var("MONGODB_URI") {
            Ok(uri) => {
                println!("using Mongo DAO with URI {uri}");
                let client = Client::with_uri_str(uri).await.expect("failed to connect");
                mongo::create_indexes(&client).await;
                Box::new(MongoDao::new(client))
            }
            Err(VarError::NotUnicode(_)) => {
                panic!("MONGODB_URI environment variable was not unicode string");
            }
            Err(VarError::NotPresent) => {
                println!("using in-memory DAO");
                let dao : InMemoryDao = Default::default();
                Box::new(dao)
            }
        }
    }

    pub fn test_config(cfg: &mut ServiceConfig, dao: Box<StateDao>) {
        let app_state = Data::new(State::new(dao));
        service::config(cfg, &app_state);
    }

    pub fn load_bytes(resource: &str) -> Bytes {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resource/test/csv_digester/");
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
