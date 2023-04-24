use crate::model::{AssetClass, Portfolio, PortfolioSummary};
use async_trait::async_trait;
use mongodb::error::Error;

pub mod local;
pub mod mongo;

#[async_trait]
pub trait Dao: Send + Sync {
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), Error>;

    async fn get_portfolio(&self, id: u32) -> Result<Option<Portfolio>, Error>;

    async fn put_summary_by_asset_class(
        &self,
        asset_allocation: &PortfolioSummary<AssetClass>,
    ) -> Result<(), Error>;
}
