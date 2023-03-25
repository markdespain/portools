use crate::util::{ValidationError, *};
use chrono::naive::NaiveDate;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Currency {
    // the amount of the currency in its minor units (e.g. cents for "USD")
    amount: u64,

    // the symbol for the currency (e.g. "USD")
    symbol: String,
}

impl Currency {
    const MIN_SYMBOL_LEN: usize = 1;
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn new(amount: u64, symbol: String) -> Result<Currency, ValidationError> {
        let symbol = validate_and_trim(
            String::from("symbol"),
            symbol,
            Currency::MIN_SYMBOL_LEN,
            Currency::MAX_SYMBOL_LEN,
        )?;
        Ok(Currency { amount, symbol })
    }
}

// a Lot an amount of securities purchased as a particular time
#[derive(Debug, PartialEq, Serialize)]
pub struct Lot {
    // the account within the lot is held
    account: String,

    // the symbol of the security held
    symbol: String,

    // the date that the lot was purchased
    date: NaiveDate,

    // the number of shares of the security in this lot
    quantity: u32,

    // the per-share cost purchase price of this lot
    // TOOD: add validation
    cost_basis: Currency,
}

impl Lot {
    const MIN_ACCOUNT_LEN: usize = 1;
    const MAX_ACCOUNT_LEN: usize = 100;

    const MIN_SYMBOL_LEN: usize = 1;
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn new(
        account: String,
        symbol: String,
        date: NaiveDate,
        quantity: u32,
        cost_basis: Currency,
    ) -> Result<Lot, ValidationError> {
        let account = validate_and_trim(
            String::from("account"),
            account,
            Lot::MIN_ACCOUNT_LEN,
            Lot::MAX_ACCOUNT_LEN,
        )?;
        let symbol = validate_and_trim(
            String::from("symbol"),
            symbol,
            Lot::MIN_SYMBOL_LEN,
            Lot::MAX_SYMBOL_LEN,
        )?;
        Ok(Lot {
            account,
            symbol,
            date,
            quantity,
            cost_basis,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::portfolio::{Currency, Lot, ValidationError};
    use chrono::naive::NaiveDate;

    // Currency Tests

    #[test]
    fn currency_new() {
        assert_eq!(
            Ok(Currency {
                amount: 1,
                symbol: String::from("USD")
            }),
            Currency::new(1, String::from("USD"))
        )
    }

    #[test]
    fn currency_new_symbol_with_whitespace() {
        assert_eq!(
            Ok(Currency {
                amount: 1,
                symbol: String::from("USD")
            }),
            Currency::new(1, String::from(" USD "))
        )
    }

    #[test]
    fn currency_new_symbol_too_short() {
        assert_eq!(
            Err(ValidationError::FieldToShort {
                field: String::from("symbol"),
                min: 1,
                actual: 0
            }),
            Currency::new(1, String::from(""))
        );
    }

    #[test]
    fn currency_new_symbol_too_long() {
        assert_eq!(
            Err(ValidationError::FieldToLong {
                field: String::from("symbol"),
                max: 5,
                actual: 10
            }),
            Currency::new(1, String::from("US Dollars"))
        );
    }

    // Lot tests

    #[test]
    fn lot_new_valid() {
        assert_eq!(
            Ok(Lot {
                account: String::from("Taxable"),
                symbol: String::from("VOO"),
                date: NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                quantity: 6,
                cost_basis: Currency {
                    amount: 30064,
                    symbol: String::from("USD")
                }
            }),
            Lot::new(
                String::from("Taxable"),
                String::from("VOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(30064, String::from("USD")).unwrap()
            )
        )
    }

    #[test]
    fn lot_new_account_with_whitespace() {
        assert_eq!(
            Ok(Lot {
                account: String::from("Taxable"),
                symbol: String::from("VOO"),
                date: NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                quantity: 6,
                cost_basis: Currency {
                    amount: 30064,
                    symbol: String::from("USD")
                }
            }),
            Lot::new(
                String::from(" Taxable "),
                String::from("VOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(30064, String::from("USD")).unwrap()
            )
        )
    }

    #[test]
    fn lot_new_symbol_with_whitespace() {
        assert_eq!(
            Ok(Lot {
                account: String::from("Taxable"),
                symbol: String::from("VOO"),
                date: NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                quantity: 6,
                cost_basis: Currency {
                    amount: 30064,
                    symbol: String::from("USD")
                }
            }),
            Lot::new(
                String::from("Taxable"),
                String::from(" VOO "),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(30064, String::from("USD")).unwrap()
            )
        )
    }

    #[test]
    fn lot_new_account_too_short() {
        assert_eq!(
            Err(ValidationError::FieldToShort {
                field: String::from("account"),
                min: 1,
                actual: 0
            }),
            Lot::new(
                String::from(""),
                String::from("VOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(30064, String::from("USD")).unwrap()
            )
        )
    }

    #[test]
    fn lot_new_account_too_long() {
        let account: String = (0..101).map(|_| "X").collect();
        assert_eq!(
            Err(ValidationError::FieldToLong {
                field: String::from("account"),
                max: 100,
                actual: 101
            }),
            Lot::new(
                account,
                String::from("VOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(30064, String::from("USD")).unwrap()
            )
        )
    }

    #[test]
    fn lot_new_symbol_too_short() {
        assert_eq!(
            Err(ValidationError::FieldToShort {
                field: String::from("symbol"),
                min: 1,
                actual: 0
            }),
            Lot::new(
                String::from("Taxable"),
                String::from(""),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(30064, String::from("USD")).unwrap()
            )
        )
    }

    #[test]
    fn lot_new_symbol_too_long() {
        assert_eq!(
            Err(ValidationError::FieldToLong {
                field: String::from("symbol"),
                max: 5,
                actual: 6
            }),
            Lot::new(
                String::from("Taxable"),
                String::from("VOODOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(30064, String::from("USD")).unwrap()
            )
        )
    }
}
