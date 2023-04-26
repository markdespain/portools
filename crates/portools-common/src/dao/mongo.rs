use crate::dao::Dao;
use crate::model::{AssetClass, Portfolio, PortfolioSummary};
use async_trait::async_trait;
use mongo_util::record;
use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::Client;
use tracing;

const DB_NAME: &str = "portools";
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

    #[tracing::instrument(skip(self))]
    async fn get_portfolio(&self, id: u32) -> Result<Option<Portfolio>, Error> {
        let filter = doc! {"id": id};
        self.client
            .database(DB_NAME)
            .collection(COLL_PORTFOLIO)
            .find_one(filter, None)
            .await
    }

    async fn put_summary_by_asset_class(
        &self,
        asset_allocation: &PortfolioSummary<AssetClass>,
    ) -> Result<(), Error> {
        record::upsert(&self.client, DB_NAME, COLL_ASSET_ALLOC, asset_allocation).await
    }
}

#[tracing::instrument(skip(client))]
pub async fn drop_and_create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    mongo_util::drop_and_create_collection_and_index::<Portfolio>(client, DB_NAME, COLL_PORTFOLIO)
        .await?;
    mongo_util::drop_and_create_collection_and_index::<PortfolioSummary<AssetClass>>(
        client,
        DB_NAME,
        COLL_ASSET_ALLOC,
    )
    .await
}

#[tracing::instrument(skip(client))]
pub async fn create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    mongo_util::create_collection_and_index_if_not_exist::<Portfolio>(
        client,
        DB_NAME,
        COLL_PORTFOLIO,
    )
    .await?;
    mongo_util::create_collection_and_index_if_not_exist::<PortfolioSummary<AssetClass>>(
        client,
        DB_NAME,
        COLL_ASSET_ALLOC,
    )
    .await
}
