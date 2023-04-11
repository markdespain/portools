#[cfg(test)]
mod tests {
    use actix_web::web::{Bytes, ServiceConfig};
    use actix_web::web::Data;
    use actix_web::{test, App};
    use chrono::NaiveDate;
    use portools::dao::local::InMemoryDao;
    use portools::model::{Currency, Lot};
    use portools::service;
    use portools::service::state::State;
    use rust_decimal::Decimal;
    use std::path::PathBuf;
    use test_util::assertion::assert_vec_eq_fn;
    use test_util::resource;
    use uuid::Uuid;

    const DATE_FORMAT: &'static str = "%Y/%m/%d";

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
            Uuid::new_v4(),
            account,
            symbol,
            NaiveDate::parse_from_str(date, DATE_FORMAT).unwrap(),
            Decimal::from(quantity),
            cost_basis,
        )
        .unwrap()
    }

    fn test_config(cfg: &mut ServiceConfig) {
        let app_state = Data::new(State::new(Box::<InMemoryDao>::new(Default::default())));
        service::config(cfg, &app_state);
    }

    #[actix_web::test]
    async fn test_lots_get_empty() {
        let app = test::init_service(App::new().configure(test_config)).await;

        let req = test::TestRequest::get().uri("/lots").to_request();
        let resp: Vec<Lot> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(Vec::<Lot>::new(), resp);
    }

    #[actix_web::test]
    async fn test_lots_put_with_valid() {
        let app =
            test::init_service(App::new().configure(test_config)).await;

        // put the CSV
        let csv = load_bytes("valid.csv");
        let put_request = test::TestRequest::put()
            .uri("/lots")
            .append_header(("Content-Length", csv.len()))
            .set_payload(csv)
            .to_request();
        let put_response = test::call_service(&app, put_request).await;
        assert_eq!(200, put_response.status().as_u16());

        // assert the state
        let req = test::TestRequest::get().uri("/lots").to_request();
        let resp: Vec<Lot> = test::call_and_read_body_json(&app, req).await;
        let expected = vec![
            new_lot("Taxable", "VOO", "2023/03/27", 1, 100.47),
            new_lot("IRA", "BND", "2023/03/28", 2, 200.26),
            new_lot("IRA", "BND", "2023/03/29", 3, 300.23),
        ];
        assert_vec_eq_fn(&expected, &resp, Lot::eq_ignore_id);
    }
}
