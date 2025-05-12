use uuid::Uuid;

use crate::domain::{
    command::NewCommand,
    error::MessageServiceError,
    message::{Hardware, Message},
};

pub trait MessageDriverPort {
    fn process(
        &self,
        message: Message,
    ) -> impl Future<Output = Result<u64, MessageServiceError>> + Send;
}

pub trait MessageDrivenPort {
    fn fetch(&self, command_id: Uuid) -> impl Future<Output = Option<NewCommand>> + Send; // FIXME not a new command
    //here

    fn insert(
        &self,
        commands: Vec<NewCommand>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;
    fn update(&self, command_id: Uuid) -> anyhow::Result<NewCommand>; // FIXME not a new command
    //here
    fn delete(&self, command_id: Uuid) -> anyhow::Result<NewCommand>; // FIXME not a new command
    //here
}
