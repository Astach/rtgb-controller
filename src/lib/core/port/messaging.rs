use crate::core::domain::message::Message;

pub trait MessageDriverPort {
    fn process(&self, event: Message);
}
pub trait MessageDrivenPort {
    fn fetch(&self);
    fn insert(&self);
    fn update(&self);
    fn delete(&self);
}
