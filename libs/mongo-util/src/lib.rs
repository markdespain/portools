use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::IndexOptions;
use mongodb::{Client, IndexModel};
use tracing;

pub async fn create_collection_and_index_if_not_exist<T>(
    client: &Client,
    database: &str,
    collection: &str,
) -> Result<(), Error> {
    create_collection_if_not_exists::<T>(client, database, collection).await?;
    let index_name = format!("{}_index", collection);
    create_index_if_not_exists::<T>(client, database, collection, &index_name).await
}

pub async fn create_index_if_not_exists<T>(
    client: &Client,
    database: &str,
    collection: &str,
    index_name: &str,
) -> Result<(), Error> {
    let db = client.database(database);
    let collection = db.collection::<T>(collection);

    let index_names = collection.list_index_names().await?;
    if index_names.contains(&index_name.to_string()) {
        tracing::info!("index exists: {}", index_name);
        Ok(())
    } else {
        tracing::info!("creating index: {}", index_name);
        let model = IndexModel::builder()
            .keys(doc! { "id": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name(Some(index_name.into()))
                    .build(),
            )
            .build();
        collection.create_index(model, None).await?;
        Ok(())
    }
}

async fn create_collection_if_not_exists<T>(
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

pub async fn drop_and_create_collection_and_index<T>(
    client: &Client,
    database: &str,
    collection: &str,
) -> Result<(), Error> {
    client
        .database(database)
        .collection::<T>(collection)
        .drop(None)
        .await?;

    create_collection_and_index_if_not_exist::<T>(client, database, collection).await
}
