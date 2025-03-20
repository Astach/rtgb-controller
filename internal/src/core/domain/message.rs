use time::OffsetDateTime;
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
}

#[derive(Debug)]
pub struct ScheduleMessageData {
    pub session_id: Uuid,
    pub hardwares: Vec<Hardware>,
    pub steps: Vec<FermentationStep>,
}
impl ScheduleMessageData {
    pub fn get_hardware_of_type(&self, hardware_type: &HardwareType) -> Option<&Hardware> {
        self.hardwares
            .iter()
            .find(|h| &h.hardware_type == hardware_type)
    }
}

#[derive(Debug)]
pub struct FermentationStep {
    pub target_temperature: f32,
    pub duration: u8,
    pub rate: Option<Rate>,
}

#[derive(Debug)]
pub struct Rate {
    pub value: u8,
    pub frequency: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareType {
    Cooling,
    Heating,
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
