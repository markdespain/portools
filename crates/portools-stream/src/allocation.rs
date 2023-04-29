use mongodb::error::Error;
use portools_common::dao::Dao;
use portools_common::model::{AssetClass, Lot, Portfolio, PortfolioSummary, PortfolioSummaryError};

pub struct PortfolioSummaryManager {
    pub dao: Box<dyn Dao>,
}

impl PortfolioSummaryManager {

    // todo(): integration test coverage

    pub async fn summarize(
        &self,
        portfolio: &Portfolio,
    ) -> Result<(), AllocationServiceError> {
        self.summarize_by_asset_class(portfolio).await?;
        self.summarize_by_symbol(portfolio).await
    }

    pub async fn summarize_by_asset_class(
        &self,
        portfolio: &Portfolio,
    ) -> Result<(), AllocationServiceError> {
        let summary = portfolio
            .get_summary_by(get_asset_class)
            .map_err(|cause| AllocationServiceError::PortfolioSummaryError { cause })?;
        self.dao
            .put_summary_by_asset_class(&summary)
            .await
            .map_err(|cause| AllocationServiceError::DataAccessError { cause })
    }

    pub async fn summarize_by_symbol(
        &self,
        portfolio: &Portfolio,
    ) -> Result<(), AllocationServiceError> {
        let summary = portfolio
            .get_summary_by_symbol()
            .map_err(|cause| AllocationServiceError::PortfolioSummaryError { cause })?;
        self.dao
            .put_summary_by_symbol(&summary)
            .await
            .map_err(|cause| AllocationServiceError::DataAccessError { cause })
    }
}

pub fn get_summary_by_asset_class(
    portfolio: &Portfolio,
) -> Result<PortfolioSummary<AssetClass>, PortfolioSummaryError> {
    portfolio.get_summary_by(get_asset_class)
}

fn get_asset_class(lot: &Lot) -> AssetClass {
    // todo:
    //   - update list with broader list of etfs from Vanguard, Shwab, iShares, etc.
    //   - should not be hard-coded
    //   - future: a percentage of each symbol could be represented by a different asset class
    let symbol = &lot.symbol;
    let asset_class = match &symbol.trim().to_ascii_uppercase()[..] {
        "VOO" | "VTI" => AssetClass::UsStocks,
        "VEA" | "VEU" => AssetClass::IntlStocks,

        "VNQ" => AssetClass::UsRealEstate,
        "VNQI" => AssetClass::IntlRealEstate,

        "BND" => AssetClass::UsBonds,
        "BNDX" => AssetClass::IntlBonds,
        _ => AssetClass::Unknown,
    };
    asset_class
}

#[derive(Debug)]
pub enum AllocationServiceError {
    PortfolioSummaryError { cause: PortfolioSummaryError },
    DataAccessError { cause: Error },
}

#[cfg(test)]
mod tests {
    use crate::allocation::{get_asset_class, get_summary_by_asset_class};
    use chrono::NaiveDate;
    use portools_common::model::{
        AssetClass, Currency, GroupSummary, Lot, Portfolio, PortfolioSummary,
    };
    use rust_decimal::Decimal;
    use std::collections::HashMap;

