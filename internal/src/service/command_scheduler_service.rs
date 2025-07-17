use std::sync::Arc;

use log::warn;
use time::Duration;
use uuid::Uuid;

use crate::{
    domain::{
        command::{CommandStatus, NewCommand, SessionData},
        error::CommandSchedulerServiceError,
        message::{FermentationStep, HardwareType, ScheduleMessageData},
    },
    port::command::{CommandDrivenPort, CommandSchedulerDriverPort},
};

pub struct CommandSchedulerService<R: CommandDrivenPort> {
    repository: Arc<R>,
}

impl<R: CommandDrivenPort> CommandSchedulerDriverPort for CommandSchedulerService<R> {
    async fn schedule(&self, data: ScheduleMessageData) -> Result<u64, CommandSchedulerServiceError> {
        self.validate(&data.steps)?;
        let heating = data
            .get_hardware_of_type(&HardwareType::Heating)
            .ok_or(CommandSchedulerServiceError::NotFound("heating hardware".into()))
            .cloned()?;
        let cooling = data
            .get_hardware_of_type(&HardwareType::Cooling)
            .ok_or(CommandSchedulerServiceError::NotFound(
                "Unable to find cooling hardware".into(),
            ))
            .cloned()?;
        let cmds = Self::build_commands(&data)?;
        self.repository
            .insert(cmds, heating, cooling)
            .await
            .map_err(|err| CommandSchedulerServiceError::TechnicalError(format!("{:?}", err.root_cause())))
    }
}

impl<R: CommandDrivenPort> CommandSchedulerService<R> {
    fn validate(&self, steps: &[FermentationStep]) -> Result<bool, CommandSchedulerServiceError> {
        if steps.is_empty() {
            return Err(CommandSchedulerServiceError::NoFermentationStep);
        }

        if steps
            .iter()
            .find(|s| s.position == 0)
            .is_some_and(|step| step.rate.is_some())
        {
            return Err(CommandSchedulerServiceError::InvalidStepConfiguration(
                "Rate can't be defined for the first fermentation step".to_string(),
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
                    None => return Err(CommandSchedulerServiceError::InvalidPosition(step.position - 1))
                };
            let rate_is_valid = self.validate_rate(previous_step, step)?;
            if !rate_is_valid {
                warn!( "Rate configuration for {:?} is not valid considering the previous step's target temperature of {:?}", step, previous_step.target_temperature);
                return Err(CommandSchedulerServiceError::InvalidRateConfiguration(format!("{:?}",step)));
            }
            Ok(rate_is_valid)
            })
    }
    fn validate_rate(
        &self, previous_step: &FermentationStep, step: &FermentationStep,
    ) -> Result<bool, CommandSchedulerServiceError> {
        let rate_temp_value =
            step.rate
                .as_ref()
                .map(|rate| i32::from(rate.value))
                .ok_or(CommandSchedulerServiceError::NotFound(format!(
                    "rate for step {:?}",
                    step
                )))?;
        let number_of_cmds = Self::get_needed_commands_for_rate(
            previous_step.target_temperature,
            step.target_temperature,
            rate_temp_value as f32,
        );
        let final_temp_delta = rate_temp_value * number_of_cmds;
        let temp_delta_between_steps = (previous_step.target_temperature - step.target_temperature).abs() as i32;
        Ok(final_temp_delta >= temp_delta_between_steps)
    }

    fn get_needed_commands_for_rate(previous_target_temp: f32, next_target_temp: f32, rate: f32) -> i32 {
        let delta = (previous_target_temp - next_target_temp).abs();
        (delta / rate).ceil() as i32
    }
    fn build_command(session_id: Uuid, position: usize, target_temp: f32, duration: Duration) -> NewCommand {
        NewCommand {
            id: Uuid::new_v4(),
            sent_at: None,
            version: 1,
            session_data: SessionData {
                id: session_id,
                step_position: position as u8,
            },
            status: CommandStatus::Planned,
            value: target_temp,
            value_holding_duration: duration,
        }
    }
    fn build_commands(data: &ScheduleMessageData) -> Result<Vec<NewCommand>, CommandSchedulerServiceError> {
        Ok(data
            .steps
            .iter()
            .flat_map(|step| -> Result<Vec<NewCommand>, CommandSchedulerServiceError> {
                match step.rate.as_ref() {
                    Some(rate) => {
                        let prev_step = data
                            .steps
                            .iter()
                            .filter(|s| s.position > 0)
                            // FIXME if step.position is 0 it will panic as u8 cannot be -1, this should never happen as the first step can't have rate but this must be handled just in case and must be tested .
                            .find(|s| s.position == step.position - 1)
                            .ok_or(CommandSchedulerServiceError::InvalidPosition(step.position - 1))?;

                        let number_of_commands = Self::get_needed_commands_for_rate(
                            prev_step.target_temperature,
                            step.target_temperature,
                            f32::from(rate.value),
                        );
                        Ok((0..number_of_commands)
                            .map(|r| {
                                let delta = (r + 1) as f32 * rate.value as f32;
                                let target_temp = if prev_step.target_temperature > step.target_temperature {
                                    prev_step.target_temperature - delta
                                } else {
                                    prev_step.target_temperature + delta
                                };
                                Self::build_command(data.session_id, step.position, target_temp, rate.duration)
                            })
                            .collect())
                    }
                    None => Ok(vec![Self::build_command(
                        data.session_id,
                        step.position,
                        step.target_temperature,
                        step.duration,
                    )]),
                }
            })
            .flatten()
            .collect())
    }
}
impl<R: CommandDrivenPort> CommandSchedulerService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        CommandSchedulerService { repository }
    }
}
