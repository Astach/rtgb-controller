use crate::core::domain::message::Message;
use crate::core::port::messaging::{MessageDrivenPort, MessageDriverPort};
use log::debug;
pub struct MessageService<R: MessageDrivenPort> {
    repository: R,
}

impl<R: MessageDrivenPort> MessageDriverPort for MessageService<R> {
    fn process(&self, message: Message) {
        debug!("{:?}", message)
    }
}
impl<R: MessageDrivenPort> MessageService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
