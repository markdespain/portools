use crate::validate::{trim_and_validate_len, validate_positive, Invalid};
use chrono::naive::NaiveDate;
use std::collections::HashMap;

use std::fmt::Debug;
use std::hash::Hash;

use rust_decimal::Decimal;
use rusty_money::{iso, FormattableCurrency, Money};
use serde::{Deserialize, Serialize};

type Id = u32;

pub trait Record {
    fn id(&self) -> Id;
}

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: Id,
    pub lots: Vec<Lot>,
}

impl Portfolio {
    pub fn get_summary_by_symbol(&self) -> Result<PortfolioSummary<String>, PortfolioSummaryError> {
        let by_symbol = |lot: &Lot| -> String { lot.symbol.clone() };
        self.get_summary_by(by_symbol)
    }

    pub fn get_summary_by<T: Eq + Hash + Debug>(
        &self,
        classifier: fn(lot: &Lot) -> T,
    ) -> Result<PortfolioSummary<T>, PortfolioSummaryError> {
        if self.lots.is_empty() {
            return Ok(PortfolioSummary {
                id: self.id,
                group_to_summary: HashMap::new(),
            });
        }
        let currency_symbol = &self.lots[0].cost_basis.symbol;
        let mut group_to_summary = HashMap::<T, GroupSummary>::new();
        for lot in self.lots.iter() {
            let class = classifier(lot);
            let entry = group_to_summary
                .entry(class)
                .or_insert_with(|| GroupSummary {
                    cost: Currency {
                        amount: Decimal::ZERO,
                        symbol: currency_symbol.into(),
                    },
                });
            let lot_total_cost = &lot
                .get_total_cost()
                .map_err(|cause| PortfolioSummaryError::LotTotalCostError { cause })?;
            let new_cast = entry
                .cost
                .add(lot_total_cost)
                .map_err(|cause| PortfolioSummaryError::SummaryCostError { cause })?;
            entry.cost = new_cast;
        }
        Ok(PortfolioSummary {
            id: self.id,
            group_to_summary,
        })
    }
}

impl Record for Portfolio {
    fn id(&self) -> Id {
        self.id
    }
}

#[derive(Debug)]
pub enum PortfolioSummaryError {
    LotTotalCostError { cause: CurrencyError<Decimal> },
    SummaryCostError { cause: CurrencyError<Currency> },
}

// a Lot is an amount of securities purchased on a particular date
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Lot {
    // name of the brokerage account within the lot is held
    account: String,

    // the symbol of the security held
    pub symbol: String,

    // the date that the lot was purchased
    date_acquired: NaiveDate,

    // the number of shares of the security in this lot.
    // Decimal is used in order to support fractional shares
    quantity: Decimal,

    // the per-share cost purchase price of this lot
    pub cost_basis: Currency,
}

impl Lot {
    const MIN_ACCOUNT_LEN: usize = 1;
    const MAX_ACCOUNT_LEN: usize = 100;

    const MIN_SYMBOL_LEN: usize = 1;
    const MAX_SYMBOL_LEN: usize = 5;

    const DATE_FORMAT: &'static str = "%Y/%m/%d";

    pub fn from_str(
        account: &str,
        symbol: &str,
        date: &str,
        quantity: &str,
        cost_basis_amount: &str,
    ) -> Result<Lot, Invalid> {
        let date = NaiveDate::parse_from_str(date, Lot::DATE_FORMAT)
            .map_err(|error| Invalid::parse_date_error("date", error))?;

        let quantity = quantity
            .parse()
            .map_err(|error| Invalid::parse_decimal_error("quantity", error))?;

        let cost_basis = Money::from_str(cost_basis_amount, iso::USD)
            .map_err(|error| Invalid::parse_money_error("cost_basis", error))?;
        let cost_basis = Currency::new(*cost_basis.amount(), cost_basis.currency().code())?;

        Lot::new(account, symbol, date, quantity, cost_basis)
    }

