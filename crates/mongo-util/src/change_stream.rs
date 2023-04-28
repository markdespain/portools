use crate::record;
use crate::record::Record;
use mongodb::change_stream::event::ResumeToken;
use mongodb::error::Error;
use mongodb::Database;
use mongodm::{field, sync_indexes, CollectionConfig, Indexes, Model};
use serde::{Deserialize, Serialize};

pub const COLL_RESUME_TOKEN: &str = "resume_token";

/// A record for a persisted resume token for a logical Mongo change stream within an application
#[derive(Debug, Serialize, Deserialize)]
struct ResumeTokenRecord {
    /// identifier for logical change stream within an application
    id: String,

    /// the latest persisted resume token for logical change stream with the id
    resume_token: Option<ResumeToken>,
}

impl Model for ResumeTokenRecord {
    type CollConf = ResumeTokenRecordConfig;
}

impl Record for ResumeTokenRecord {
    type IdType = String;

    fn id_field() -> &'static str {
        field!(id in ResumeTokenRecord)
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}

struct ResumeTokenRecordConfig;
impl CollectionConfig for ResumeTokenRecordConfig {
    fn collection_name() -> &'static str {
        COLL_RESUME_TOKEN
    }

    fn indexes() -> Indexes {
        Indexes::new().with(ResumeTokenRecord::id_index())
    }
}

pub async fn put_resume_token(
    database: &Database,
    change_stream_id: &str,
    resume_token: &Option<ResumeToken>,
) -> Result<(), Error> {
    let record = ResumeTokenRecord {
        id: change_stream_id.into(),
        resume_token: resume_token.clone(),
    };
    record::upsert(database, &record).await
}

pub async fn get_resume_token(
    database: &Database,
    change_stream_id: &str,
) -> Result<Option<ResumeToken>, Error> {
    match record::find_by_id(database, change_stream_id.to_string()).await {
        Ok(Some(ResumeTokenRecord {
            id: _,
            resume_token,
        })) => Ok(resume_token),
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn sync_collections_and_indexes(db: &Database) -> Result<(), Error> {
    sync_indexes::<ResumeTokenRecordConfig>(db).await
}
