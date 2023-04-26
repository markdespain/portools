use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::FindOneAndReplaceOptions;
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub type Id = u32;

pub trait Record {
    fn id(&self) -> Id;
}

const ID_FIELD: &str = "id";

pub async fn upsert<T: Record + Serialize + DeserializeOwned>(
    client: &Client,
    database: &str,
    collection: &str,
    record: &T,
) -> Result<(), Error> {
    let filter = doc! {ID_FIELD: record.id()};
    let options = FindOneAndReplaceOptions::builder()
        .upsert(Some(true))
        .build();
    let collection: Collection<T> = client.database(database).collection::<T>(collection);
    match collection
        .find_one_and_replace(filter, record, options)
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

#[tracing::instrument(skip(client))]
pub async fn find_by_id<T: Record + DeserializeOwned + Send + Sync + Unpin>(
    client: &Client,
    database: &str,
    collection: &str,
    id: u32,
) -> Result<Option<T>, Error> {
    let filter = doc! {ID_FIELD: id};
    client
        .database(database)
        .collection(collection)
        .find_one(filter, None)
        .await
}
