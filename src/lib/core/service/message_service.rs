use crate::core::domain::event::EventMessage;
use crate::core::port::messaging::{MessageDrivenPort, MessageDriverPort};

pub struct MessageService<R: MessageDrivenPort> {
    repository: R,
}

impl<R: MessageDrivenPort> MessageDriverPort for MessageService<R> {
    fn process(&self, event: EventMessage) {
        //TODO use correct method
        self.repository.fetch();
        todo!()
    }
}
impl<R: MessageDrivenPort> MessageService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
