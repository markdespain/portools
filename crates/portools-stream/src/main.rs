use allocation::PortfolioSummaryManager;
use mongo_util::change_stream;
use mongodb::change_stream::event::{ChangeStreamEvent, OperationType};
use mongodb::change_stream::ChangeStream;
use mongodb::options::{
    ChangeStreamOptions, FullDocumentBeforeChangeType, FullDocumentType, ReadConcern,
};
use mongodb::Client;
use portools_common::dao::mongo::{MongoDao, DB_NAME};
use portools_common::log;
use portools_common::model::Portfolio;
use portools_stream::allocation;

const APP_NAME: &str = "portools-stream";
const COLL_PORTFOLIO: &str = "portfolio";
const CHANGE_STREAM_ID: &str = APP_NAME;

#[tokio::main]
async fn main() {
    log::init(APP_NAME).unwrap_or_else(|e| panic!("failed initialize logging. cause: {:?}", e));

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(&uri)
        .await
        .unwrap_or_else(|error| panic!("failed to connect to {uri}. error: {error}"));

    let mut change_stream = init_change_stream(&client)
        .await
        .unwrap_or_else(|error| panic!("failed to initialize change stream stream: {error}"));

    let service = PortfolioSummaryManager {
        dao: Box::new(MongoDao::new(client.clone())),
    };

    let database = &client.database(DB_NAME);
    while change_stream.is_alive() {
        match change_stream.next_if_any().await {
            Ok(Some(event)) => {
                consume_next_change_event(&event, &service).await;
                let resume_token = change_stream.resume_token();
                change_stream::put_resume_token(
                    database,
                    CHANGE_STREAM_ID,
                    &resume_token,
                )
                .await
                .unwrap_or_else(
                    |error| tracing::error!(%error, ?resume_token, "failed to persist new resume token"),
                )
            }
            Ok(None) => {}
            Err(error) => tracing::error!( %error, "got an error from the change stream"),
        }
    }
    tracing::info!("change stream is no longer alive");
}

async fn consume_next_change_event(
    event: &ChangeStreamEvent<Portfolio>,
    allocation_service: &PortfolioSummaryManager,
) {
    // todo: add tracing span with at least portfolio id
    match event.operation_type {
        OperationType::Insert | OperationType::Replace => {
            if let Some(ref portfolio) = event.full_document {
                match allocation_service
                    .summarize(portfolio)
                    .await
                {
                    Err(error) => {
                        tracing::error!(?error, "failed to update portfolio summary by asset class")
                    }
                    Ok(_) => tracing::info!("updated portfolio allocation"),
                }
            }
        }
        // todo: support delete (and other operation types?)
        _ => {
            tracing::warn!(operation_type = ?event.operation_type, "unsupported operation type")
        }
    }
}

async fn init_change_stream(
    client: &Client,
) -> mongodb::error::Result<ChangeStream<ChangeStreamEvent<Portfolio>>> {
    let database = &client.database(DB_NAME);
    change_stream::sync_collections_and_indexes(database)
        .await
        .unwrap_or_else(|error| {
            panic!("should be able create collections and indexes. error: {error}")
        });

    let resume_token = change_stream::get_resume_token(database, CHANGE_STREAM_ID)
        .await
        .unwrap_or_else(|error| panic!("failed to get initial resume token. error: {error}"));

    let options: ChangeStreamOptions = ChangeStreamOptions::builder()
        .full_document(Some(FullDocumentType::UpdateLookup))
        .full_document_before_change(Some(FullDocumentBeforeChangeType::Off))
        .read_concern(Some(ReadConcern::MAJORITY))
        .resume_after(resume_token)
        .build();

    client
        .database(DB_NAME)
        .collection::<Portfolio>(COLL_PORTFOLIO)
        .watch(None, Some(options))
        .await
}
