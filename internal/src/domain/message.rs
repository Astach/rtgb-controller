use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug)]
pub struct Message {
    pub id: Uuid,
    pub sent_at: OffsetDateTime,
    pub version: u32,
    pub message_type: MessageType,
}

#[derive(Debug)]
pub enum MessageType {
    Schedule(ScheduleMessageData),
    Tracking(TrackingMessageData),
}

#[derive(Debug, Default)]
pub struct TrackingMessageData {
    pub session_id: Uuid,
    pub temperature: f32,
}

#[derive(Debug)]
pub struct ScheduleMessageData {
    pub session_id: Uuid,
    pub hardwares: Vec<Hardware>,
    pub steps: Vec<FermentationStep>,
}
impl ScheduleMessageData {
    pub fn get_hardware_of_type(&self, hardware_type: &HardwareType) -> Option<&Hardware> {
        self.hardwares.iter().find(|h| &h.hardware_type == hardware_type)
    }
}

#[derive(Debug, PartialEq)]
pub struct FermentationStep {
    pub position: usize,
    pub target_temperature: f32,
    pub duration: Duration,
    pub rate: Option<Rate>,
}

#[derive(Debug, PartialEq)]
pub struct Rate {
    pub value: u8,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareType {
    Cooling,
    Heating,
}
impl HardwareType {
    pub fn name(&self) -> &'static str {
        match self {
            HardwareType::Cooling => "Cooling",
            HardwareType::Heating => "Heating",
        }
    }
}
#[derive(Debug, Clone)]
pub struct Hardware {
    pub hardware_type: HardwareType,
    pub id: String,
}

impl Hardware {
    pub fn new(id: String, hardware_type: HardwareType) -> Self {
        Hardware { id, hardware_type }
    }
}
