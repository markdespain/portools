use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::IndexOptions;
use mongodb::{Client, IndexModel};

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
        println!("index exists: {}", index_name.to_string());
        Ok(())
    } else {
        println!("creating index: {}", index_name.to_string());
        let model = IndexModel::builder()
            .keys(doc! { "id": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name(Some(index_name.to_string()))
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
    collection_name: &str,
) -> Result<(), Error> {
    let db = client.database(database);
    let names = db.list_collection_names(None).await?;
    if !names.contains(&collection_name.to_string()) {
        println!("creating collection {}", collection_name.clone());
        db.create_collection(collection_name, None).await
    } else {
        println!("collection exists: {}", collection_name);
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