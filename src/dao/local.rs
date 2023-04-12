use crate::dao::Dao;
use crate::model::Portfolio;
use async_trait::async_trait;
use mongodb::error::Error;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct InMemoryDao {
    portfolios: Mutex<HashMap<u32, Portfolio>>,
}

#[async_trait]
impl Dao for InMemoryDao {
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), Error> {
        let mut l = self.portfolios.lock().unwrap();
        l.insert(portfolio.id, portfolio.clone());
        Ok(())
    }

    async fn get_portfolio(&self, id: u32) -> Result<Option<Portfolio>, Error> {
        let l = self.portfolios.lock().unwrap();
        Ok(l.get(&id).map(|p| p.to_owned()))
    }
}
