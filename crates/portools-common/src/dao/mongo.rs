use crate::dao::Dao;
use crate::model::{AssetClass, Portfolio, PortfolioSummary};
use async_trait::async_trait;
use mongo_util::record;
use mongodb::error::Error;
use mongodb::Client;

pub const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";
const COLL_ASSET_ALLOC: &str = "portfolio_by_asset_class";

#[derive(Clone, Debug)]
pub struct MongoDao {
    client: Client,
}

impl MongoDao {
    pub fn new(client: Client) -> MongoDao {
        MongoDao { client }
    }
}

#[async_trait]
impl Dao for MongoDao {
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), Error> {
        record::upsert(&self.client, DB_NAME, COLL_PORTFOLIO, portfolio).await
    }

    async fn get_portfolio(&self, id: u32) -> Result<Option<Portfolio>, Error> {
        record::find_by_id(&self.client, DB_NAME, COLL_PORTFOLIO, id).await
    }

    async fn put_summary_by_asset_class(
        &self,
        asset_allocation: &PortfolioSummary<AssetClass>,
    ) -> Result<(), Error> {
        record::upsert(&self.client, DB_NAME, COLL_ASSET_ALLOC, asset_allocation).await
    }
}

pub async fn drop_and_create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    record::drop_and_create_collection_and_index::<u32, Portfolio>(client, DB_NAME, COLL_PORTFOLIO)
        .await?;
    record::drop_and_create_collection_and_index::<u32, PortfolioSummary<AssetClass>>(
        client,
        DB_NAME,
        COLL_ASSET_ALLOC,
    )
    .await
}

pub async fn create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    record::create_collection_and_index_if_not_exist::<u32, Portfolio>(
        client,
        DB_NAME,
        COLL_PORTFOLIO,
    )
    .await?;
    record::create_collection_and_index_if_not_exist::<u32, PortfolioSummary<AssetClass>>(
        client,
        DB_NAME,
        COLL_ASSET_ALLOC,
    )
    .await
}
