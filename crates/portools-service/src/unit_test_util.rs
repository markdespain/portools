#[cfg(test)]
pub mod resource {
    use actix_web::web::Bytes;
    use std::path::PathBuf;
    use test_util::resource;

    pub fn load_bytes(resource: &str) -> Bytes {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resource/test/csv_digester/");
        path.push(resource);
        resource::load_bytes(path.to_str().unwrap())
    }
}

#[cfg(test)]
pub mod fixture {
    use crate::model::{Currency, Lot};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    pub fn lot() -> Lot {
        Lot::new(
            "Taxable",
            "VOO",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(6),
            Currency::new("300.64".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap()
    }
}
