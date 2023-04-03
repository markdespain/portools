use crate::util::{validate_and_trim, ValidationError};
use chrono::naive::NaiveDate;
use rust_decimal::Decimal;
use rusty_money::{iso, Money};
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

    pub fn new(amount: Decimal, symbol: String) -> Result<Currency, ValidationError> {
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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Lot {
    // name of the brokerage account within the lot is held
    account: String,

    // the symbol of the security held
    symbol: String,

    // the date that the lot was purchased
    date_acquired: NaiveDate,

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

    pub fn new_str(
        account: String,
        symbol: String,
        date: String,
        quantity: String,
        cost_basis_amount: String,
    ) -> Result<Lot, ValidationError> {
        let date = NaiveDate::parse_from_str(&date, "%Y/%m/%d")
            .map_err(|error| ValidationError::new(format!("invalid date: {:?}", error)))?;

        let quantity = quantity
            .parse()
            .map_err(|error| ValidationError::new(format!("invalid quantity: {error}")))?;

        // todo: support other currencies
        let cost_basis = Money::from_str(&cost_basis_amount, iso::USD)
            .map_err(|error| ValidationError::new(format!("invalid cost_basis: {:?}", error)))?;
        let cost_basis = Currency::new(*cost_basis.amount(), cost_basis.currency().to_string())
            .map_err(|error| ValidationError::new(format!("invalid cost_basis: {:?}", error)))?;

        Lot::new(account, symbol, date, quantity, cost_basis)
    }

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

        // todo: validate quantity
        // toto: let quantity be a decimal
        // todo: validate cost_basis
        Ok(Lot {
            account,
            symbol,
            date_acquired: date,
            quantity,
            cost_basis,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::portfolio::{Currency, Lot};
    use chrono::naive::NaiveDate;
    use rust_decimal::Decimal;

    // Currency Tests

    #[test]
    fn currency_new() {
        assert_eq!(
            Ok(Currency {
                amount: Decimal::from(1),
                symbol: String::from("USD")
            }),
            Currency::new(Decimal::from_str_exact("1").unwrap(), String::from("USD"))
        );
    }

    #[test]
    fn currency_new_symbol_with_whitespace() {
        assert_eq!(
            Ok(Currency {
                amount: Decimal::from(1),
                symbol: String::from("USD")
            }),
            Currency::new(Decimal::from_str_exact("1").unwrap(), String::from(" USD "))
        );
    }

    #[test]
    fn currency_new_symbol_too_short() {
        assert!(
            Currency::new(Decimal::from_str_exact("1").unwrap(), String::from("")).is_err()
        );
    }

    #[test]
    fn currency_new_symbol_too_long() {
        assert!(
            Currency::new(
                Decimal::from_str_exact("1").unwrap(),
                String::from("US Dollars")
            ).is_err()
        );
    }

    // Lot tests

    #[test]
    fn lot_new_valid() {
        assert_eq!(
            Ok(Lot {
                account: String::from("Taxable"),
                symbol: String::from("VOO"),
                date_acquired: NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                quantity: 6,
                cost_basis: Currency {
                    amount: Decimal::from_str_exact("300.64").unwrap(),
                    symbol: String::from("USD")
                }
            }),
            Lot::new(
                String::from("Taxable"),
                String::from("VOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(
                    Decimal::from_str_exact("300.64").unwrap(),
                    String::from("USD")
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn lot_new_account_with_whitespace() {
        assert_eq!(
            Ok(Lot {
                account: String::from("Taxable"),
                symbol: String::from("VOO"),
                date_acquired: NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                quantity: 6,
                cost_basis: Currency {
                    amount: Decimal::from_str_exact("300.64").unwrap(),
                    symbol: String::from("USD")
                }
            }),
            Lot::new(
                String::from(" Taxable "),
                String::from("VOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(
                    Decimal::from_str_exact("300.64").unwrap(),
                    String::from("USD")
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn lot_new_symbol_with_whitespace() {
        assert_eq!(
            Ok(Lot {
                account: String::from("Taxable"),
                symbol: String::from("VOO"),
                date_acquired: NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                quantity: 6,
                cost_basis: Currency {
                    amount: Decimal::from_str_exact("300.64").unwrap(),
                    symbol: String::from("USD")
                }
            }),
            Lot::new(
                String::from("Taxable"),
                String::from(" VOO "),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(
                    Decimal::from_str_exact("300.64").unwrap(),
                    String::from("USD")
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn lot_new_account_too_short() {
        assert!(
            Lot::new(
                String::from(""),
                String::from("VOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(
                    Decimal::from_str_exact("300.64").unwrap(),
                    String::from("USD")
                )
                .unwrap()
            ).is_err()
        );
    }

    #[test]
    fn lot_new_account_too_long() {
        let account: String = (0..101).map(|_| "X").collect();
        assert!(
            Lot::new(
                account,
                String::from("VOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(
                    Decimal::from_str_exact("300.64").unwrap(),
                    String::from("USD")
                )
                .unwrap()
            ).is_err()
        );
    }

    #[test]
    fn lot_new_symbol_too_short() {
        assert!(
            Lot::new(
                String::from("Taxable"),
                String::from(""),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(
                    Decimal::from_str_exact("300.64").unwrap(),
                    String::from("USD")
                )
                .unwrap()
            ).is_err()
        );
    }

    #[test]
    fn lot_new_symbol_too_long() {
        assert!(
            Lot::new(
                String::from("Taxable"),
                String::from("VOODOO"),
                NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                6,
                Currency::new(
                    Decimal::from_str_exact("300.64").unwrap(),
                    String::from("USD")
                ).unwrap()
            ).is_err()
        );
    }
}
