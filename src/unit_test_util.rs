#[cfg(test)]
pub mod fixture {
    use crate::model::{Currency, Lot};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use uuid::uuid;

    pub fn lot() -> Lot {
        Lot::new(
            uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            "Taxable",
            "VOO",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(6),
            Currency::new("300.64".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap()
    }
}
