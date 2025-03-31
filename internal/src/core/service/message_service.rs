use crate::core::domain::command::{Command, CommandStatus, CommandType, SessionData};
use crate::core::domain::message::{
    FermentationStep, HardwareType, Message, MessageType, ScheduleMessageData,
};
use crate::core::port::messaging::{MessageDrivenPort, MessageDriverPort};
use anyhow::anyhow;
use log::{error, warn};
use uuid::Uuid;

pub struct MessageService<R: MessageDrivenPort> {
    repository: R,
}

impl<R: MessageDrivenPort + Sync> MessageDriverPort for MessageService<R> {
    async fn process(&self, message: Message) -> anyhow::Result<u64> {
        match message.message_type {
            MessageType::Schedule(data) => {
                self.validate(&data.steps)?;
                let heating = data
                    .get_hardware_of_type(&HardwareType::Heating)
                    .ok_or(anyhow!("Unable to find heating hardware"))
                    .cloned()?;
                let cooling = data
                    .get_hardware_of_type(&HardwareType::Cooling)
                    .ok_or(anyhow!("Unable to find cooling hardware"))
                    .cloned()?;
                let cmds = self.build_commands(&data)?;
                self.repository.insert(cmds, heating, cooling).await
            }
        }
    }
}
/// StartFermentation if we're at the first fermentation step
/// IncreaseTemperature the temperature if the step target_temp if higher than the one before
/// DecreaseTemperature the temperature if the step target_temp if lower than the one before
/// StopFermentation if we've reached the last fermentation step
impl<R: MessageDrivenPort> MessageService<R> {
    fn validate(&self, steps: &[FermentationStep]) -> anyhow::Result<()> {
        if steps.len() < 2 {
            return Err(anyhow::anyhow!(
                "There must be at least a StartFermentation and StopFermentation step"
            ));
        }
        if steps.first().is_some_and(|step| step.rate.is_some()) {
            return Err(anyhow::anyhow!(
                "rate can't be defined for StartFermentation"
            ));
        }
        //TODO must be tested
        if steps.iter().filter(|x| x.rate.is_some()).all(|step| {
            let step_idx = match steps.iter().position(|x| x == step) {
                Some(idx) => idx,
                None => {
                    error!(
                        "Unable to find step with a rate in the array of steps: {:?}",
                        step
                    );
                    return false;
                }
            };
            let rate_temp_value = step.rate.as_ref().map_or(0, |rate| i32::from(rate.value));
            let number_of_cmds = self.calculate_rate_commands_number(
                steps[step_idx - 1].target_temperature,
                step.target_temperature,
                rate_temp_value as f32,
            );
            let final_temp_delta = rate_temp_value * number_of_cmds;
            let temp_delta_between_steps =
                (steps[step_idx - 1].target_temperature - step.target_temperature).abs() as i32;
            let rate_is_valid = final_temp_delta == temp_delta_between_steps;
            if !rate_is_valid {
                warn!(
                    "Rate configuration {:?} for step with index {:?} with target temp {:?} is not valid considering the previous step's target temperature of {:?}",
                    &step.rate,
                    step_idx,
                    step.target_temperature,
                    steps[step_idx - 1].target_temperature
                );
            }
            rate_is_valid
        }) {
            return Err(anyhow::anyhow!(
                "some rate(s) are misconfigured, there executions would overpass the whished targeted temperature"
            ));
        }
        Ok(())
    }
    fn build_command_type(
        &self,
        idx: usize,
        steps: &[FermentationStep],
        target_temp: f32,
        holding_duration: Option<u8>,
    ) -> anyhow::Result<CommandType> {
        Ok(match idx {
            0 => CommandType::StartFermentation { target_temp },
            x if x == steps.len() - 1 => CommandType::StopFermentation {
                target_temp,
                holding_duration,
            },
            _ => {
                let is_target_temp_higher = target_temp > steps[idx - 1].target_temperature;
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

    fn build_command(&self, session_id: Uuid, idx: usize, command_type: CommandType) -> Command {
        Command {
            id: Uuid::new_v4(),
            sent_at: None,
            version: 1,
            session_data: SessionData {
                id: session_id,
                fermentation_step_idx: idx as u8,
            },
            status: CommandStatus::Planned,
            command_type,
        }
    }

    fn calculate_rate_commands_number(
        &self,
        previous_target_temp: f32,
        next_target_temp: f32,
        rate: f32,
    ) -> i32 {
        let delta = (previous_target_temp - next_target_temp).abs();
        (delta / rate) as i32
    }

    // TODO test it
    fn build_commands(&self, data: &ScheduleMessageData) -> anyhow::Result<Vec<Command>> {
        data.steps
            .iter()
            .flat_map(|step| {
                step.rate.as_ref().map_or(
                    {
                        vec![
                            self.build_command_type(
                                step.position,
                                &data.steps,
                                step.target_temperature,
                                None,
                            )
                            .map(|c_type| {
                                self.build_command(data.session_id, step.position, c_type)
                            }),
                        ]
                    },
                    |rate| {
                        // FIXME can't have a rate on the first step (StartFermentation) as we don't know the current
                        // temperature
                        let number_of_commands = self.calculate_rate_commands_number(
                            data.steps[step.position - 1].target_temperature,
                            step.target_temperature,
                            f32::from(rate.value),
                        );
                        (0..number_of_commands)
                            .map(|_| {
                                self.build_command_type(
                                    step.position,
                                    &data.steps,
                                    f32::from(rate.value),
                                    Some(rate.duration),
                                )
                                .map(|c_type| {
                                    self.build_command(data.session_id, step.position, c_type)
                                })
                            })
                            .collect()
                    },
                )
            })
            .collect()
    }
}

impl<R: MessageDrivenPort> MessageService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
