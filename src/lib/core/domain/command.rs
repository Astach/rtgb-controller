use uuid::{Timestamp, Uuid};

pub struct CommandMessage {
    id: Uuid,
    sent_at: Timestamp,
    version: u16,
    command_type: CommandType,
}

enum CommandType {
    StartFermentation(FermentationCommandData),
    IncreaseTemperature(FermentationCommandData),
    DecreaseTemperature(FermentationCommandData),
    StopFermentation(FermentationCommandData),
}
struct FermentationCommandData {
    value: u16,
    session_id: Uuid,
    target_id: String,
    status: CommandStatus,
}
enum CommandStatus {
    Planned,
    Sent,
    Acknowledged,
}
