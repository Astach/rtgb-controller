use uuid::Uuid;

use crate::core::domain::{
    command::Command,
    message::{Hardware, Message},
};

pub trait MessageDriverPort {
    fn process(&self, message: Message) -> impl Future<Output = anyhow::Result<u64>> + Send;
}

pub trait MessageDrivenPort {
    fn fetch(&self, command_id: Uuid) -> impl Future<Output = Option<Command>> + Send;
    fn insert(
        &self,
        commands: Vec<Command>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;
    fn update(&self, command_id: Uuid) -> anyhow::Result<Command>;
    fn delete(&self, command_id: Uuid) -> anyhow::Result<Command>;
}
