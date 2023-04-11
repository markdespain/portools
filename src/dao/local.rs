use crate::dao::Dao;
use crate::model::Lot;
use async_trait::async_trait;
use mongodb::error::Error;
use std::sync::Mutex;

pub struct MutexDao {
    lots: Mutex<Vec<Lot>>,
}

impl MutexDao {
    pub fn new() -> MutexDao {
        MutexDao {
            lots: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Dao for MutexDao {
    async fn put_lots(&self, lots: &[Lot]) -> Result<(), Error> {
        let mut l = self.lots.lock().unwrap();
        *l = lots.to_vec();
        Ok(())
    }

    async fn get_lots(&self) -> Result<Vec<Lot>, Error> {
        let l = self.lots.lock().unwrap();
        Ok(l.to_vec())
    }
}
