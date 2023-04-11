use crate::dao::Dao;

pub type StateDao = dyn Dao + Send + Sync;

pub struct State {
    pub dao: Box<StateDao>,
}

impl State {
    pub fn new(dao: Box<StateDao>) -> State {
        State { dao }
    }
}
