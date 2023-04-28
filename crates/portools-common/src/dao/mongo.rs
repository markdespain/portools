use crate::dao::Dao;
use crate::model::{AssetClass, Portfolio, PortfolioSummary};
use async_trait::async_trait;
use mongo_util::record;
use mongodb::error::Error;
use mongodb::Client;

use mongo_util::record::{drop_and_create, Record};
use mongodm::field;
use mongodm::{sync_indexes, CollectionConfig, Index, IndexOption, Indexes, Model};

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
        let database = self.client.database(DB_NAME);
        record::upsert(&database, portfolio).await
    }

    async fn get_portfolio(&self, id: u32) -> Result<Option<Portfolio>, Error> {
        let database = self.client.database(DB_NAME);
        record::find_by_id(&database, id).await
    }

    async fn put_summary_by_asset_class(
        &self,
        asset_allocation: &PortfolioSummary<AssetClass>,
    ) -> Result<(), Error> {
        let database = self.client.database(DB_NAME);
        record::upsert(&database, asset_allocation).await
    }
}

pub async fn drop_and_create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    let db = client.database(DB_NAME);
    drop_and_create::<Portfolio>(&db).await?;
    drop_and_create::<PortfolioSummary<AssetClass>>(&db).await
}

pub async fn create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    let db = client.database(DB_NAME);
    sync_indexes::<PortfolioConfig>(&db).await?;
    sync_indexes::<PortfolioByAssetClassConfig>(&db).await
}

pub struct PortfolioConfig;
impl CollectionConfig for PortfolioConfig {
    fn collection_name() -> &'static str {
        COLL_PORTFOLIO
    }

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new(field!(id in Portfolio)).with_option(IndexOption::Unique))
        // field! macro can be used as well
    }
}
impl Model for Portfolio {
    type CollConf = PortfolioConfig;
}
impl Record for Portfolio {
    type IdType = u32;

    fn id_field() -> &'static str {
        field!(id in Portfolio)
    }

    fn id(&self) -> u32 {
        self.id
    }
}

pub struct PortfolioByAssetClassConfig;
impl CollectionConfig for PortfolioByAssetClassConfig {
    fn collection_name() -> &'static str {
        PORTFOLIO_BY_ASSET_CLASS
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new(field!(id in PortfolioSummary<AssetClass>)).with_option(IndexOption::Unique),
        )
    }
}
impl Model for PortfolioSummary<AssetClass> {
    type CollConf = PortfolioByAssetClassConfig;
}

impl Record for PortfolioSummary<AssetClass> {
    type IdType = u32;

    fn id_field() -> &'static str {
        field!(id in PortfolioSummary<AssetClass>)
    }

    fn id(&self) -> u32 {
        self.id
    }
}
