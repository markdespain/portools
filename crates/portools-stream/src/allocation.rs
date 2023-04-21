use itertools::Itertools;
use mongodb::error::Error;
use portools_common::dao::Dao;
use portools_common::model::{AssetAllocation, AssetClass, AssetClassAmount, Currency, Portfolio};

pub struct AllocationService {
    pub dao: Box<dyn Dao>,
}

impl AllocationService {
    pub async fn update_allocation(&self, portfolio: &Portfolio) -> Result<(), Error> {
        let asset_allocations: Vec<AssetClassAmount> = match portfolio.lots.is_empty() {
            true => Vec::new(),
            false => {
                let currency_symbol = &portfolio.lots[0].cost_basis.symbol;
                portfolio
                    .lots
                    .iter()
                    .group_by(|lot| get_asset_class(&lot.symbol))
                    .into_iter()
                    .map(|(asset_class, group)| {
                        let sum = group.into_iter().map(|lot| lot.cost_basis.amount).sum();
                        AssetClassAmount {
                            asset_class,
                            amount: Currency::new(sum, currency_symbol).unwrap(),
                        }
                    })
                    .collect()
            }
        };
        let allocation = AssetAllocation {
            id: portfolio.id,
            allocations: asset_allocations,
        };
        self.dao.put_asset_allocation(&allocation).await
    }
}

fn get_asset_class(symbol: &str) -> AssetClass {
    // todo: should not be hard-coded
    // todo: a percentage of each symbol could be represented by a different asset class
    let asset_class = match &symbol.trim().to_ascii_uppercase()[..] {
        "VOO" | "VTI" | "SCHB" | "VTV" => AssetClass::UsStocks,
        "VNQ" => AssetClass::UsRealEstate,
        "VEA" | "VEU" | "SCHF" => AssetClass::IntlStocks,
        "VNQI" => AssetClass::IntlRealEstate,
        "AGG" | "BND" | "VTEB" => AssetClass::UsBonds,
        _ => AssetClass::Other,
    };
    asset_class
}
