use mongodb::error::Error;
use mongodb::Database;
use mongodm::{sync_indexes, CollectionConfig};

pub trait TypedCollectionConfig<S>: CollectionConfig {}

pub async fn drop_and_create<Struct, Config: TypedCollectionConfig<Struct>>(
    db: &Database,
) -> Result<(), Error> {
    db.collection::<Struct>(Config::collection_name())
        .drop(None)
        .await?;
    sync_indexes::<Config>(&db).await
}
