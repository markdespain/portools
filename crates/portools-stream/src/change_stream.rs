use mongo_util::record;
use mongodb::change_stream::event::ResumeToken;
use mongodb::error::Error;
use mongodb::Client;
use mongodb::options::{FindOneAndReplaceOptions, FindOneOptions, InsertOneOptions, ReadConcern, WriteConcern};
use mongodm::{CollectionConfig, doc, field, Index, Indexes, IndexOption, Model, sync_indexes, ToRepository};
use serde::{Deserialize, Serialize};
use portools_common::dao::mongo::DB_NAME;

const COLL_RESUME_TOKEN: &str = "resume_token";

#[derive(Debug, Serialize, Deserialize)]
struct ResumeTokenRecord {
    // logical identifier for the change stream within an application
    id: String,
    resume_token: Option<ResumeToken>,
}

pub struct ResumeTokenRecordCollConf;
impl CollectionConfig for ResumeTokenRecordCollConf {
    fn collection_name() -> &'static str { COLL_RESUME_TOKEN }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new(field!(id in ResumeTokenRecord)).with_option(IndexOption::Unique))
    }
}
impl Model for ResumeTokenRecord {
    type CollConf = ResumeTokenRecordCollConf;
}

pub async fn create_collections_and_indexes(client: &Client) -> Result<(), Error> {
    let db = client.database(DB_NAME);
    sync_indexes::<ResumeTokenRecordCollConf>(&db).await
}

pub async fn put_resume_token(
    client: &Client,
    change_stream_id: &str,
    resume_token: &Option<ResumeToken>,
) -> Result<(), Error> {
    let record = ResumeTokenRecord {
        id: change_stream_id.into(),
        resume_token: resume_token.clone(),
    };
    let options = FindOneAndReplaceOptions::builder()
        .write_concern(Some(WriteConcern::MAJORITY))
        .upsert(Some(true))
        .build();
    let filter = doc! {field!(id in ResumeTokenRecord): change_stream_id.to_string()};
    let repository  = client.database(DB_NAME).repository::<ResumeTokenRecord>();
    match repository.find_one_and_replace(filter, &record, Some(options)).await
    {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

pub async fn get_resume_token(
    client: &Client,
    change_stream_id: &str,
) -> Result<Option<ResumeToken>, Error> {
    let filter = doc! {field!(id in ResumeTokenRecord): change_stream_id };
    let options = FindOneOptions::builder()
        .read_concern(Some(ReadConcern::MAJORITY))
        .build();
    let repository  = client.database(DB_NAME).repository::<ResumeTokenRecord>();
    match repository
        .find_one(filter, Some(options))
        .await {
        Ok(Some(ResumeTokenRecord {
                    id: _,
                    resume_token,
                })) => Ok(resume_token),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

