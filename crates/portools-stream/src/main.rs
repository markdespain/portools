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
const COLL_RESUME_TOKEN: &str = "resume_token";
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

    while change_stream.is_alive() {
        consume_next_change_event(&mut change_stream, &service).await;
        change_stream::put_resume_token(
            &client,
            DB_NAME,
            COLL_RESUME_TOKEN,
            CHANGE_STREAM_ID,
            change_stream.resume_token(),
        )
        .await
        .unwrap_or_else(|error| {
            tracing::error!(
                %error,
                "failed to persist new resume token")
        });
    }
    tracing::info!("change stream is no longer alive");
}

async fn consume_next_change_event(
    change_stream: &mut ChangeStream<ChangeStreamEvent<Portfolio>>,
    allocation_service: &PortfolioSummaryManager,
) {
    let event = change_stream.next_if_any().await.unwrap_or_else(|error| {
        tracing::error!( %error, "got an error from the change stream");
        None
    });
    if event.is_none() {
        return;
    }
    // todo: add tracing span with at least portfolio id
    let event = event.unwrap();
    match event.operation_type {
        OperationType::Insert | OperationType::Replace => {
            if let Some(ref portfolio) = event.full_document {
                match allocation_service
                    .put_summary_by_asset_class(portfolio)
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
    change_stream::create_collections_and_indexes(&client, DB_NAME, COLL_RESUME_TOKEN)
        .await
        .unwrap_or_else(|error| {
            panic!("should be able create collections and indexes. error: {error}")
        });

    let resume_token =
        change_stream::get_resume_token(&client, DB_NAME, COLL_RESUME_TOKEN, CHANGE_STREAM_ID)
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
