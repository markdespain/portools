use crate::validate::{trim_and_validate_len, Invalid};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub const USD: &str = "USD";
pub const JPY: &str = "JPY";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Currency {
    // the amount of the currency
    pub amount: Decimal,

    // the symbol for the currency (e.g. "USD")
    pub symbol: String,
}

impl Currency {
    const MIN_SYMBOL_LEN: usize = 1;
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn new(amount: Decimal, symbol: &str) -> Result<Currency, Invalid> {
        let symbol = trim_and_validate_len(
            "symbol",
            symbol,
            Currency::MIN_SYMBOL_LEN,
            Currency::MAX_SYMBOL_LEN,
        )?;
        Ok(Currency { amount, symbol })
    }

    pub fn add(&self, other: &Currency) -> Result<Currency, CurrencyError<Currency>> {
        if self.symbol != other.symbol {
            return Err(CurrencyError::SymbolMismatch {
                left: self.symbol.clone(),
                right: other.symbol.clone(),
            });
        }
        match self.amount.checked_add(other.amount) {
            Some(sum) => Ok(Currency {
                amount: sum,
                symbol: self.symbol.clone(),
            }),
            None => Err(CurrencyError::Overflow {
                left: self.clone(),
                right: other.clone(),
                operation: "add".into(),
            }),
        }
    }

    pub fn multiply(&self, other: &Decimal) -> Result<Currency, CurrencyError<Decimal>> {
        match self.amount.checked_mul(*other) {
            Some(sum) => Ok(Currency {
                amount: sum,
                symbol: self.symbol.clone(),
            }),
            None => Err(CurrencyError::Overflow {
                left: self.clone(),
                right: *other,
                operation: "add".into(),
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CurrencyError<T> {
    SymbolMismatch {
        left: String,
        right: String,
    },
    Overflow {
        left: Currency,
        right: T,
        operation: String,
    },
}

#[cfg(test)]
mod tests {
    use crate::model::currency::{Currency, JPY, USD};
    use crate::model::CurrencyError;
    use crate::unit_test_util::fixture;
    use rust_decimal::Decimal;
    use test_util::assertion::assert_ok_eq;

    #[test]
    fn new() {
        assert_eq!(Ok(fixture::currency()), Currency::new(Decimal::ONE, USD));
    }

    #[test]
    fn new_with_symbol_with_whitespace() {
        assert_eq!(
            Ok(fixture::currency()),
            Currency::new(Decimal::ONE, " USD ")
        );
    }

    #[test]
    fn new_with_symbol_too_short() {
        assert!(Currency::new(Decimal::ONE, "").is_err());
    }

    #[test]
    fn new_with_symbol_too_long() {
        assert!(Currency::new(Decimal::ONE, "US Dollars").is_err());
    }

    #[test]
    fn add_basic() {
        let currency_1 = Currency::new(Decimal::ONE, USD).unwrap();
        let currency_2 = Currency::new(Decimal::TWO, USD).unwrap();
        let sum = currency_1.add(&currency_2);
        assert_ok_eq(&Currency::new(Decimal::from(3), USD).unwrap(), &sum)
    }

    #[test]
    fn add_causing_overflow() {
        let currency_1 = Currency::new(Decimal::MAX, USD).unwrap();
        let currency_2 = Currency::new(Decimal::ONE, USD).unwrap();
        match currency_1.add(&currency_2) {
            Ok(value) => panic!("expected error, but got {:?}", value),
            Err(CurrencyError::Overflow { left, right, .. }) => {
                assert_eq!(currency_1, left);
                assert_eq!(currency_2, right);
            }
            Err(unexpected) => panic!("got a different error than expected: {:?}", unexpected),
        };
    }

    #[test]
    fn add_with_different_currencies() {
        let currency_1 = Currency::new(Decimal::ONE, USD).unwrap();
        let currency_2 = Currency::new(Decimal::ONE, JPY).unwrap();
        match currency_1.add(&currency_2) {
            Ok(value) => panic!("expected error, but got {:?}", value),
            Err(CurrencyError::SymbolMismatch { left, right, .. }) => {
                assert_eq!(USD, left);
                assert_eq!(JPY, right);
            }
            Err(unexpected) => panic!("got a different error than expected: {:?}", unexpected),
        };
    }

    #[test]
    fn multiply_basic() {
        let currency = Currency::new(Decimal::ONE, USD).unwrap();
        let sum = currency.multiply(&Decimal::TWO);
        assert_ok_eq(&Currency::new(Decimal::TWO, USD).unwrap(), &sum)
    }

    #[test]
    fn multiply_causing_overflow() {
        let currency = Currency::new(Decimal::MAX, USD).unwrap();
        match currency.multiply(&Decimal::TWO) {
            Ok(value) => panic!("expected error, but got {:?}", value),
            Err(CurrencyError::Overflow { left, right, .. }) => {
                assert_eq!(currency, left);
                assert_eq!(Decimal::TWO, right);
            }
            Err(unexpected) => panic!("got a different error than expected: {:?}", unexpected),
        };
    }
}
