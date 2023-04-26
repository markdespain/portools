
pub type Id = u32;

pub trait Record {
    fn id(&self) -> Id;
}