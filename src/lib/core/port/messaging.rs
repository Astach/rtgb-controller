use crate::core::domain::event::EventMessage;

pub trait MessageDriverPort {
    fn process(&self, event: EventMessage);
}
pub trait MessageDrivenPort {
    fn fetch(&self);
    fn insert(&self);
    fn update(&self);
    fn delete(&self);
}
