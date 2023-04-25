#[cfg(test)]
pub mod fixture {
    use crate::model::{Currency, Lot, USD};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    pub fn currency() -> Currency {
        Currency {
            amount: Decimal::ONE,
            symbol: USD.into(),
        }
    }

    pub fn lot() -> Lot {
        Lot::new(
            "Taxable",
            "VOO",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(6),
            Currency::new("300.64".parse().unwrap(), USD).unwrap(),
        )
        .unwrap()
    }
}

#[cfg(test)]
pub mod factory {
    use crate::model::{Currency, Lot, USD};
    use crate::validate::Invalid;

    pub fn new_lot_from_spec(lot: Lot) -> Result<Lot, Invalid> {
        Lot::new(
            &lot.account,
            &lot.symbol,
            lot.date_acquired,
            lot.quantity,
            lot.cost_basis,
        )
    }

    pub fn new_usd_unchecked(amount: &str) -> Currency {
        Currency::new(amount.parse().unwrap(), USD).unwrap()
    }
}
