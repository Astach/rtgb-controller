use anyhow::{Result, bail};
use internal::domain::message::{
    FermentationStep, Hardware, HardwareType, Message, MessageType, Rate, ScheduleMessageData, TrackingMessageData,
};
use serde::Deserialize;
use serde_json;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub sent_at: OffsetDateTime,
    pub version: u32,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub data: EventData,
}
#[derive(Deserialize, Debug, Clone)]
pub enum EventData {
    Schedule {
        session_id: Uuid,
        hardwares: Vec<HardwareData>,
        steps: Vec<FermentationStepData>,
    },
    Tracking {
        session_id: Uuid,
        temperature: f32,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct FermentationStepData {
    pub position: usize,
    pub target_temperature: f32,
    pub duration: i64,
    pub rate: Option<RateData>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct RateData {
    value: u8,
    duration: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HardwareData {
    hardware_type: String,
    id: String,
}

impl TryFrom<&async_nats::jetstream::Message> for Event {
    type Error = anyhow::Error;

    fn try_from(value: &async_nats::jetstream::Message) -> Result<Self, Self::Error> {
        let utf8_str = std::str::from_utf8(value.payload.as_ref())
            .map_err(|e| anyhow::anyhow!("UTF-8 conversion error: {}", e))?;

        serde_json::from_str(utf8_str).map_err(|e| anyhow::anyhow!("JSON deserialization error: {}, {}", e, utf8_str))
    }
}
impl From<&FermentationStepData> for FermentationStep {
    fn from(value: &FermentationStepData) -> Self {
        FermentationStep {
            position: value.position,
            target_temperature: value.target_temperature,
            duration: Duration::hours(value.duration),
            rate: value.rate.as_ref().map(|r| Rate {
                value: r.value,
                duration: Duration::hours(r.duration),
            }),
        }
    }
}
impl TryFrom<HardwareData> for Hardware {
    type Error = anyhow::Error;

    fn try_from(value: HardwareData) -> anyhow::Result<Self, Self::Error> {
        match value.hardware_type.to_lowercase().as_str() {
            "heating" => Ok(Hardware {
                id: value.id.to_string(),
                hardware_type: HardwareType::Heating,
            }),
            "cooling" => Ok(Hardware {
                id: value.id.to_string(),
                hardware_type: HardwareType::Cooling,
            }),
            _ => bail!("Unknown hardware type: {}", value.hardware_type),
        }
    }
}
impl TryFrom<Event> for Message {
    type Error = anyhow::Error;

    fn try_from(value: Event) -> std::result::Result<Self, Self::Error> {
        Ok(match value.event_type.as_str() {
            "schedule" => Message {
                id: value.id,
                sent_at: value.sent_at,
                version: value.version,
                message_type: MessageType::Schedule(ScheduleMessageData::try_from(value.data)?),
            },
            "tracking" => Message {
                id: value.id,
                sent_at: value.sent_at,
                version: value.version,
                message_type: MessageType::Tracking(TrackingMessageData::try_from(value.data)?),
            },
            _ => bail!("Event type {} is not supported", value.event_type),
        })
    }
}

impl TryFrom<EventData> for TrackingMessageData {
    type Error = anyhow::Error;

    fn try_from(value: EventData) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            EventData::Schedule { .. } => {
                bail!("Cannot convert schedule event data to tracking message data")
            }
            EventData::Tracking {
                session_id,
                temperature,
            } => TrackingMessageData {
                session_id,
                temperature,
            },
        })
    }
}
impl TryFrom<EventData> for ScheduleMessageData {
    type Error = anyhow::Error;

    fn try_from(value: EventData) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            EventData::Schedule {
                session_id,
                hardwares,
                steps,
            } => ScheduleMessageData {
                session_id,
                hardwares: hardwares
                    .into_iter()
                    .map(Hardware::try_from)
                    .collect::<Result<Vec<Hardware>, _>>()?,
                steps: steps.iter().map(FermentationStep::from).collect(),
            },
            EventData::Tracking { .. } => {
                bail!("Cannot convert tracking event data to schedule message data")
            }
        })
    }
}
#[cfg(test)]
mod tests {

    use internal::domain::message::{FermentationStep, Hardware, HardwareType, Message, MessageType};
    use time::{Duration, OffsetDateTime};
    use uuid::Uuid;

    use crate::inbound::model::event::{FermentationStepData, HardwareData, RateData};

    use super::{Event, EventData};

    #[test]
    fn should_map_schedule_event_to_message() {
        let event_data = EventData::Schedule {
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
        let msg = Message::try_from(event.clone()).unwrap();
        assert_eq!(msg.sent_at, event.sent_at);
        assert_eq!(msg.version, event.version);
        assert_eq!(msg.id, event.id);
        match msg.message_type {
            MessageType::Schedule(schedule_message_data) => {
                assert_eq!(schedule_message_data.hardwares.len(), 1);
                let hw = schedule_message_data.hardwares.first().unwrap();
                assert_eq!(hw.hardware_type, HardwareType::Cooling);
                assert_eq!(hw.id, "anId");
                assert_eq!(schedule_message_data.steps.len(), 1);
                let step = schedule_message_data.steps.first().unwrap();
                assert_eq!(step.rate, None);
                assert_eq!(step.duration, Duration::hours(1));
                assert_eq!(step.target_temperature, 21.0);
                assert_eq!(step.position, 0);
            }
            MessageType::Tracking(_) => panic!("should be an schedule message"),
        }
    }

    #[test]
    #[should_panic]
    fn should_be_err_on_invalid_event_type() {
        let event_data = EventData::Schedule {
            session_id: Uuid::new_v4(),
            hardwares: vec![],
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
            event_type: "skedule".to_string(),
            data: event_data,
        };
        Message::try_from(event).unwrap();
    }

    #[test]
    #[should_panic]
    fn should_be_err_on_invalid_hardware_type() {
        let hardware_data = HardwareData {
            id: "anId".to_string(),
            hardware_type: "chilling".to_string(),
        };
        Hardware::try_from(hardware_data).unwrap();
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
                rate: Some(RateData { value: 1, duration: 1 }),
            },
        ];

        step_data.iter().flat_map(FermentationStep::try_from).for_each(|step| {
            assert_eq!(step.duration, Duration::hours(step_data[step.position].duration as i64));
            assert_eq!(step.target_temperature, step_data[step.position].target_temperature);
            match (&step.rate, &step_data[step.position].rate) {
                (None, None) => {} // Pass
                (Some(r), Some(rd)) => {
                    assert_eq!(r.value, rd.value);
                    assert_eq!(r.duration, Duration::hours(rd.duration));
                }
                _ => panic!("Mismatched Rate options value"),
            }
        });
    }
}
