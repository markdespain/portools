use std::fmt::Error;

const MAX_ACCOUNT_LEN: usize = 100;
const MAX_SYMBOL_LEN: usize = 5;

// a Lot an amount of securities purchased as a particular time
#[derive(Debug)]
pub struct Lot {
    // the account within the lot is held
    account: String,

    // the symbol of the security held
    symbol: String,

    // the date that the lot was purchased
    // TOOD: replace with a better "data" data type
    date: String,

    // the number of shares of the security in this lot
    quantity: u32,

    // the per-share cost purchase price of this lot
    // TOOD: add validation (can't be negative, infinity, NaN, etc.)
    // TODO: replace with "money" type?
    cost_basis: f32,
}

impl Lot {
    pub fn new(
        account: String,
        symbol: String,
        date: String,
        quantity: u32,
        cost_basis: f32,
    ) -> Result<Lot, ValidationError> {
        if account.len() > MAX_ACCOUNT_LEN {
            return Err(ValidationError::AccountNameToLong {
                max: MAX_ACCOUNT_LEN,
                actual: account.len(),
            });
        }
        if symbol.len() > MAX_SYMBOL_LEN {
            return Err(ValidationError::SymbolToLong {
                max: MAX_SYMBOL_LEN,
                actual: symbol.len(),
            });
        }
        if !is_finite_non_neg(cost_basis) {
            return Err(ValidationError::InvalidCostBasis { actual: cost_basis });
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

fn is_finite_non_neg(v: f32) -> bool {
    f32::is_finite(v) && v >= 0.0
}

#[derive(Debug)]
pub enum ValidationError {
    AccountNameToLong { max: usize, actual: usize },
    SymbolToLong { max: usize, actual: usize },
    InvalidCostBasis { actual: f32 },
}