    pub fn new(
        account: &str,
        symbol: &str,
        date_acquired: NaiveDate,
        quantity: Decimal,
        cost_basis: Currency,
    ) -> Result<Lot, Invalid> {
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

    // todo(): add unit tests
    pub fn get_total_cost(&self) -> Result<Currency, CurrencyError<Decimal>> {
        self.cost_basis.multiply(&self.quantity)
    }

    #[cfg(test)]
    fn date_acquired_string(&self) -> String {
        format!("{}", self.date_acquired.format(Lot::DATE_FORMAT))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetClass {
    IntlBonds,
    UsBonds,
    IntlRealEstate,
    UsRealEstate,
    UsStocks,
    IntlStocks,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// todo(): add validating constructor and #[non_exhaustive]
pub struct GroupSummary {
    pub cost: Currency,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PortfolioSummary<T: Hash + Eq> {
    pub id: Id,
    pub group_to_summary: HashMap<T, GroupSummary>,
}

impl<T: Hash + Eq> Record for PortfolioSummary<T> {
    fn id(&self) -> Id {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Currency, GroupSummary, Lot, Portfolio, PortfolioSummary};
    use crate::unit_test_util::fixture;
    use crate::validate::Reason::{ParseDateError, ParseDecimalError, ParseMoneyError};
    use crate::validate::{Invalid, Reason};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use test_util;
    use test_util::assertion::{assert_err_eq, assert_is_err, assert_ok_eq};

    // Currency Tests

    fn new_from_spec(lot: Lot) -> Result<Lot, Invalid> {
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

    // Lot tests

    fn assert_new_from_spec(expected: Lot, spec: Lot) {
        let actual = new_from_spec(spec);
        assert_ok_eq(&expected, &actual);
    }

    fn assert_new_from_spec_is_err(spec: Lot, expected_err: Invalid) {
        let actual = new_from_spec(spec);
        assert_err_eq(expected_err, actual);
    }

    fn assert_format_err(expected_field: &str, result: Result<Lot, Invalid>) {
        let actual_err = assert_invalid(expected_field, result);
        match actual_err.reason {
            ParseDecimalError { .. } => {
                // skip assertion of error message, since it may change unexpectedly due
                // to being human readable and due to potentially coming from an external
                // library
            }
            unexpected_error => {
                panic!(
                    "expected reason to be Invalid::FormatError but got: {:?}",
                    unexpected_error
                );
            }
        }
    }

    fn assert_invalid(expected_field: &str, result: Result<Lot, Invalid>) -> Invalid {
        let invalid = assert_is_err(result);
        assert_eq!(
            expected_field, invalid.field,
            "field name should match expected"
        );
        invalid
    }

    fn assert_parse_money_error(expected_field: &str, actual: Result<Lot, Invalid>) {
        let actual_err = assert_invalid(expected_field, actual);
        match actual_err.reason {
            ParseMoneyError { .. } => {
                // skip assertion of error message, since it may change unexpectedly due
                // to being human readable and due to potentially coming from an external
                // library
            }
            unexpected_error => {
                panic!(
                    "expected reason to be ParseMoneyError but got: {:?}",
                    unexpected_error
                );
            }
        }
    }

    fn assert_parse_date_error(expected_field: &str, actual: Result<Lot, Invalid>) {
        let actual_err = assert_invalid(expected_field, actual);
        match actual_err.reason {
            ParseDateError { .. } => {
                // skip assertion of error message, since it may change unexpectedly due
                // to being human readable and due to potentially coming from an external
                // library
            }
            unexpected_error => {
                panic!(
                    "expected reason to be ParseMoneyError but got: {:?}",
                    unexpected_error
                );
            }
        }
    }

    #[test]
    fn lot_from_str_valid() {
        let expected = fixture::lot();
        let lot = Lot::from_str(
            &expected.account,
            &expected.symbol,
            &expected.date_acquired_string(),
            &expected.quantity.to_string(),
            &expected.cost_basis.amount.to_string(),
        );
        assert_ok_eq(&fixture::lot(), &lot)
    }

    #[test]
    fn lot_from_str_with_date_with_invalid_format() {
        let fixture = fixture::lot();
        let date_acquired = format!("{}", fixture.date_acquired.format("%Y-%m-%d"));
        let lot = Lot::from_str(
            &fixture.account,
            &fixture.symbol,
            &date_acquired,
            &fixture.quantity.to_string(),
            &fixture.cost_basis.amount.to_string(),
        );
        assert_parse_date_error("date", lot);
    }

    #[test]
    fn lot_from_str_with_quantity_not_an_decimal() {
        let fixture = fixture::lot();
        let quantity = "not a number";
        let lot = Lot::from_str(
            &fixture.account,
            &fixture.symbol,
            &fixture.date_acquired_string(),
            quantity,
            &fixture.cost_basis.amount.to_string(),
        );
        assert_format_err("quantity", lot);
    }

    #[test]
    fn lot_from_str_with_cost_basis_not_an_number() {
        let fixture = fixture::lot();
        let cost_basis = "not a number";
        let lot = Lot::from_str(
            &fixture.account,
            &fixture.symbol,
            &fixture.date_acquired_string(),
            &fixture.quantity.to_string(),
            cost_basis,
        );
        assert_parse_money_error("cost_basis", lot);
    }

    #[test]
    fn lot_new_valid() {
        assert_new_from_spec(fixture::lot(), fixture::lot());
    }

    #[test]
    fn lot_new_with_negative_quantity() {
        let lot_spec = Lot {
            quantity: Decimal::from(-1),
            ..fixture::lot()
        };
        let expected_error = Invalid {
            field: "quantity".into(),
            reason: Reason::MustBePositive,
        };
        assert_new_from_spec_is_err(lot_spec, expected_error);
    }

    #[test]
    fn lot_new_with_zero_cost_basis() {
        let lot_spec = Lot {
            cost_basis: Currency::new(Decimal::from(0), "USD").unwrap(),
            ..fixture::lot()
        };
        let expected_error = Invalid {
            field: "cost_basis".into(),
            reason: Reason::MustBePositive,
        };
        assert_new_from_spec_is_err(lot_spec, expected_error);
    }

    #[test]
    fn lot_new_with_negative_cost_basis() {
        let lot_spec = Lot {
            cost_basis: Currency::new(Decimal::from(-1), "USD").unwrap(),
            ..fixture::lot()
        };
        let expected_error = Invalid {
            field: "cost_basis".into(),
            reason: Reason::MustBePositive,
        };
        assert_new_from_spec_is_err(lot_spec, expected_error);
    }

    #[test]
    fn lot_new_with_zero_quantity() {
        let lot_spec = Lot {
            quantity: Decimal::ZERO,
            ..fixture::lot()
        };
        let expected_error = Invalid {
            field: "quantity".into(),
            reason: Reason::MustBePositive,
        };
        assert_new_from_spec_is_err(lot_spec, expected_error);
    }

    #[test]
    fn lot_new_account_with_whitespace() {
        assert_new_from_spec(
            fixture::lot(),
            Lot {
                account: " Taxable ".into(),
                ..fixture::lot()
            },
        );
    }

    #[test]
    fn lot_new_symbol_with_whitespace() {
        assert_new_from_spec(
            fixture::lot(),
            Lot {
                symbol: " VOO ".into(),
                ..fixture::lot()
            },
        );
    }

    #[test]
    fn lot_new_account_too_short() {
        let lot_spec = Lot {
            account: "".into(),
            ..fixture::lot()
        };
        let expected_error = Invalid {
            field: "account".into(),
            reason: Reason::MustHaveLongerLen,
        };
        assert_new_from_spec_is_err(lot_spec, expected_error);
    }

    #[test]
    fn lot_new_account_too_long() {
        let lot_spec = Lot {
            account: (0..101).map(|_| "X").collect(),
            ..fixture::lot()
        };

        let expected_error = Invalid {
            field: "account".into(),
            reason: Reason::MustHaveShorterLen,
        };
        assert_new_from_spec_is_err(lot_spec, expected_error);
    }

    #[test]
    fn lot_new_symbol_too_short() {
        let lot_spec = Lot {
            symbol: "".into(),
            ..fixture::lot()
        };
        let expected_error = Invalid {
            field: "symbol".into(),
            reason: Reason::MustHaveLongerLen,
        };
        assert_new_from_spec_is_err(lot_spec, expected_error);
    }

    #[test]
    fn lot_new_symbol_too_long() {
        let lot_spec = Lot {
            symbol: "VOODOO".into(),
            ..fixture::lot()
        };
        let expected_error = Invalid {
            field: "symbol".into(),
            reason: Reason::MustHaveShorterLen,
        };
        assert_new_from_spec_is_err(lot_spec, expected_error);
    }

    #[test]
    fn portfolio_get_allocation_by_symbol_with_empty_lots() {
        let id = 1;
        let portfolio = Portfolio {
            id,
            lots: Vec::new(),
        };

        let allocation = portfolio.get_summary_by_symbol().unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::new()
            },
            allocation
        )
    }

    #[test]
    fn portfolio_get_allocation_by_symbol_with_one_lot() {
        let id = 1;
        let lot = fixture::lot();
        let portfolio = Portfolio {
            id,
            lots: vec![lot.clone()],
        };

        let allocation = portfolio.get_summary_by_symbol().unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([(
                    lot.symbol.clone(),
                    GroupSummary {
                        cost: lot.get_total_cost().unwrap()
                    }
                )])
            },
            allocation
        )
    }

    #[test]
    fn portfolio_get_allocation_by_symbol_with_two_lots_with_same_symbol() {
        let id = 1;
        let lot_1 = Lot::new(
            "Taxable",
            "VOO",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(1),
            Currency::new("100.00".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();
        let lot_2 = Lot::new(
            "Taxable",
            "VOO",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(2),
            Currency::new("200.00".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();

        let portfolio = Portfolio {
            id,
            lots: vec![lot_1.clone(), lot_2.clone()],
        };
        let allocation = portfolio.get_summary_by_symbol().unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([(
                    lot_1.symbol.clone(),
                    GroupSummary {
                        cost: lot_1
                            .get_total_cost()
                            .unwrap()
                            .add(&lot_2.get_total_cost().unwrap())
                            .unwrap()
                    }
                )])
            },
            allocation
        )
    }

    #[test]
    fn portfolio_get_allocation_by_symbol_with_two_lots_with_different_symbols() {
        let id = 1;
        let lot_1 = Lot::new(
            "Taxable",
            "VOO",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(6),
            Currency::new("300.64".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();
        let lot_2 = Lot::new(
            "Taxable",
            "VTI",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(6),
            Currency::new("300.64".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();

        let portfolio = Portfolio {
            id,
            lots: vec![lot_1.clone(), lot_2.clone()],
        };
        let allocation = portfolio.get_summary_by_symbol().unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([
                    (
                        lot_1.symbol.clone(),
                        GroupSummary {
                            cost: lot_1.get_total_cost().unwrap()
                        }
                    ),
                    (
                        lot_2.symbol.clone(),
                        GroupSummary {
                            cost: lot_2.get_total_cost().unwrap()
                        }
                    ),
                ])
            },
            allocation
        );
    }

    #[test]
    fn portfolio_get_allocation_by_symbol_with_three_lots_with_a_shared_symbol() {
        let id = 1;
        let lot_1 = Lot::new(
            "Taxable",
            "VOO",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(1),
            Currency::new("100.00".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();
        let lot_2 = Lot::new(
            "Taxable",
            "VTI",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(2),
            Currency::new("200.00".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();
        let lot_3 = Lot::new(
            "IRA",
            "VOO",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(3),
            Currency::new("300.00".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();

        let portfolio = Portfolio {
            id,
            lots: vec![lot_1.clone(), lot_2.clone(), lot_3.clone()],
        };
        let allocation = portfolio.get_summary_by_symbol().unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([
                    (
                        lot_1.symbol.clone(),
                        GroupSummary {
                            cost: lot_1
                                .get_total_cost()
                                .unwrap()
                                .add(&lot_3.get_total_cost().unwrap())
                                .unwrap()
                        }
                    ),
                    (
                        lot_2.symbol.clone(),
                        GroupSummary {
                            cost: lot_2.get_total_cost().unwrap()
                        }
                    ),
                ])
            },
            allocation
        );
    }
}
