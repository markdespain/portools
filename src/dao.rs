use crate::model::Lot;
use futures_util::TryStreamExt;
use mongodb::error::Error;
use mongodb::results::InsertManyResult;
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};

const DB_NAME: &str = "portools";
const COLL_NAME: &str = "lots";

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
        // toto: remove expect
        .expect("creating an index should succeed");
}

pub async fn put_lots(client: &Client, lots: &Vec<Lot>) -> Result<InsertManyResult, Error> {
    let collection: Collection<Lot> = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_many(lots, None).await;
    match result {
        Ok(result) => Ok(result),
        Err(err) => Err(err),
    }
}

pub async fn get_lots(client: &Client) -> Result<Vec<Lot>, Error> {
    let mut cursor = client
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
