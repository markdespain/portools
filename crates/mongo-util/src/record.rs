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

#[tracing::instrument(
skip(client, record),
fields(id = record.id())
)]
pub async fn upsert<T: Record + Serialize + DeserializeOwned>(
    client: &Client,
    database: &str,
    collection: &str,
    record: &T,
) -> Result<(), Error> {
    let filter = doc! {"id": record.id()};
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
