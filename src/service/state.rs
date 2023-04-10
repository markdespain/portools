use crate::dao;
use crate::dao::mongo::MongoDao;
use mongodb::Client;

pub struct State {
    pub dao: Box<dyn dao::Dao + Send + Sync>,
}

impl State {
    pub fn new(client: Client) -> State {
        State {
            dao: Box::new(MongoDao { client }),
        }
    }
}
