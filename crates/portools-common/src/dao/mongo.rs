use crate::dao::Dao;
use crate::model::{AssetClass, Portfolio, PortfolioSummary};
use async_trait::async_trait;
use mongo_util::record;
use mongodb::error::Error;
use mongodb::{Client, Database};

use mongo_util::mongodm::{drop_and_create, TypedCollectionConfig};
use mongodm::field;
use mongodm::mongo::{bson::doc, options::ClientOptions};
use mongodm::{sync_indexes, CollectionConfig, Index, IndexOption, Indexes, Model, ToRepository};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";
const PORTFOLIO_BY_ASSET_CLASS: &str = "portfolio_by_asset_class";

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
        record::upsert(
            &self.client,
            DB_NAME,
            PORTFOLIO_BY_ASSET_CLASS,
            asset_allocation,
        )
        .await
    }
}

pub async fn drop_and_create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    let db = client.database(DB_NAME);
    drop_and_create::<Portfolio, PortfolioConfig>(&db).await?;
    drop_and_create::<PortfolioSummary<AssetClass>, PortfolioByAssetClassConfig>(&db).await
}

pub async fn create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    let db = client.database(DB_NAME);
    sync_indexes::<PortfolioConfig>(&db).await?;
    sync_indexes::<PortfolioByAssetClassConfig>(&db).await
}

struct PortfolioConfig;
impl CollectionConfig for PortfolioConfig {
    fn collection_name() -> &'static str {
        COLL_PORTFOLIO
    }

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new(field!(id in Portfolio)).with_option(IndexOption::Unique))
        // field! macro can be used as well
    }
}
impl TypedCollectionConfig<Portfolio> for PortfolioConfig {}

struct PortfolioByAssetClassConfig;
impl CollectionConfig for PortfolioByAssetClassConfig {
    fn collection_name() -> &'static str {
        PORTFOLIO_BY_ASSET_CLASS
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new(field!(id in PortfolioSummary<AssetClass>)).with_option(IndexOption::Unique),
        ) // field! macro can be used as well
    }
}
impl TypedCollectionConfig<PortfolioSummary<AssetClass>> for PortfolioByAssetClassConfig {}
