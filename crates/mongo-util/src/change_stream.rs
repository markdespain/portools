use crate::mongodm::TypedCollectionConfig;
use crate::record;
use crate::record::Record;
use mongodb::change_stream::event::ResumeToken;
use mongodb::error::Error;
use mongodb::{Client, Database};
use mongodm::{field, sync_indexes, CollectionConfig, Index, IndexOption, Indexes};
use serde::{Deserialize, Serialize};

pub const COLL_RESUME_TOKEN: &str = "resume_token";

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

struct ResumeTokenRecordConfig;
impl CollectionConfig for ResumeTokenRecordConfig {
    fn collection_name() -> &'static str {
        COLL_RESUME_TOKEN
    }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new(field!(id in ResumeTokenRecord)).with_option(IndexOption::Unique))
        // field! macro can be used as well
    }
}
impl TypedCollectionConfig<ResumeToken> for ResumeTokenRecordConfig {}

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

pub async fn sync_collections_and_indexes(db: &Database) -> Result<(), Error> {
    sync_indexes::<ResumeTokenRecordConfig>(&db).await
}
