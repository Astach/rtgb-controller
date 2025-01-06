use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use time::OffsetDateTime;
use uuid::Uuid;

use internal::core::domain::{
    self,
    message::{FermentationStep, Rate, ScheduleMessageData},
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    pub id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub sent_at: OffsetDateTime,
    pub version: u32,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: EventData,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct EventData {
    session_id: String,

    steps: Vec<FermentationStepData>,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct FermentationStepData {
    pub target_temperature: u16,
    pub duration: u8,
    pub rate: Option<RateData>,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct RateData {
    value: u8,
    frequency: u8,
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

impl Event {
    pub fn to_domain(self) -> Result<domain::message::Message> {
        Self::types(self.event_type, self.data).map(|msg_type| domain::message::Message {
            id: self.id,
            sent_at: self.sent_at,
            version: self.version,
            message_type: msg_type,
        })
    }
    fn types(raw_type: String, data: EventData) -> Result<domain::message::MessageType> {
        match raw_type.to_lowercase().as_str() {
            "schedule" => Ok(domain::message::MessageType::Schedule(
                ScheduleMessageData {
                    session_id: Uuid::from_str(data.session_id.as_str())
                        .context("Invalid session ID")?,
                    steps: Self::steps(data.steps),
                },
            )),
            _ => Err(anyhow!("Unknown message type: {}", raw_type)),
        }
    }
    fn steps(steps: Vec<FermentationStepData>) -> Vec<FermentationStep> {
        steps
            .iter()
            .map(|step| FermentationStep {
                target_temperature: step.target_temperature,
                duration: step.duration,
                rate: step.rate.as_ref().map(|r| Rate {
                    value: r.value,
                    frequency: r.frequency,
                }),
            })
            .collect()
    }
}
