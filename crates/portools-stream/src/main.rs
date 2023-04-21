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

    let mut change_stream = match init_change_stream(&client).await {
        Ok(stream) => stream,
        Err(_error) => panic!("failed to initialize change stream stream: {_error}"),
    };

    let service = AllocationService {
        dao: Box::new(MongoDao::new(client)),
    };

    //let mut resume_token = None;
    while change_stream.is_alive() {
        match change_stream.next_if_any().await {
            Ok(Some(event)) => {
                // process event
                match event.operation_type {
                    OperationType::Insert | OperationType::Replace => {
                        if let Some(ref portfolio) = event.full_document {
                            match service.update_allocation(portfolio).await {
                                Err(_error) => {
                                    tracing::error!("failed to update portfolio allocation")
                                }
                                Ok(_) => {
                                    tracing::info!("updated portfolio allocation")
                                }
                            }
                        }
                    }
                    _ => {
                        tracing::warn!(
                            "unsupported operation performed: {:?}, document: {:?}",
                            event.operation_type,
                            event.full_document
                        );
                    }
                }
            }
            Ok(None) => {
                tracing::trace!("got none");
            }
            Err(error) => {
                tracing::error!("got error from stream: {error}")
            }
        }
        //resume_token = change_stream.resume_token();
    }
    tracing::info!("change stream is no longer alive");
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
