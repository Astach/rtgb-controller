use core::str;

use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Default)]
pub struct NewCommand {
    pub id: Uuid,
    pub sent_at: Option<OffsetDateTime>,
    pub version: u8,
    pub session_data: SessionData,
    pub status: CommandStatus,
    pub value: f32,
    pub value_holding_duration: u8,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Command {
    pub uuid: Uuid,
    pub fermentation_step_id: i32,
    pub status: CommandStatus,
    pub session_id: i32,
    pub temparature_data: CommandTemperatureData,
}
impl Command {
    pub fn status(mut self, command_status: CommandStatus) -> Self {
        self.status = command_status;
        self
    }
    pub fn value_reached_at(mut self, reached_at: OffsetDateTime) -> Self {
        self.temparature_data.value_reached_at = Some(reached_at);
        self
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct CommandTemperatureData {
    pub value: f32,
    pub value_reached_at: Option<OffsetDateTime>,
    pub value_holding_duration: u8,
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
#[derive(Default)]
pub struct SessionData {
    pub id: Uuid,
    pub step_position: u8,
}
