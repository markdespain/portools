use crate::dao::Dao;
use crate::model::{AssetClass, Portfolio, PortfolioSummary};
use async_trait::async_trait;
use mongo_util::record;
use mongodb::error::Error;
use mongodb::Client;

use mongodm::{ToRepository, Model, CollectionConfig, Indexes, Index, IndexOption, sync_indexes};
use mongodm::mongo::{options::ClientOptions, bson::doc};
use serde::{Serialize, Deserialize};
use std::borrow::Cow;
use mongodm::field;

pub const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";
const COLL_PORTFOLIO_BY_ASSET_CLASS: &str = "portfolio_by_asset_class";

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
        record::upsert(&self.client, DB_NAME, COLL_PORTFOLIO_BY_ASSET_CLASS, asset_allocation).await
    }
}

pub async fn drop_and_create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    record::drop_and_create_collection_and_index::<u32, Portfolio>(client, DB_NAME, COLL_PORTFOLIO)
        .await?;
    record::drop_and_create_collection_and_index::<u32, PortfolioSummary<AssetClass>>(
        client,
        DB_NAME,
        COLL_PORTFOLIO_BY_ASSET_CLASS,
    )
    .await
}

pub struct PortfolioCollConf;
impl CollectionConfig for PortfolioCollConf {
    fn collection_name() -> &'static str { COLL_PORTFOLIO }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new(field!(id in PortfolioSummary<AssetClass>)).with_option(IndexOption::Unique))
    }
}
impl Model for Portfolio {
    type CollConf = PortfolioCollConf;
}

pub struct PortfolioByAssetClassCollConf;
impl CollectionConfig for PortfolioByAssetClassCollConf {
    fn collection_name() -> &'static str { COLL_PORTFOLIO_BY_ASSET_CLASS }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new(field!(id in PortfolioSummary<AssetClass>)).with_option(IndexOption::Unique))
    }
}
impl Model for PortfolioSummary<AssetClass> {
    type CollConf = PortfolioByAssetClassCollConf;
}

pub async fn create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    let db = client.database(DB_NAME);
    sync_indexes::<PortfolioCollConf>(&db).await?;
    sync_indexes::<PortfolioByAssetClassCollConf>(&db).await
}


