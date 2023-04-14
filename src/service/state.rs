use crate::dao::Dao;

pub struct State {
    pub dao: Box<dyn Dao>,
}

impl State {
    pub fn new(dao: Box<dyn Dao>) -> State {
        State { dao }
    }
}
