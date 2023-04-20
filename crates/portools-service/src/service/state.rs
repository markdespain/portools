use crate::config::Limits;
use portools_common::dao::Dao;

pub struct State {
    pub limits: Limits,
    pub dao: Box<dyn Dao>,
}
