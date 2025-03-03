use async_trait::async_trait;
use uuid::Uuid;

use crate::core::domain::{
    command::Command,
    message::{Hardware, Message},
};

#[async_trait]
pub trait MessageDriverPort {
    async fn process(&self, event: Message) -> anyhow::Result<()>;
}

#[async_trait]
pub trait MessageDrivenPort {
    fn fetch(&self, command_id: Uuid) -> Option<Command>;
    async fn insert(
        &self,
        commands: Vec<Command>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> anyhow::Result<i32>;
    fn update(&self, command_id: Uuid) -> anyhow::Result<Command>;
    fn delete(&self, command_id: Uuid) -> anyhow::Result<Command>;
}
