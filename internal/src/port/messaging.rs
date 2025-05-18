use uuid::Uuid;

use crate::domain::{
    command::{Command, CommandStatus, NewCommand},
    error::MessageServiceError,
    message::{Hardware, Message},
    sorting::{QueryOptions, Sorting},
};

pub trait MessageDriverPort {
    fn process(&self, message: Message) -> impl Future<Output = Result<u64, MessageServiceError>>;
}

pub trait MessageDrivenPort {
    fn fetch(
        &self,
        session_id: Uuid,
        status: &CommandStatus,
        options: QueryOptions,
    ) -> impl Future<Output = Result<Vec<Command>, anyhow::Error>> + Send;

    fn insert(
        &self,
        commands: Vec<NewCommand>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;

    fn update(
        &self,
        session_uuid: Uuid,
        new_status: CommandStatus,
    ) -> impl Future<Output = anyhow::Result<Command>> + Send;
    fn delete(&self, command_id: Uuid) -> anyhow::Result<u64>;
}
