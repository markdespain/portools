use mongodb::bson::{doc, Bson};
use mongodb::error::Error;
use mongodb::options::{FindOneAndReplaceOptions, FindOneOptions, ReadConcern, WriteConcern};
use mongodb::Database;
use mongodm::{sync_indexes, CollectionConfig, Index, IndexOption, Model, ToRepository};

/// Trait that is
/// - has an "id" field that a uniquely identifies a record in persistent storage
/// - is associated with a mongodm::Model, for ODM to and from persistent storage
pub trait Record: Model {
    /// the type of the ID field
    type IdType: Into<Bson>;

    /// returns the name of the ID field within persistent storage
    fn id_field() -> &'static str;

    /// returns the value of the ID field
    fn id(&self) -> Self::IdType;

    /// Returns an Index describing a unique index on this Record's id field.
    ///
    /// This can be provide as part the associated Model's CollectionConfig::indexes()
    fn id_index() -> Index {
        Index::new(Self::id_field()).with_option(IndexOption::Unique)
    }
}

pub async fn drop_and_create<M: Model>(db: &Database) -> Result<(), Error> {
    db.collection::<M>(M::CollConf::collection_name())
        .drop(None)
        .await?;
    sync_indexes::<M::CollConf>(db).await
}

pub async fn upsert<R>(database: &Database, record: &R) -> Result<(), Error>
where
    R: Record,
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
