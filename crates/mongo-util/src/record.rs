use mongodb::bson::{doc, Bson};
use mongodb::error::Error;
use mongodb::options::{FindOneAndReplaceOptions, FindOneOptions, ReadConcern, WriteConcern};
use mongodb::Database;
use mongodm::{sync_indexes, CollectionConfig, Model, ToRepository};

pub trait Record : Model {
    type IdType : Into<Bson>;

    fn id_field() -> &'static str;

    fn id(record: &Self) -> Self::IdType;
}

pub async fn drop_and_create<M: Model>(db: &Database) -> Result<(), Error> {
    db.collection::<M>(M::CollConf::collection_name())
        .drop(None)
        .await?;
    sync_indexes::<M::CollConf>(db).await
}

pub async fn upsert<R>(database: &Database, record: &R) -> Result<(), Error>
where
    R: Record
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

pub async fn find_by_id<R>(database: &Database, id: R::IdType) -> Result<Option<R>, Error>
where
    R: Record + Send + Sync,
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
