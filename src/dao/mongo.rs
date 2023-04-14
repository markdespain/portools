use crate::dao::Dao;
use crate::model::Portfolio;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::options::{FindOneAndReplaceOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};

const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";

#[derive(Clone, Debug)]
pub struct MongoDao {
    client: Client,
}

impl MongoDao {
    pub fn new(client: Client) -> MongoDao {
        MongoDao { client }
    }
}

#[async_trait]
impl Dao for MongoDao {
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), Error> {
        let filter = doc! {"id": portfolio.id};
        let options = FindOneAndReplaceOptions::builder()
            .upsert(Some(true))
            .build();
        let collection: Collection<Portfolio> =
            self.client.database(DB_NAME).collection(COLL_PORTFOLIO);
        match collection
            .find_one_and_replace(filter, portfolio, options)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn get_portfolio(&self, id: u32) -> Result<Option<Portfolio>, Error> {
        let filter = doc! {"id": id};
        self.client
            .database(DB_NAME)
            .collection(COLL_PORTFOLIO)
            .find_one(filter, None)
            .await
    }
}

pub async fn drop_and_create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    drop_and_create_collection_and_index::<Portfolio>(client, COLL_PORTFOLIO).await
}

pub async fn create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    create_collection_and_index_if_not_exist::<Portfolio>(client, COLL_PORTFOLIO).await
}

async fn create_collection_and_index_if_not_exist<T>(
    client: &Client,
    collection_name: &str,
) -> Result<(), Error> {
    create_collection_if_not_exists::<T>(client, collection_name).await?;
    let index_name = format!("{}_index", collection_name);
    create_index_if_not_exists::<T>(client, collection_name, &index_name).await
}

async fn create_index_if_not_exists<T>(
    client: &Client,
    collection_name: &str,
    index_name: &str,
) -> Result<(), Error> {
    let db = client.database(DB_NAME);
    let collection = db.collection::<T>(collection_name);

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
    collection_name: &str,
) -> Result<(), Error> {
    let db = client.database(DB_NAME);
    let names = db.list_collection_names(None).await?;
    if !names.contains(&collection_name.to_string()) {
        println!("creating collection: {}", collection_name.clone());
        db.create_collection(collection_name, None).await
    } else {
        println!("collection exists: {}", collection_name);
        Ok(())
    }
}

async fn drop_and_create_collection_and_index<T>(
    client: &Client,
    collection_name: &str,
) -> Result<(), Error> {
    client
        .database(DB_NAME)
        .collection::<T>(collection_name)
        .drop(None)
        .await?;

    create_collection_and_index_if_not_exist::<T>(client, collection_name).await
}
