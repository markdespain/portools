mod currency;
pub use currency::*;

mod lot;
pub use lot::*;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

type Id = u32;

pub trait Record {
    fn id(&self) -> Id;
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
#[non_exhaustive]
pub struct GroupSummary {
    pub cost: Currency,
}

impl GroupSummary {
    pub fn new(cost: Currency) -> Result<GroupSummary, GroupSummaryError> {
        if cost.amount.is_sign_negative() {
            Err(GroupSummaryError::NegativeCost { cost })
        } else {
            Ok(GroupSummary { cost })
        }
    }
}

#[derive(Debug)]
pub enum GroupSummaryError {
    NegativeCost { cost: Currency },
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
    use crate::model::currency::Currency;
    use crate::model::{GroupSummary, Lot, Portfolio, PortfolioSummary};
    use crate::unit_test_util::fixture;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::HashMap;

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
