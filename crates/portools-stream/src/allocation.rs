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
        "VOO" | "VTI" | "SCHB" | "VTV" => AssetClass::UsStocks,
        "VNQ" => AssetClass::UsRealEstate,
        "VEA" | "VEU" | "SCHF" => AssetClass::IntlStocks,
        "VNQI" => AssetClass::IntlRealEstate,
        "AGG" | "BND" | "VTEB" => AssetClass::UsBonds,
        _ => AssetClass::Unknown,
    };
    asset_class
}

#[derive(Debug)]
pub enum AllocationServiceError {
    PortfolioSummaryError { cause: PortfolioSummaryError },
    DataAccessError { cause: Error },
}
