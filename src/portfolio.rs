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
#[derive(Debug)]
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
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn new(
        account: String,
        symbol: String,
        date: NaiveDate,
        quantity: u32,
        cost_basis: Currency,
    ) -> Result<Lot, LotValidationError> {
        if account.len() < Lot::MIN_ACCOUNT_LEN {
            return Err(LotValidationError::AccountToShort {
                min: Lot::MAX_ACCOUNT_LEN,
                actual: account.len(),
            });
        }
        if account.len() > Lot::MAX_ACCOUNT_LEN {
            return Err(LotValidationError::AccountToLong {
                max: Lot::MAX_ACCOUNT_LEN,
                actual: account.len(),
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

#[derive(Debug)]
pub enum LotValidationError {
    AccountToShort { min: usize, actual: usize },
    AccountToLong { max: usize, actual: usize },
    SymbolToLong { max: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use crate::portfolio::{Currency, CurrencyValidationError};

    #[test]
    fn currency_new() {
        assert_eq!(
            Currency {
                amount: 1,
                symbol: String::from("USD")
            },
            Currency::new(1, String::from("USD")).unwrap()
        )
    }

    #[test]
    fn currency_new_symbol_with_whitespace() {
        assert_eq!(
            Currency {
                amount: 1,
                symbol: String::from("USD")
            },
            Currency::new(1, String::from(" USD ")).unwrap()
        )
    }

    #[test]
    fn currency_new_symbol_too_short() {
        assert_eq!(
            CurrencyValidationError::SymbolToShort { min: 1, actual: 0 },
            Currency::new(1, String::from("")).err().unwrap()
        );
    }

    #[test]
    fn currency_new_symbol_too_long() {
        assert_eq!(
            CurrencyValidationError::SymbolToLong { max: 5, actual: 10 },
            Currency::new(1, String::from("US Dollars")).err().unwrap()
        );
    }
}
