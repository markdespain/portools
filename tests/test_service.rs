#[cfg(test)]
mod tests {

    use actix_web::web::Data;
    use actix_web::{test, App};
    use portools::dao::local::MutexDao;
    use portools::model::Lot;
    use portools::service;
    use portools::service::state::State;

    #[actix_web::test]
    async fn test_lots_get() {
        let app_state = Data::new(State::new(Box::new(MutexDao::new())));
        let app =
            test::init_service(App::new().configure(|cfg| service::config(cfg, &app_state))).await;
        let req = test::TestRequest::get().uri("/lots").to_request();
        let resp: Vec<Lot> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(Vec::<Lot>::new(), resp)
    }
}
