use std::any;

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
    fn validate(&self, steps: &[FermentationStep]) -> anyhow::Result<bool> {
        if steps.is_empty() {
            return Err(anyhow::anyhow!(
                "There must be at least a one fermentation step"
            ));
        }

        if steps
            .iter()
            .find(|s| s.position == 0)
            .is_some_and(|step| step.rate.is_some())
        {
            return Err(anyhow::anyhow!(
                "rate can't be defined for the first fermentation step"
            ));
        }

        //TODO must be tested
        steps
            .iter()
            .filter(|x| x.rate.is_some())
            .try_fold(true, |_, step| {
                let previous_step = steps.iter().find(|s| s.position == step.position - 1);
                let previous_step = match previous_step {
                    Some(prev_step) => prev_step,
                    None => return Err(anyhow::anyhow!( "Unable to find step with position {:?}", step.position - 1)) 
                };
            let rate_is_valid = self.validate_rate(previous_step, step)?;
            if !rate_is_valid {
                warn!( "Rate configuration for {:?} is not valid considering the previous step's target temperature of {:?}", step, previous_step.target_temperature);
                return Err(anyhow::anyhow!( "Rate for {:?} is misconfigured, the final temperature after its execution is would be below the whished targeted temperature", step));
            }
            Ok(rate_is_valid)
            })
    }
    fn validate_rate(
        &self,
        previous_step: &FermentationStep,
        step: &FermentationStep,
    ) -> anyhow::Result<bool> {
        let rate_temp_value = step
            .rate
            .as_ref()
            .map(|rate| i32::from(rate.value))
            .ok_or(anyhow::anyhow!("Unable to find rate for step {:?}", step))?;
        let number_of_cmds = self.calculate_rate_commands_number(
            previous_step.target_temperature,
            step.target_temperature,
            rate_temp_value as f32,
        );
        let final_temp_delta = rate_temp_value * number_of_cmds;
        let temp_delta_between_steps =
            (previous_step.target_temperature - step.target_temperature).abs() as i32;
        Ok(final_temp_delta >= temp_delta_between_steps)
    }
    fn build_command_type(
        &self,
        previous_step: Option<&FermentationStep>,
        step: &FermentationStep,
        value: f32,
        holding_duration: Option<u8>,
    ) -> anyhow::Result<CommandType> {
        Ok(match step.position {
            0 => CommandType::StartFermentation { value },
            _ => {
                let is_target_temp_higher = step.target_temperature
                    > previous_step
                        .ok_or(anyhow::anyhow!("Step before {:?} must exist", step))?
                        .target_temperature;
                if is_target_temp_higher {
                    CommandType::IncreaseTemperature {
                        value,
                        holding_duration,
                    }
                } else {
                    CommandType::DecreaseTemperature {
                        value,
                        holding_duration,
                    }
                }
            }
        })
    }

    fn build_command(
        &self,
        session_id: Uuid,
        step_position: usize,
        command_type: CommandType,
    ) -> Command {
        Command {
            id: Uuid::new_v4(),
            sent_at: None,
            version: 1,
            session_data: SessionData {
                id: session_id,
                step_position: step_position as u8,
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
        (delta / rate).ceil() as i32
    }

    // TODO test it
    fn build_commands(&self, data: &ScheduleMessageData) -> anyhow::Result<Vec<Command>> {
        let mut commands: anyhow::Result<Vec<Command>> = data
            .steps
            .iter()
            .flat_map(|step| {
                step.rate.as_ref().map_or(
                    {
                        vec![
                            self.build_command_type(
                                data.steps.iter().find(|s| s.position == step.position - 1),
                                step,
                                step.target_temperature,
                                None,
                            )
                            .map(|c_type| {
                                self.build_command(data.session_id, step.position, c_type)
                            }),
                        ]
                    },
                    |rate| {
                        let number_of_commands = self.calculate_rate_commands_number(
                            data.steps[step.position - 1].target_temperature,
                            step.target_temperature,
                            f32::from(rate.value),
                        );
                        (0..number_of_commands)
                            .map(|_| {
                                self.build_command_type(
                                    data.steps.iter().find(|s| s.position == step.position - 1),
                                    step,
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
            .collect();
        let last_pos = data
            .steps
            .iter()
            .max_by_key(|s| s.position)
            .map(|s| s.position)
            .ok_or(anyhow!("Can't find last position"))?;
        if let Ok(cmds) = commands.as_mut() {
            cmds.push(self.build_command(data.session_id, last_pos, CommandType::StopFermentation));
        }
        commands
    }
}

impl<R: MessageDrivenPort> MessageService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
