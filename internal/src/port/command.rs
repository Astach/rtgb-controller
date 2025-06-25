use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    command::{Command, CommandStatus, NewCommand},
    error::{CommandExecutorServiceError, CommandSchedulerServiceError},
    message::{Hardware, HardwareType, ScheduleMessageData, TrackingMessageData},
    sorting::QueryOptions,
};

pub trait CommandSchedulerDriverPort {
    fn schedule(&self, data: ScheduleMessageData) -> impl Future<Output = Result<u64, CommandSchedulerServiceError>>;
}
pub trait CommandExecutorDriverPort {
    fn process(
        &self, tracking_message_data: TrackingMessageData,
    ) -> impl Future<Output = Result<(), CommandExecutorServiceError>>;
}

#[cfg_attr(test, mockall::automock)]
pub trait CommandDrivenPort {
    fn fetch_hardware_id(
        &self, session_uuid: Uuid, hardware_type: &HardwareType,
    ) -> impl Future<Output = anyhow::Result<String>> + Send;
    fn fetch_active_hardware_type(
        &self, session_uuid: &Uuid,
    ) -> impl Future<Output = anyhow::Result<Option<HardwareType>>> + Send;
    fn update_active_hardware_type(
        &self, session_uuid: Uuid, active_hardware_type: Option<HardwareType>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn fetch_commands(
        &self, session_id: Uuid, status: &CommandStatus, options: QueryOptions,
    ) -> impl Future<Output = Result<Vec<Command>, anyhow::Error>> + Send;

    fn insert(
        &self, commands: Vec<NewCommand>, heating_h: Hardware, cooling_h: Hardware,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;

    fn update_status(&self, uuid: Uuid, status: &CommandStatus)
    -> impl Future<Output = anyhow::Result<Command>> + Send;
    fn update_value_reached_at(
        &self, uuid: Uuid, value_reached_at: OffsetDateTime,
    ) -> impl Future<Output = anyhow::Result<Command>> + Send;
}
