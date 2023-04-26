use crate::record::Record;
use crate::{record, schema};
use mongodb::change_stream::event::ResumeToken;
use mongodb::error::Error;
use mongodb::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ResumeTokenRecord {
    // logical identifier for the change stream within an application
    id: String,
    resume_token: Option<ResumeToken>,
}

impl Record<String> for ResumeTokenRecord {
    fn id(&self) -> String {
        self.id.clone()
    }
}

pub async fn put_resume_token(
    client: &Client,
    database: &str,
    collection: &str,
    change_stream_id: &str,
    resume_token: &Option<ResumeToken>,
) -> Result<(), Error> {
    let record = ResumeTokenRecord {
        id: change_stream_id.into(),
        resume_token: resume_token.clone(),
    };
    record::upsert(client, database, collection, &record).await
}

pub async fn get_resume_token(
    client: &Client,
    database: &str,
    collection: &str,
    change_stream_id: &str,
) -> Result<Option<ResumeToken>, Error> {
    match record::find_by_id(client, database, collection, change_stream_id.to_string()).await {
        Ok(Some(ResumeTokenRecord {
            id: _,
            resume_token,
        })) => Ok(resume_token),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn create_collections_and_indexes(
    client: &Client,
    database: &str,
    collection: &str,
) -> Result<(), Error> {
    schema::create_collection_and_index_if_not_exist::<ResumeTokenRecord>(
        client, database, collection,
    )
    .await
}
