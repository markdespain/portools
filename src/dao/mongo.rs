use crate::dao::Dao;
use crate::model::Lot;
use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::IndexOptions;
use mongodb::{Client, Collection, IndexModel};

const DB_NAME: &str = "portools";
const COLL_NAME: &str = "lots";

pub struct MongoDao {
    pub client: Client,
}

#[async_trait]
impl Dao for MongoDao {
    async fn put_lots(&self, lots: &[Lot]) -> Result<(), Error> {
        let collection: Collection<Lot> = self.client.database(DB_NAME).collection(COLL_NAME);
        let result = collection.insert_many(lots, None).await;
        match result {
            Ok(_) => Ok(()), // note: this could return the inserted ids
            Err(err) => Err(err),
        }
    }

    async fn get_lots(&self) -> Result<Vec<Lot>, Error> {
        let mut cursor = self
            .client
            .database(DB_NAME)
            .collection(COLL_NAME)
            .find(None, None)
            .await?;
        let mut result = Vec::new();
        while let Some(lot) = cursor.try_next().await? {
            result.push(lot);
        }
        Ok(result)
    }
}

pub async fn create_lots_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "id": 1 })
        .options(options)
        .build();
    client
        .database(DB_NAME)
        .collection::<Lot>(COLL_NAME)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}
