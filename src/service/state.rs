use crate::config::Limits;
use crate::dao::Dao;

pub struct State {
    pub limits: Limits,
    pub dao: Box<dyn Dao>,
}
