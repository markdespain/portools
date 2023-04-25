use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::validate::{Invalid, trim_and_validate_len};

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

    // todo: unit tests
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

#[derive(Debug)]
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
    use crate::unit_test_util::fixture;
    use rust_decimal::Decimal;
    use crate::model::currency::Currency;

    #[test]
    fn currency_new() {
        assert_eq!(
            Ok(fixture::currency()),
            Currency::new(Decimal::from(1), "USD")
        );
    }

    #[test]
    fn currency_new_symbol_with_whitespace() {
        assert_eq!(
            Ok(fixture::currency()),
            Currency::new(Decimal::from(1), " USD ")
        );
    }

    #[test]
    fn currency_new_symbol_too_short() {
        assert!(Currency::new(Decimal::from(1), "").is_err());
    }

    #[test]
    fn currency_new_symbol_too_long() {
        assert!(Currency::new(Decimal::from(1), "US Dollars").is_err());
    }
}
