use core::str;
use time::Duration;

use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct NewCommand {
    pub id: Uuid,
    pub sent_at: Option<OffsetDateTime>,
    pub version: u8,
    pub session_data: SessionData,
    pub status: CommandStatus,
    pub value: f32,
    pub value_holding_duration: Duration,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Command {
    pub uuid: Uuid,
    pub fermentation_step_id: i32,
    pub status: CommandStatus,
    pub session_id: i32,
    pub temperature_data: CommandTemperatureData,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct CommandTemperatureData {
    pub value: f32,
    pub value_reached_at: Option<OffsetDateTime>,
    pub value_holding_duration: Duration,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub enum CommandStatus {
    #[default]
    Planned,
    Running {
        since: OffsetDateTime,
    },
    // next one
    Executed {
        at: OffsetDateTime,
    }, // when the target_temp is reached and optional duration passed
}

impl CommandStatus {
    pub fn name(&self) -> &str {
        match self {
            CommandStatus::Planned => "Planned",
            CommandStatus::Running { .. } => "Running",
            CommandStatus::Executed { .. } => "Executed",
        }
    }
    pub fn date(&self) -> Option<OffsetDateTime> {
        match self {
            CommandStatus::Planned => None,
            CommandStatus::Running { since } => Some(*since),
            CommandStatus::Executed { at } => Some(*at),
        }
    }
}
#[derive(Default, Debug)]
pub struct SessionData {
    pub id: Uuid,
    pub step_position: u8,
}
