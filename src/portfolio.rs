use crate::util::{trim_and_validate_len, validate_positive, ValidationError};
use chrono::naive::NaiveDate;
use rust_decimal::Decimal;
use rusty_money::{iso, FormattableCurrency, Money};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Currency {
    // the amount of the currency
    amount: Decimal,

    // the symbol for the currency (e.g. "USD")
    symbol: String,
}

impl Currency {
    const MIN_SYMBOL_LEN: usize = 1;
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn new(amount: Decimal, symbol: &str) -> Result<Currency, ValidationError> {
        let symbol = trim_and_validate_len(
            "symbol",
            symbol,
            Currency::MIN_SYMBOL_LEN,
            Currency::MAX_SYMBOL_LEN,
        )?;
        Ok(Currency { amount, symbol })
    }
}

// a Lot is an amount of securities purchased on a particular date
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Lot {
    // name of the brokerage account within the lot is held
    account: String,

    // the symbol of the security held
    symbol: String,

    // the date that the lot was purchased
    date_acquired: NaiveDate,

    // the number of shares of the security in this lot.
    // Decimal is used in order to support fractional shares
    quantity: Decimal,

    // the per-share cost purchase price of this lot
    cost_basis: Currency,
}

impl Lot {
    const MIN_ACCOUNT_LEN: usize = 1;
    const MAX_ACCOUNT_LEN: usize = 100;

    const MIN_SYMBOL_LEN: usize = 1;
    const MAX_SYMBOL_LEN: usize = 5;

    pub fn from_str(
        account: &str,
        symbol: &str,
        date: &str,
        quantity: &str,
        cost_basis_amount: &str,
    ) -> Result<Lot, ValidationError> {
        let date = NaiveDate::parse_from_str(date, "%Y/%m/%d")
            .map_err(|error| ValidationError::new(format!("invalid date: {:?}", error)))?;

        let quantity = quantity
            .parse()
            .map_err(|error| ValidationError::new(format!("invalid quantity: {error}")))?;

        // todo: support other currencies
        let cost_basis = Money::from_str(&cost_basis_amount, iso::USD)
            .map_err(|error| ValidationError::new(format!("invalid cost_basis: {:?}", error)))?;
        let cost_basis = Currency::new(*cost_basis.amount(), cost_basis.currency().code())
            .map_err(|error| ValidationError::new(format!("invalid cost_basis: {:?}", error)))?;

        Lot::new(account, symbol, date, quantity, cost_basis)
    }

    pub fn new(
        account: &str,
        symbol: &str,
        date_acquired: NaiveDate,
        quantity: Decimal,
        cost_basis: Currency,
    ) -> Result<Lot, ValidationError> {
        let account = trim_and_validate_len(
            "account",
            account,
            Lot::MIN_ACCOUNT_LEN,
            Lot::MAX_ACCOUNT_LEN,
        )?;
        let symbol =
            trim_and_validate_len("symbol", symbol, Lot::MIN_SYMBOL_LEN, Lot::MAX_SYMBOL_LEN)?;
        validate_positive("quantity", &quantity)?;
        validate_positive("cost_basis", &cost_basis.amount)?;
        Ok(Lot {
            account,
            symbol,
            date_acquired,
            quantity,
            cost_basis,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::portfolio::{Currency, Lot};
    use crate::util::ValidationError;
    use chrono::naive::NaiveDate;
    use rust_decimal::Decimal;

    // Currency Tests

    fn currency_fixture() -> Currency {
        Currency {
            amount: Decimal::from(1),
            symbol: String::from("USD"),
        }
    }

    fn lot_fixture() -> Lot {
        Lot {
            account: String::from("Taxable"),
            symbol: String::from("VOO"),
            date_acquired: NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
            quantity: Decimal::from(6),
            cost_basis: Currency {
                amount: "300.64".parse().unwrap(),
                symbol: String::from("USD"),
            },
        }
    }

    // for testing purposes
    fn new_lot_from_struct(lot: Lot) -> Result<Lot, ValidationError> {
        Lot::new(
            &lot.account,
            &lot.symbol,
            lot.date_acquired,
            lot.quantity,
            lot.cost_basis,
        )
    }

    #[test]
    fn currency_new() {
        assert_eq!(
            Ok(currency_fixture()),
            Currency::new(Decimal::from(1), "USD")
        );
    }

    #[test]
    fn currency_new_symbol_with_whitespace() {
        assert_eq!(
            Ok(currency_fixture()),
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

    // Lot tests

    #[test]
    fn lot_new_valid() {
        assert_eq!(Ok(lot_fixture()), new_lot_from_struct(lot_fixture()));
    }

    #[test]
    fn lot_new_with_negative_quantity() {
        assert!(new_lot_from_struct(Lot {
            quantity: Decimal::from(-1),
            ..lot_fixture()
        })
        .is_err());
    }

    #[test]
    fn lot_new_with_zero_cost_basis() {
        assert!(new_lot_from_struct(Lot {
            cost_basis: Currency::new(Decimal::from(0), "USD").unwrap(),
            ..lot_fixture()
        })
        .is_err());
    }

    #[test]
    fn lot_new_with_negative_cost_basis() {
        assert!(new_lot_from_struct(Lot {
            cost_basis: Currency::new(Decimal::from(-1), "USD").unwrap(),
            ..lot_fixture()
        })
        .is_err());
    }

    #[test]
    fn lot_new_with_zero_quantity() {
        assert!(new_lot_from_struct(Lot {
            quantity: Decimal::ZERO,
            ..lot_fixture()
        })
        .is_err());
    }

    #[test]
    fn lot_new_account_with_whitespace() {
        assert_eq!(
            Ok(lot_fixture()),
            new_lot_from_struct(Lot {
                account: String::from(" Taxable "),
                ..lot_fixture()
            })
        );
    }

    #[test]
    fn lot_new_symbol_with_whitespace() {
        assert_eq!(
            Ok(lot_fixture()),
            new_lot_from_struct(Lot {
                symbol: String::from(" VOO "),
                ..lot_fixture()
            })
        );
    }

    #[test]
    fn lot_new_account_too_short() {
        assert!(new_lot_from_struct(Lot {
            account: String::from(""),
            ..lot_fixture()
        })
        .is_err());
    }

    #[test]
    fn lot_new_account_too_long() {
        assert!(new_lot_from_struct(Lot {
            account: (0..101).map(|_| "X").collect(),
            ..lot_fixture()
        })
        .is_err());
    }

    #[test]
    fn lot_new_symbol_too_short() {
        assert!(new_lot_from_struct(Lot {
            symbol: String::from(""),
            ..lot_fixture()
        })
        .is_err());
    }

    #[test]
    fn lot_new_symbol_too_long() {
        assert!(new_lot_from_struct(Lot {
            symbol: String::from("VOODOO"),
            ..lot_fixture()
        })
        .is_err());
    }
}