    pub fn lot_for_symbol(symbol: &str) -> Lot {
        Lot::new(
            "Taxable",
            symbol,
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(6),
            Currency::new("300.64".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn test_get_asset_class() {
        assert_eq!(
            AssetClass::UsStocks,
            get_asset_class(&lot_for_symbol("VOO"))
        );
        assert_eq!(
            AssetClass::UsStocks,
            get_asset_class(&lot_for_symbol("VTI"))
        );

        assert_eq!(
            AssetClass::UsRealEstate,
            get_asset_class(&lot_for_symbol("VNQ"))
        );

        assert_eq!(
            AssetClass::IntlRealEstate,
            get_asset_class(&lot_for_symbol("VNQI"))
        );

        assert_eq!(AssetClass::UsBonds, get_asset_class(&lot_for_symbol("BND")));

        assert_eq!(
            AssetClass::IntlBonds,
            get_asset_class(&lot_for_symbol("BNDX"))
        );

        // not yet supported
        assert_eq!(
            AssetClass::Unknown,
            get_asset_class(&lot_for_symbol("SCHB"))
        );
    }

    #[test]
    fn test_get_asset_class_with_lowercase_and_padded_symbol() {
        assert_eq!(
            AssetClass::UsStocks,
            get_asset_class(&lot_for_symbol("  voo  "))
        );
    }

    #[test]
    fn get_summary_by_asset_class_with_one_lot() {
        let id = 1;
        let lot = lot_for_symbol("VOO");
        let portfolio = Portfolio {
            id,
            lots: vec![lot.clone()],
        };

        let allocation = get_summary_by_asset_class(&portfolio).unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([(
                    AssetClass::UsStocks,
                    GroupSummary::new(lot.get_total_cost().unwrap()).unwrap()
                )])
            },
            allocation
        )
    }

    #[test]
    fn get_summary_by_asset_class_with_two_lots_with_same_symbol() {
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
        let allocation = get_summary_by_asset_class(&portfolio).unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([(
                    AssetClass::UsStocks,
                    GroupSummary::new(
                        lot_1
                            .get_total_cost()
                            .unwrap()
                            .add(&lot_2.get_total_cost().unwrap())
                            .unwrap()
                    )
                    .unwrap()
                )])
            },
            allocation
        )
    }

    #[test]
    fn portfolio_get_allocation_by_symbol_with_two_lots_with_same_asset_class_different_symbols() {
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

        let portfolio = Portfolio {
            id,
            lots: vec![lot_1.clone(), lot_2.clone()],
        };
        let allocation = get_summary_by_asset_class(&portfolio).unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([(
                    AssetClass::UsStocks,
                    GroupSummary::new(
                        lot_1
                            .get_total_cost()
                            .unwrap()
                            .add(&lot_2.get_total_cost().unwrap())
                            .unwrap()
                    )
                    .unwrap()
                )])
            },
            allocation
        )
    }

    #[test]
    fn get_summary_by_asset_class_with_two_lots_with_different_asset_classes() {
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
            "IRA",
            "BND",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(6),
            Currency::new("300.64".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();

        let portfolio = Portfolio {
            id,
            lots: vec![lot_1.clone(), lot_2.clone()],
        };
        let allocation = get_summary_by_asset_class(&portfolio).unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([
                    (
                        AssetClass::UsStocks,
                        GroupSummary::new(lot_1.get_total_cost().unwrap()).unwrap()
                    ),
                    (
                        AssetClass::UsBonds,
                        GroupSummary::new(lot_2.get_total_cost().unwrap()).unwrap()
                    ),
                ])
            },
            allocation
        );
    }

    #[test]
    fn get_summary_by_asset_class_lots_with_a_shared_asset_class() {
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
            "VEA",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(2),
            Currency::new("200.00".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();
        let lot_3 = Lot::new(
            "IRA",
            "VTI",
            NaiveDate::from_ymd_opt(2023, 3, 27).unwrap(),
            Decimal::from(3),
            Currency::new("300.00".parse().unwrap(), "USD").unwrap(),
        )
        .unwrap();

        let portfolio = Portfolio {
            id,
            lots: vec![lot_1.clone(), lot_2.clone(), lot_3.clone()],
        };
        let allocation = get_summary_by_asset_class(&portfolio).unwrap();
        assert_eq!(
            PortfolioSummary {
                id,
                group_to_summary: HashMap::from([
                    (
                        AssetClass::UsStocks,
                        GroupSummary::new(
                            lot_1
                                .get_total_cost()
                                .unwrap()
                                .add(&lot_3.get_total_cost().unwrap())
                                .unwrap()
                        )
                        .unwrap()
                    ),
                    (
                        AssetClass::IntlStocks,
                        GroupSummary::new(lot_2.get_total_cost().unwrap()).unwrap()
                    ),
                ])
            },
            allocation
        );
    }
}
