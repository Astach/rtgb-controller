use uuid::{Timestamp, Uuid};

pub struct Message {
    id: Uuid,
    sent_at: Timestamp,
    version: u32,
    message_type: MesssageType,
}

enum MesssageType {
    Schedule(ScheduleMessageData),
}
struct ScheduleMessageData {
    session_id: Uuid,
    steps: Vec<FermentationStep>,
}
struct FermentationStep {
    target_temperature: u16,
    duration: u8,
    rate: Option<Rate>,
}
struct Rate {
    value: u8,
    frequency: u8,
}
