use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde_json;
use time::OffsetDateTime;
use uuid::Uuid;

use internal::domain::{
    self,
    message::{FermentationStep, Hardware, HardwareType, Rate, ScheduleMessageData},
};

#[derive(Deserialize, Debug)]
pub struct Event {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub sent_at: OffsetDateTime,
    pub version: u32,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: EventData,
}
#[derive(Deserialize, Debug)]
pub struct EventData {
    session_id: Uuid,
    hardwares: Vec<HardwareData>,
    steps: Vec<FermentationStepData>,
}
#[derive(Deserialize, Debug)]
pub struct FermentationStepData {
    pub position: usize,
    pub target_temperature: f32,
    pub duration: u8,
    pub rate: Option<RateData>,
}
#[derive(Deserialize, Debug)]
pub struct RateData {
    value: u8,
    duration: u8,
}

#[derive(Deserialize, Debug)]
pub struct HardwareData {
    hardware_type: String,
    id: String,
}

impl TryFrom<&async_nats::jetstream::Message> for Event {
    type Error = anyhow::Error;

    fn try_from(value: &async_nats::jetstream::Message) -> Result<Self, Self::Error> {
        let utf8_str = std::str::from_utf8(value.payload.as_ref())
            .map_err(|e| anyhow::anyhow!("UTF-8 conversion error: {}", e))?;

        serde_json::from_str(utf8_str)
            .map_err(|e| anyhow::anyhow!("JSON deserialization error: {}, {}", e, utf8_str))
    }
}
impl From<&FermentationStepData> for FermentationStep {
    fn from(value: &FermentationStepData) -> Self {
        FermentationStep {
            position: value.position,
            target_temperature: value.target_temperature,
            duration: value.duration,
            rate: value.rate.as_ref().map(|r| Rate {
                value: r.value,
                duration: r.duration,
            }),
        }
    }
}
impl TryFrom<&HardwareData> for Hardware {
    type Error = anyhow::Error;

    fn try_from(value: &HardwareData) -> anyhow::Result<Self, Self::Error> {
        match value.hardware_type.to_lowercase().as_str() {
            "heating" => Ok(Hardware {
                id: value.id.to_string(),
                hardware_type: HardwareType::Heating,
            }),
            "cooling" => Ok(Hardware {
                id: value.id.to_string(),
                hardware_type: HardwareType::Cooling,
            }),
            _ => Err(anyhow!("Unknown hardware type: {}", value.hardware_type)),
        }
    }
}

impl Event {
    pub fn to_domain(&self) -> Result<domain::message::Message> {
        Self::types(self.event_type.as_str(), &self.data).map(|msg_type| domain::message::Message {
            id: self.id,
            sent_at: self.sent_at,
            version: self.version,
            message_type: msg_type,
        })
    }
    fn types(raw_type: &str, data: &EventData) -> Result<domain::message::MessageType> {
        Self::hardwares(data).map(|hws| match raw_type.to_lowercase().as_str() {
            "schedule" => Ok(domain::message::MessageType::Schedule(
                ScheduleMessageData {
                    session_id: data.session_id,
                    hardwares: hws,
                    steps: Self::steps(&data.steps),
                },
            )),
            _ => Err(anyhow!("Unknown message type: {}", raw_type)),
        })?
    }
    fn hardwares(data: &EventData) -> Result<Vec<Hardware>> {
        data.hardwares.iter().map(Hardware::try_from).collect()
    }
    fn steps(steps: &[FermentationStepData]) -> Vec<FermentationStep> {
        steps.into_iter().map(FermentationStep::from).collect()
    }
}
#[cfg(test)]
mod tests {

    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::inbound::model::event::{FermentationStepData, HardwareData, RateData};

    use super::{Event, EventData};

    #[test]
    fn should_map_event_to_message() {
        let event_data = EventData {
            session_id: Uuid::new_v4(),
            hardwares: vec![HardwareData {
                id: "anId".to_string(),
                hardware_type: "cooling".to_string(),
            }],
            steps: vec![FermentationStepData {
                position: 0,
                target_temperature: 21.0,
                duration: 1,
                rate: None,
            }],
        };
        let event = Event {
            id: Uuid::new_v4(),
            sent_at: OffsetDateTime::now_utc(),
            version: 1,
            event_type: "schedule".to_string(),
            data: event_data,
        };
        let msg = event.to_domain().unwrap();
        assert_eq!(msg.sent_at, event.sent_at);
        assert_eq!(msg.version, event.version);
        assert_eq!(msg.id, event.id);
    }

    #[test]
    fn should_map_event_type_to_message_type() {
        let event_data = EventData {
            session_id: Uuid::new_v4(),
            hardwares: vec![HardwareData {
                id: "anId".to_string(),
                hardware_type: "cooling".to_string(),
            }],
            steps: vec![FermentationStepData {
                position: 0,
                target_temperature: 21.0,
                duration: 1,
                rate: None,
            }],
        };
        Event::types("schedule", &event_data).unwrap();
        Event::types("SchEdule", &event_data).unwrap();
    }

    #[test]
    #[should_panic]
    fn should_be_err_on_invalid_event_type() {
        let event_data = EventData {
            session_id: Uuid::new_v4(),
            hardwares: vec![HardwareData {
                id: "anId".to_string(),
                hardware_type: "cooling".to_string(),
            }],
            steps: vec![FermentationStepData {
                position: 0,
                target_temperature: 21.0,
                duration: 1,
                rate: None,
            }],
        };
        Event::types("takeovertheworld", &event_data).unwrap();
    }
    #[test]
    #[should_panic]
    fn should_be_err_on_invalid_hardware_type() {
        let event_data = EventData {
            session_id: Uuid::new_v4(),
            hardwares: vec![HardwareData {
                id: "anId".to_string(),
                hardware_type: "chilling".to_string(),
            }],
            steps: vec![FermentationStepData {
                position: 0,
                target_temperature: 21.0,
                duration: 1,
                rate: None,
            }],
        };
        Event::types("schedule", &event_data).unwrap();
    }
    #[test]
    fn should_map_event_step_to_fermentation_step() {
        let step_data = vec![
            FermentationStepData {
                position: 0,
                target_temperature: 21.0,
                duration: 1,
                rate: None,
            },
            FermentationStepData {
                position: 1,
                target_temperature: 22.0,
                duration: 2,
                rate: Some(RateData {
                    value: 1,
                    duration: 1,
                }),
            },
        ];
        Event::steps(&step_data).iter().for_each(|step| {
            assert_eq!(step.duration, step_data[step.position].duration);
            assert_eq!(
                step.target_temperature,
                step_data[step.position].target_temperature
            );
            match (&step.rate, &step_data[step.position].rate) {
                (None, None) => {} // Pass
                (Some(r), Some(rd)) => {
                    assert_eq!(r.value, rd.value);
                    assert_eq!(r.duration, rd.duration);
                }
                _ => panic!("Mismatched Rate options value"),
            }
        });
    }
}
