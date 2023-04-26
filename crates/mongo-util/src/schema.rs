use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::IndexOptions;
use mongodb::{Client, IndexModel};

pub async fn create_index_if_not_exists<T>(
    client: &Client,
    database: &str,
    collection: &str,
    index: &str,
    field: &str,
) -> Result<(), Error> {
    let db = client.database(database);
    let collection = db.collection::<T>(collection);

    let index_names = collection.list_index_names().await?;
    if index_names.contains(&index.to_string()) {
        tracing::info!(index, "index exists");
        Ok(())
    } else {
        tracing::info!(index, "creating index");
        let model = IndexModel::builder()
            .keys(doc! { field: 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name(Some(index.into()))
                    .build(),
            )
            .build();
        collection.create_index(model, None).await?;
        Ok(())
    }
}

pub async fn create_collection_if_not_exists<T>(
    client: &Client,
    database: &str,
    collection: &str,
) -> Result<(), Error> {
    let db = client.database(database);
    let names = db.list_collection_names(None).await?;
    if !names.contains(&collection.to_string()) {
        tracing::info!("creating collection {}", collection);
        db.create_collection(collection, None).await
    } else {
        tracing::info!("collection exists: {}", collection);
        Ok(())
    }
}
