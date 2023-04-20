use mongodb::error::Error;
use portools_common::dao::Dao;
use portools_common::model::{AssetAllocation, Currency, Portfolio};
use rust_decimal::Decimal;

pub struct AllocationService {
    pub dao: Box<dyn Dao>,
}

impl AllocationService {
    pub async fn update_allocation(&self, portfolio: &Portfolio) -> Result<(), Error> {
        // hard code and allocation for now
        // todo: implement AssetAllocation calculation
        let allocation = AssetAllocation {
            id: portfolio.id,
            intl_bonds: Currency::new(Decimal::from(10), "USD").unwrap(),
            us_bonds: Currency::new(Decimal::from(20), "USD").unwrap(),
            intl_real_estate: Currency::new(Decimal::from(30), "USD").unwrap(),
            us_real_estate: Currency::new(Decimal::from(40), "USD").unwrap(),
            other: Currency::new(Decimal::from(40), "USD").unwrap(),
        };
        self.dao.put_asset_allocation(&allocation).await
    }
}
