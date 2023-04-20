use crate::model::{AssetAllocation, Portfolio};
use async_trait::async_trait;
use mongodb::error::Error;

pub mod local;
pub mod mongo;

#[async_trait]
pub trait Dao: Send + Sync {
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), Error>;

    async fn get_portfolio(&self, id: u32) -> Result<Option<Portfolio>, Error>;

    async fn put_asset_allocation(&self, asset_allocation: &AssetAllocation) -> Result<(), Error>;
}
