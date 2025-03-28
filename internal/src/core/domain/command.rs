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
    // start the fermentation process, move on when target_temp is reached
    StartFermentation {
        target_temp: f32,
    },
    // increase temperature by target_temp
    IncreaseTemperature {
        target_temp: f32,
        holding_duration: Option<u8>, // for how long should we hold the targeted temperature once reached
    },
    // decrease temperature by target_temp
    DecreaseTemperature {
        target_temp: f32,
        holding_duration: Option<u8>, // for how long should we hold the targeted temperature once reached
    },
    // stop the fermentation process once the target_temp is reached
    StopFermentation {
        target_temp: f32,
        holding_duration: Option<u8>, // for how long should we hold the targeted temperature once reached
    },
}
impl Default for CommandType {
    fn default() -> Self {
        CommandType::StartFermentation { target_temp: 20.0 }
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
            CommandType::StopFermentation {
                holding_duration, ..
            } => *holding_duration,
        }
    }
    pub fn target_temp(&self) -> f32 {
        match self {
            CommandType::StartFermentation { target_temp } => *target_temp,
            CommandType::IncreaseTemperature { target_temp, .. } => *target_temp,
            CommandType::DecreaseTemperature { target_temp, .. } => *target_temp,
            CommandType::StopFermentation { target_temp, .. } => *target_temp,
        }
    }
}

/// Status evolves like so Planned -> Sent -> Acknowledged
pub enum CommandStatus {
    Planned, // we don't know when is the planned date when we create the
    // command as it depends on the finish date of the previous command if any
    Sent(OffsetDateTime),
    Acknowledged(OffsetDateTime), // when was it acknowledged ( used to know when to trigger the
    // next one
    Executed, // when the target_temp is reached and optional duration passed
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
        }
    }
    pub fn date(&self) -> Option<OffsetDateTime> {
        match self {
            CommandStatus::Planned => None,
            CommandStatus::Sent(offset_date_time) => Some(*offset_date_time),
            CommandStatus::Acknowledged(offset_date_time) => Some(*offset_date_time),
        }
    }
}
#[derive(Default)]
pub struct SessionData {
    pub id: Uuid,
    pub fermentation_step_idx: u8,
}
