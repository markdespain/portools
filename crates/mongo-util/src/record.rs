use mongodb::bson::{doc, Bson};
use mongodb::error::Error;
use mongodb::options::{FindOneAndReplaceOptions, FindOneOptions, ReadConcern, WriteConcern};
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Trait that for a struct that that be read from and written to a MongoDB collection
///
/// NOTE: Currently, structs implementing this trait should have a field named "id" for it's
/// id field.  Otherwise, the record will be written to the collection without that field, which
/// can cause issues with code assuming that field exists
// todo(): how to address the above limitation?
pub trait Record<I: Into<Bson>> {
    fn id(&self) -> I;
}

const ID_FIELD: &str = "id";

pub async fn upsert<T, I>(
    client: &Client,
    database: &str,
    collection: &str,
    record: &T,
) -> Result<(), Error>
where
    T: Record<I> + Serialize + DeserializeOwned,
    I: Into<Bson>,
{
    let filter = doc! {ID_FIELD: record.id().into()};
    let options = FindOneAndReplaceOptions::builder()
        .upsert(Some(true))
        .write_concern(Some(WriteConcern::MAJORITY))
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

pub async fn find_by_id<T, I>(
    client: &Client,
    database: &str,
    collection: &str,
    id: I,
) -> Result<Option<T>, Error>
where
    T: Record<I> + DeserializeOwned + Send + Sync + Unpin,
    I: Into<Bson>,
{
    let filter = doc! {ID_FIELD: id.into()};
    let options = FindOneOptions::builder()
        .read_concern(Some(ReadConcern::MAJORITY))
        .build();
    client
        .database(database)
        .collection(collection)
        .find_one(filter, Some(options))
        .await
}
