use mongodb::bson::{doc, Bson};
use mongodb::error::Error;
use mongodb::options::{FindOneAndReplaceOptions, FindOneOptions, ReadConcern, WriteConcern};
use mongodb::Database;
use mongodm::{sync_indexes, CollectionConfig, Model, ToRepository};

pub trait Record<I>: Model {
    fn id_field() -> &'static str;

    fn id(record: &Self) -> I;
}

pub async fn drop_and_create<M: Model>(db: &Database) -> Result<(), Error> {
    db.collection::<M>(M::CollConf::collection_name())
        .drop(None)
        .await?;
    sync_indexes::<M::CollConf>(db).await
}

pub async fn upsert<R, I>(database: &Database, record: &R) -> Result<(), Error>
where
    R: Record<I>,
    I: Into<Bson>,
{
    let filter = doc! { R::id_field() : R::id(record).into() };
    let options = FindOneAndReplaceOptions::builder()
        .upsert(Some(true))
        .write_concern(Some(WriteConcern::MAJORITY))
        .build();
    match database
        .repository::<R>()
        .find_one_and_replace(filter, record, options)
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

pub async fn find_by_id<R, I>(database: &Database, id: I) -> Result<Option<R>, Error>
where
    R: Record<I> + Send + Sync,
    I: Into<Bson>,
{
    let filter = doc! { R::id_field() : id.into() };
    let options = FindOneOptions::builder()
        .read_concern(Some(ReadConcern::MAJORITY))
        .build();
    database
        .repository::<R>()
        .find_one(filter, Some(options))
        .await
}
