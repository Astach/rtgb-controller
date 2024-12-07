#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    id: Uuid,
    sent_at: Timestamp,
    version: u32,
    #[serde(rename = "type")]
    event_type: String,
    data: EventData,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct EventData {
    session_id: String,

    steps: Vec<FermentationStep>,
}
#[derive(Deserialize, Serialize, Debug)]
struct FermentationStep {
    target_temperature: u16,
    duration: u8,
    rate: Option<Rate>,
}
#[derive(Deserialize, Serialize, Debug)]
struct Rate {
    value: u8,
    frequency: u8,
}
