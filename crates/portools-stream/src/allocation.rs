use mongodb::error::Error;
use portools_common::dao::Dao;
use portools_common::model::{AssetClass, Lot, Portfolio, PortfolioSummary, PortfolioSummaryError};

pub struct AllocationService {
    pub dao: Box<dyn Dao>,
}

impl AllocationService {
    // todo(): integration test coverage
    pub async fn put_summary_by_asset_class(
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
}

// todo: add unit tests
pub fn get_summary_by_asset_class(
    portfolio: &Portfolio,
) -> Result<PortfolioSummary<AssetClass>, PortfolioSummaryError> {
    portfolio.get_summary_by(get_asset_class)
}

// todo: add unit tests
fn get_asset_class(lot: &Lot) -> AssetClass {
    // todo: should not be hard-coded
    // todo: a percentage of each symbol could be represented by a different asset class
    let symbol = &lot.symbol;
    let asset_class = match &symbol.trim().to_ascii_uppercase()[..] {
        // todo: update hard-coded list with broader list of etfs from Vanguard, Shwab, iShares, etc.
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
    use crate::allocation::get_asset_class;
    use chrono::NaiveDate;
    use portools_common::model::{AssetClass, Currency, Lot};
    use rust_decimal::Decimal;

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
}
