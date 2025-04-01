use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Default)]
pub struct Command {
    pub id: Uuid,
    pub sent_at: Option<OffsetDateTime>,
    pub version: u8,
    pub session_data: SessionData,
    pub command_type: CommandType,
    pub status: CommandStatus,
}

#[derive(Clone)]
pub enum CommandType {
    // start the fermentation process, move on when the temperature is value
    StartFermentation {
        value: f32,
    },
    // increase temperature by value
    IncreaseTemperature {
        value: f32,
        holding_duration: Option<u8>, // for how long should we hold the targeted temperature once reached
    },
    // decrease temperature by value
    DecreaseTemperature {
        value: f32,
        holding_duration: Option<u8>, // for how long should we hold the targeted temperature once reached
    },
    // stop the fermentation process (send a command to both heating and cooling socket to turn off)
    StopFermentation,
}
impl Default for CommandType {
    fn default() -> Self {
        CommandType::StartFermentation { value: 20.0 }
    }
}

impl CommandType {
    pub fn name(&self) -> &str {
        match self {
            CommandType::StartFermentation { .. } => "StartFermentation",
            CommandType::IncreaseTemperature { .. } => "IncreaseTemperature",
            CommandType::DecreaseTemperature { .. } => "DecreaseTemperature",
            CommandType::StopFermentation { .. } => "StopFermentation",
        }
    }
    pub fn holding_duration(&self) -> Option<u8> {
        match self {
            CommandType::StartFermentation { .. } => None,
            CommandType::IncreaseTemperature {
                holding_duration, ..
            } => *holding_duration,
            CommandType::DecreaseTemperature {
                holding_duration, ..
            } => *holding_duration,
            CommandType::StopFermentation => None,
        }
    }
    pub fn value(&self) -> Option<f32> {
        match self {
            CommandType::StartFermentation { value } => Some(*value),
            CommandType::IncreaseTemperature { value, .. } => Some(*value),
            CommandType::DecreaseTemperature { value, .. } => Some(*value),
            CommandType::StopFermentation => None,
        }
    }
}

/// Status evolves like so Planned -> Sent -> Acknowledged -> Executed
pub enum CommandStatus {
    Planned, // we don't know when is the planned date when we create the
    // command as it depends on the finish date of the previous command if any
    Sent(OffsetDateTime),
    Acknowledged(OffsetDateTime), // when was it acknowledged ( used to know when to trigger the
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
            CommandStatus::Sent(..) => "Sent",
            CommandStatus::Acknowledged(..) => "Acknowledged",
            CommandStatus::Executed(..) => "Executed",
        }
    }
    pub fn date(&self) -> Option<OffsetDateTime> {
        match self {
            CommandStatus::Planned => None,
            CommandStatus::Sent(offset_date_time) => Some(*offset_date_time),
            CommandStatus::Acknowledged(offset_date_time) => Some(*offset_date_time),
            CommandStatus::Executed(offset_date_time) => Some(*offset_date_time),
        }
    }
}
#[derive(Default)]
pub struct SessionData {
    pub id: Uuid,
    pub step_position: u8,
}
