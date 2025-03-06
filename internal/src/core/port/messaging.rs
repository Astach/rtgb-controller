use uuid::Uuid;

use crate::core::domain::{
    command::Command,
    message::{Hardware, Message},
};

pub trait MessageDriverPort {
    async fn process(&self, event: Message) -> anyhow::Result<()>;
}

pub trait MessageDrivenPort {
    fn fetch(&self, command_id: Uuid) -> Option<Command>;
    async fn insert(
        &self,
        commands: Vec<Command>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> anyhow::Result<u64>;
    fn update(&self, command_id: Uuid) -> anyhow::Result<Command>;
    fn delete(&self, command_id: Uuid) -> anyhow::Result<Command>;
}
