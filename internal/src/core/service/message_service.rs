use crate::core::domain::command::{Command, CommandStatus, CommandType, SessionData};
use crate::core::domain::message::{
    FermentationStep, Hardware, HardwareType, Message, MessageType, ScheduleMessageData,
};
use crate::core::port::messaging::{MessageDrivenPort, MessageDriverPort};
use anyhow::anyhow;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct MessageService<R: MessageDrivenPort> {
    repository: R,
}

impl<R: MessageDrivenPort + Sync> MessageDriverPort for MessageService<R> {
    async fn process(&self, message: Message) -> anyhow::Result<()> {
        match message.message_type {
            MessageType::Schedule(data) => {
                let heating = data
                    .get_hardware_of_type(&HardwareType::Heating)
                    .ok_or(anyhow!("Unable to find heating hardware"))
                    .cloned()?;
                let cooling = data
                    .get_hardware_of_type(&HardwareType::Cooling)
                    .ok_or(anyhow!("Unable to find cooling hardware"))
                    .cloned()?;
                let cmds = self.convert_to_commands(&data)?;
                self.save_schedule(cmds, heating, cooling).await.map(|_| ())
            }
        }
    }
}

impl<R: MessageDrivenPort> MessageService<R> {
    pub fn steps_to_command_type(
        &self,
        idx: usize,
        steps: &[FermentationStep],
        target_temp: f32,
        holding_duration: Option<u8>,
    ) -> anyhow::Result<CommandType> {
        Ok(match idx {
            0 => CommandType::StartFermentation { target_temp },
            x if x == steps.len() - 1 => CommandType::StopFermentation { target_temp },
            _ => {
                let is_target_temp_higher =
                    steps[idx].target_temperature > steps[idx - 1].target_temperature;
                if is_target_temp_higher {
                    CommandType::IncreaseTemperature {
                        target_temp,
                        holding_duration,
                    }
                } else {
                    CommandType::DecreaseTemperature {
                        target_temp,
                        holding_duration,
                    }
                }
            }
        })
    }

    pub fn build_command(&self, session_id: Uuid, idx: u8, command_type: CommandType) -> Command {
        Command {
            id: Uuid::new_v4(),
            sent_at: None,
            version: 1,
            session_data: SessionData {
                id: session_id,
                fermentation_step_idx: idx,
            },
            status: if let CommandType::StartFermentation { .. } = &command_type {
                CommandStatus::Planned(Some(OffsetDateTime::now_utc()))
            } else {
                CommandStatus::Planned(None)
            },
            command_type,
        }
    }
    // TODO test it
    fn convert_to_commands(&self, data: &ScheduleMessageData) -> anyhow::Result<Vec<Command>> {
        data.steps
            .iter()
            .enumerate()
            .flat_map(|(idx, step)| {
                step.rate.as_ref().map_or_else(
                    || {
                        vec![
                            self.steps_to_command_type(
                                idx,
                                &data.steps,
                                step.target_temperature,
                                None,
                            )
                            .map(|c_type| self.build_command(data.session_id, idx as u8, c_type)),
                        ]
                    },
                    |rate| {
                        let delta = (data.steps[idx - 1].target_temperature.abs()
                            - step.target_temperature.abs() / f32::from(rate.value))
                            as i32;
                        (0..delta)
                            .map(|_| {
                                self.steps_to_command_type(
                                    idx,
                                    &data.steps,
                                    f32::from(rate.value),
                                    Some(rate.frequency),
                                )
                                .map(|c_type| {
                                    self.build_command(data.session_id, idx as u8, c_type)
                                })
                            })
                            .collect()
                    },
                )
            })
            .collect()
    }
    async fn save_schedule(
        &self,
        commands: Vec<Command>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> anyhow::Result<u64> {
        self.repository.insert(commands, heating_h, cooling_h).await
    }
}

impl<R: MessageDrivenPort> MessageService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
