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
    pub steps: Vec<FermentationStep>,
}

#[derive(Debug)]
pub struct FermentationStep {
    pub target_temperature: u16,
    pub duration: u8,
    pub rate: Option<Rate>,
}

#[derive(Debug)]
pub struct Rate {
    pub value: u8,
    pub frequency: u8,
}
