use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Limits {
    pub portfolio: PortfolioLimits
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioLimits {
    pub max_file_size: usize,
    pub max_num_lots: usize,
}

impl Default for PortfolioLimits {
    fn default() -> Self {
        PortfolioLimits {
            max_file_size: 10_000,
            max_num_lots: 10_000,
        }
    }
}

