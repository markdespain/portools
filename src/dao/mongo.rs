use crate::dao::Dao;
use crate::model::Portfolio;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::{FindOneAndReplaceOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};

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
        // should be an upsert
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

pub async fn drop_and_create_collections_and_indexes(client: &Client) {
    drop_and_create_collection_and_index::<Portfolio>(client, COLL_PORTFOLIO).await;
}

pub async fn create_collections_and_indexes(client: &Client) {
    create_collection_and_index::<Portfolio>(client, COLL_PORTFOLIO).await;
}

async fn create_collection_and_index<T>(client: &Client, collection: &str) {
    let db = client.database(DB_NAME);

    db.create_collection(collection, None)
        .await
        .expect("creating a collection should succeed");

    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "id": 1 })
        .options(options)
        .build();
    db.collection::<T>(collection)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

async fn drop_and_create_collection_and_index<T>(client: &Client, collection_name: &str) {
    client
        .database(DB_NAME)
        .collection::<T>(collection_name)
        .drop(None)
        .await
        .expect("dropping the collection should succeed");

    create_collection_and_index::<T>(client, collection_name).await;
}
