use chrono::naive::NaiveDate;

#[derive(Debug, PartialEq)]
pub struct Currency {
    // the amount of the currency in its minor units (e.g. cents for "USD")
    amount: u64,

    // the symbol for the currency (e.g. "USD")
    symbol: String,
}

impl Currency {
    const MIN_SYMBOL_LEN: usize = 1;
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn new(amount: u64, symbol: String) -> Result<Currency, CurrencyValidationError> {
        let symbol = symbol.trim().to_string();
        if symbol.len() < Currency::MIN_SYMBOL_LEN {
            return Err(CurrencyValidationError::SymbolToShort {
                min: Currency::MIN_SYMBOL_LEN,
                actual: symbol.len(),
            });
        }
        if symbol.len() > Currency::MAX_SYMBOL_LEN {
            return Err(CurrencyValidationError::SymbolToLong {
                max: Currency::MAX_SYMBOL_LEN,
                actual: symbol.len(),
            });
        }
        Ok(Currency { amount, symbol })
    }
}

#[derive(Debug, PartialEq)]
pub enum CurrencyValidationError {
    SymbolToShort { min: usize, actual: usize },
    SymbolToLong { max: usize, actual: usize },
}

// a Lot an amount of securities purchased as a particular time
#[derive(Debug, PartialEq)]
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
    ) -> Result<Lot, LotValidationError> {
        let account = account.trim().to_string();
        if account.len() < Lot::MIN_ACCOUNT_LEN {
            return Err(LotValidationError::AccountToShort {
                min: Lot::MIN_ACCOUNT_LEN,
                actual: account.len(),
            });
        }
        if account.len() > Lot::MAX_ACCOUNT_LEN {
            return Err(LotValidationError::AccountToLong {
                max: Lot::MAX_ACCOUNT_LEN,
                actual: account.len(),
            });
        }
        let symbol = symbol.trim().to_string();
        if symbol.len() < Lot::MIN_SYMBOL_LEN {
            return Err(LotValidationError::SymbolToShort {
                min: Lot::MIN_SYMBOL_LEN,
                actual: symbol.len(),
            });
        }
        if symbol.len() > Lot::MAX_SYMBOL_LEN {
            return Err(LotValidationError::SymbolToLong {
                max: Lot::MAX_SYMBOL_LEN,
                actual: symbol.len(),
            });
        }
        Ok(Lot {
            account,
            symbol,
            date,
            quantity,
            cost_basis,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum LotValidationError {
    AccountToShort { min: usize, actual: usize },
    AccountToLong { max: usize, actual: usize },
    SymbolToShort { min: usize, actual: usize },
    SymbolToLong { max: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use crate::portfolio::{Currency, CurrencyValidationError, Lot, LotValidationError};
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
            Err(CurrencyValidationError::SymbolToShort { min: 1, actual: 0 }),
            Currency::new(1, String::from(""))
        );
    }

    #[test]
    fn currency_new_symbol_too_long() {
        assert_eq!(
            Err(CurrencyValidationError::SymbolToLong { max: 5, actual: 10 }),
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
            Err(LotValidationError::AccountToShort { min: 1, actual: 0 }),
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
            Err(LotValidationError::AccountToLong {
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
            Err(LotValidationError::SymbolToShort { min: 1, actual: 0 }),
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
            Err(LotValidationError::SymbolToLong { max: 5, actual: 6 }),
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
