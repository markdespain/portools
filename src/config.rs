use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Limits {
    pub max_file_size: usize,
    pub max_num_lots: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_file_size: 10_000,
            max_num_lots: 10_000,
        }
    }
}