use crate::dao::Dao;
use crate::model::{AssetAllocation, Portfolio, Record};
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::FindOneAndReplaceOptions;
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing;

const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";
const COLL_ASSET_ALLOC: &str = "asset_alloc";

#[derive(Clone, Debug)]
pub struct MongoDao {
    client: Client,
}

impl MongoDao {
    pub fn new(client: Client) -> MongoDao {
        MongoDao { client }
    }

    #[tracing::instrument(
    skip(self, item),
    fields(id = item.id())
    )]
    async fn put<T: Record + Serialize + DeserializeOwned>(
        &self,
        collection: &str,
        item: &T,
    ) -> Result<(), Error> {
        let filter = doc! {"id": item.id()};
        let options = FindOneAndReplaceOptions::builder()
            .upsert(Some(true))
            .build();
        let collection: Collection<T> = self.client.database(DB_NAME).collection::<T>(collection);
        match collection.find_one_and_replace(filter, item, options).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl Dao for MongoDao {
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), Error> {
        self.put(COLL_PORTFOLIO, portfolio).await
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

    async fn put_asset_allocation(&self, asset_allocation: &AssetAllocation) -> Result<(), Error> {
        self.put(COLL_ASSET_ALLOC, asset_allocation).await
    }
}

#[tracing::instrument(skip(client))]
pub async fn drop_and_create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    mongo_util::drop_and_create_collection_and_index::<Portfolio>(client, DB_NAME, COLL_PORTFOLIO)
        .await?;
    mongo_util::drop_and_create_collection_and_index::<AssetAllocation>(
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
    mongo_util::create_collection_and_index_if_not_exist::<AssetAllocation>(
        client,
        DB_NAME,
        COLL_ASSET_ALLOC,
    )
    .await
}
