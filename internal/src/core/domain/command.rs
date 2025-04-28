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

pub enum CommandStatus {
    Planned,
    Running(OffsetDateTime),
    // next one
    Executed(OffsetDateTime), // when the target_temp is reached and optional duration passed
}

impl Default for CommandStatus {
    fn default() -> Self {
        CommandStatus::Planned
    }
}

impl CommandStatus {
    pub fn name(&self) -> &str {
        match self {
            CommandStatus::Planned => "Planned",
            CommandStatus::Running(..) => "Running",
            CommandStatus::Executed(..) => "Executed",
        }
    }
    pub fn date(&self) -> Option<OffsetDateTime> {
        match self {
            CommandStatus::Planned => None,
            CommandStatus::Running(offset_date_time) => Some(*offset_date_time),
            CommandStatus::Executed(offset_date_time) => Some(*offset_date_time),
        }
    }
}
#[derive(Default)]
pub struct SessionData {
    pub id: Uuid,
    pub step_position: u8,
}
