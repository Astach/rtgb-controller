use std::future::Future;

use uuid::Uuid;

use crate::core::domain::{
    command::Command,
    message::{Hardware, Message},
};

pub trait MessageDriverPort {
    fn process(&self, event: Message) -> anyhow::Result<()>;
}
pub trait MessageDrivenPort {
    fn fetch(&self, command_id: Uuid) -> Option<Command>;
    fn insert<F>(&self, commands: Vec<Command>, heating_h: Hardware, cooling_h: Hardware) -> F
    where
        F: Future<Output = anyhow::Result<i32>>;
    fn update(&self, command_id: Uuid) -> anyhow::Result<Command>;
    fn delete(&self, command_id: Uuid) -> anyhow::Result<Command>;
}
