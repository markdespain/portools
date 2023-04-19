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
