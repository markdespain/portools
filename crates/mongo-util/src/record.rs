use mongodb::bson::{Bson, doc};
use mongodb::error::Error;
use mongodb::options::FindOneAndReplaceOptions;
use mongodb::{Client, Collection};
use serde::de::DeserializeOwned;
use serde::Serialize;

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
where T: Record<I> + Serialize + DeserializeOwned, I: Into<Bson>
{
    let filter = doc! {ID_FIELD: record.id().into()};
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

pub async fn find_by_id<T, I>(
    client: &Client,
    database: &str,
    collection: &str,
    id: I,
) -> Result<Option<T>, Error>
where T: Record<I> + DeserializeOwned + Send + Sync + Unpin, I : Into<Bson>
{
    let filter = doc! {ID_FIELD: id.into()};
    client
        .database(database)
        .collection(collection)
        .find_one(filter, None)
        .await
}
