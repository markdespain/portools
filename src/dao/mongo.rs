use crate::dao::Dao;
use crate::model::Portfolio;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::{FindOneAndReplaceOptions};
use mongodb::{Client, Collection};

const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";

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
        let filter = doc! {"id": portfolio.id};
        let options = FindOneAndReplaceOptions::builder()
            .upsert(Some(true))
            .build();
        let collection: Collection<Portfolio> =
            self.client.database(DB_NAME).collection(COLL_PORTFOLIO);
        match collection
            .find_one_and_replace(filter, portfolio, options)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn get_portfolio(&self, id: u32) -> Result<Option<Portfolio>, Error> {
        let filter = doc! {"id": id};
        self.client
            .database(DB_NAME)
            .collection(COLL_PORTFOLIO)
            .find_one(filter, None)
            .await
    }
}

pub async fn drop_and_create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    mongo_util::drop_and_create_collection_and_index::<Portfolio>(client, DB_NAME, COLL_PORTFOLIO)
        .await
}

pub async fn create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    mongo_util::create_collection_and_index_if_not_exist::<Portfolio>(
        client,
        DB_NAME,
        COLL_PORTFOLIO,
    )
    .await
}
