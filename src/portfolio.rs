use chrono::naive::NaiveDate;

#[derive(Debug)]
pub struct Currency {
    // the amount of the currency in its minor units (e.g. cents for "USD")
    amount: u64,

    // the symbol for the currency (e.g. "USD")
    symbol: String,
}

impl Currency {
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn new(amount: u64, symbol: String) -> Result<Currency, CurrencyValidationError> {
        if symbol.len() > Currency::MAX_SYMBOL_LEN {
            return Err(CurrencyValidationError::SymbolToLong {
                max: Currency::MAX_SYMBOL_LEN,
                actual: symbol.len(),
            });
        }
        Ok(Currency { amount, symbol })
    }
}

#[derive(Debug)]
pub enum CurrencyValidationError {
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
    const MAX_ACCOUNT_LEN: usize = 100;
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn new(
        account: String,
        symbol: String,
        date: NaiveDate,
        quantity: u32,
        cost_basis: Currency,
    ) -> Result<Lot, LotValidationError> {
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
    AccountToLong { max: usize, actual: usize },
    SymbolToLong { max: usize, actual: usize },
}
