use uuid::Uuid;

use crate::domain::{
    command::{Command, CommandStatus, NewCommand},
    error::{CommandExecutorServiceError, CommandSchedulerServiceError},
    message::{Hardware, ScheduleMessageData, TrackingMessageData},
    sorting::QueryOptions,
};

pub trait CommandSchedulerDriverPort {
    fn schedule(&self, data: ScheduleMessageData) -> impl Future<Output = Result<u64, CommandSchedulerServiceError>>;
}
pub trait CommandExecutorDriverPort {
    fn process(
        &self, tracking_message_data: TrackingMessageData,
    ) -> impl Future<Output = Result<(), CommandExecutorServiceError>>;
    fn execute(&self, command: Command) -> impl Future<Output = Result<Command, CommandExecutorServiceError>>;
}

pub trait CommandDrivenPort {
    fn fetch(
        &self, session_id: Uuid, status: &CommandStatus, options: QueryOptions,
    ) -> impl Future<Output = Result<Vec<Command>, anyhow::Error>> + Send;

    fn insert(
        &self, commands: Vec<NewCommand>, heating_h: Hardware, cooling_h: Hardware,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;

    fn update(&self, command: Command) -> impl Future<Output = anyhow::Result<Command>> + Send;
    fn delete(&self, command_id: Uuid) -> anyhow::Result<u64>;
}
