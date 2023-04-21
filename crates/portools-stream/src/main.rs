use allocation::AllocationService;
use mongodb::change_stream::event::{ChangeStreamEvent, OperationType};
use mongodb::change_stream::ChangeStream;
use mongodb::options::{
    ChangeStreamOptions, FullDocumentBeforeChangeType, FullDocumentType, ReadConcern,
};
use mongodb::Client;
use portools_common::dao::mongo::MongoDao;
use portools_common::log;
use portools_common::model::Portfolio;
use portools_stream::allocation;

const APP_NAME: &str = "portools-stream";
const DB_NAME: &str = "portools";
const COLL_PORTFOLIO: &str = "portfolio";

#[tokio::main]
async fn main() {
    log::init(APP_NAME).unwrap_or_else(|e| panic!("failed initialize logging. cause: {:?}", e));

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(&uri)
        .await
        .unwrap_or_else(|_| panic!("should be able to connect to {}", uri));

    let mut change_stream = init_change_stream(&client)
        .await
        .unwrap_or_else(|e| panic!("failed to initialize change stream stream: {e}"));

    let service = AllocationService {
        dao: Box::new(MongoDao::new(client)),
    };

    //let mut resume_token = None;
    while change_stream.is_alive() {
        consume_next_change_event(&mut change_stream, &service).await;
        //resume_token = change_stream.resume_token();
    }
    tracing::info!("change stream is no longer alive");
}

async fn consume_next_change_event(
    change_stream: &mut ChangeStream<ChangeStreamEvent<Portfolio>>,
    allocation_service: &AllocationService,
) {
    let event = change_stream.next_if_any().await.unwrap_or_else(|error| {
        tracing::error!( %error, "got an error from the change stream");
        None
    });
    if event.is_none() {
        return;
    }
    let event = event.unwrap();
    match event.operation_type {
        OperationType::Insert | OperationType::Replace => {
            if let Some(ref portfolio) = event.full_document {
                match allocation_service.update_allocation(portfolio).await {
                    Err(error) => {
                        tracing::error!(%error, "failed to update portfolio allocation")
                    }
                    Ok(_) => tracing::info!("updated portfolio allocation"),
                }
            }
        }
        _ => {
            tracing::warn!(operation_type = ?event.operation_type, "unsupported operation type")
        }
    }
}

async fn init_change_stream(
    client: &Client,
) -> mongodb::error::Result<ChangeStream<ChangeStreamEvent<Portfolio>>> {
    // todo(): handle resume token saving and provide to change_stream_options on init
    let change_stream_options: ChangeStreamOptions = ChangeStreamOptions::builder()
        .full_document(Some(FullDocumentType::UpdateLookup))
        .full_document_before_change(Some(FullDocumentBeforeChangeType::Off))
        .read_concern(Some(ReadConcern::MAJORITY))
        .build();

    client
        .database(DB_NAME)
        .collection::<Portfolio>(COLL_PORTFOLIO)
        .watch(None, Some(change_stream_options))
        .await
}
