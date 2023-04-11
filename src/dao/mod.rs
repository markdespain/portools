use crate::model::Lot;
use async_trait::async_trait;
use mongodb::error::Error;

pub mod local;
pub mod mongo;

#[async_trait]
pub trait Dao {
    async fn put_lots(&self, lots: &[Lot]) -> Result<(), Error>;

    async fn get_lots(&self) -> Result<Vec<Lot>, Error>;
}
