pub struct Limits {
    pub max_file_size: usize,
    pub max_num_lots: usize,
}

pub const APP_LIMITS: Limits = Limits {
    max_file_size: 10_000,
    max_num_lots: 10_000,
};
