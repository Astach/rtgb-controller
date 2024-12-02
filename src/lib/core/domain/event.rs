use uuid::{Timestamp, Uuid};

pub struct EventMessage {
    id: Uuid,
    sent_at: Timestamp,
    version: u32,
    event_type: EventType,
}

enum EventType {
    Schedule(ScheduleEventData),
}
struct ScheduleEventData {
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
