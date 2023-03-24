// a Lot an amount of securities purchased as a particular time
#[derive(Debug)]
pub struct Lot {
    // the account within the lot is held
    account: String,

    // the symbol of the security held
    // TODO: create a type that has validation?
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
    ) -> Lot {
        Lot {
            account,
            symbol,
            date,
            quantity,
            cost_basis,
        }
    }
}
