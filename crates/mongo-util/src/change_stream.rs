use crate::record;
use crate::record::Record;
use mongodb::change_stream::event::ResumeToken;
use mongodb::error::Error;
use mongodb::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ResumeTokenRecord {
    // logical identifier for the change stream within an application
    change_stream_id: String,
    resume_token: Option<ResumeToken>,
}

impl Record<String> for ResumeTokenRecord {
    fn id(&self) -> String {
        self.change_stream_id.clone()
    }
}

pub async fn put_resume_token(
    client: &Client,
    database: &str,
    collection: &str,
    change_stream_id: &str,
    resume_token: Option<ResumeToken>,
) -> Result<(), Error> {
    let record = ResumeTokenRecord {
        change_stream_id: change_stream_id.into(),
        resume_token,
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
            change_stream_id: _,
            resume_token,
        })) => Ok(resume_token),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}
