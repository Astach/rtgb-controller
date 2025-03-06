use time::OffsetDateTime;
use uuid::Uuid;

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
    StartFermentation {
        target_temp: u8,
    },
    IncreaseTemperature {
        target_temp: u8,
        holding_duration: Option<u8>, // for how long should we hold the targeted temperature once reached
    },
    DecreaseTemperature {
        target_temp: u8,
        holding_duration: Option<u8>, // for how long should we hold the targeted temperature once reached
    },
    StopFermentation {
        target_temp: u8,
    },
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
            CommandType::StopFermentation { .. } => None,
        }
    }
    pub fn target_temp(&self) -> u8 {
        match self {
            CommandType::StartFermentation { target_temp } => *target_temp,
            CommandType::IncreaseTemperature { target_temp, .. } => *target_temp,
            CommandType::DecreaseTemperature { target_temp, .. } => *target_temp,
            CommandType::StopFermentation { target_temp } => *target_temp,
        }
    }
}

/// Status evolves like so Planned -> Sent -> Acknowledged
pub enum CommandStatus {
    Acknowledged(OffsetDateTime), // when was it acknowledged ( used to know when to trigger the
    // next one)
    Planned(Option<OffsetDateTime>), // we don't know when is the planned date when we create the
    // command as it depends on the finish date of the previous command if any
    Sent(OffsetDateTime),
}

pub struct SessionData {
    pub id: Uuid,
    pub fermentation_step_idx: u8,
}
