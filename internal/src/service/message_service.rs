
use crate::domain::command::{CommandStatus, NewCommand, SessionData};
use crate::domain::error::MessageServiceError;
use crate::domain::message::{
    FermentationStep, HardwareType, Message, MessageType, ScheduleMessageData,
};
use crate::port::messaging::{MessageDrivenPort, MessageDriverPort};
use log::warn;
use uuid::Uuid;
pub struct MessageService<R: MessageDrivenPort> {
    repository: R,
}


impl<R: MessageDrivenPort + Sync> MessageDriverPort for MessageService<R> {
    async fn process(&self, message: Message) -> Result<u64, MessageServiceError> {
        match message.message_type {
            MessageType::Schedule(data) => {
                self.validate(&data.steps)?;
                let heating = data
                    .get_hardware_of_type(&HardwareType::Heating)
                    .ok_or(MessageServiceError::NotFound("heating hardware".into()))
                    .cloned()?;
                let cooling = data
                    .get_hardware_of_type(&HardwareType::Cooling)
                    .ok_or(MessageServiceError::NotFound(
                        "Unable to find cooling hardware".into(),
                    ))
                    .cloned()?;
                let cmds = self.build_commands(&data)?;
                self.repository.insert(cmds, heating, cooling).await.map_err(|err|MessageServiceError::TechnicalError(format!("{:?}", err.root_cause())))
            }
        }
    }
}
/// StartFermentation if we're at the first fermentation step
/// IncreaseTemperature the temperature if the step target_temp if higher than the one before
/// DecreaseTemperature the temperature if the step target_temp if lower than the one before
impl<R: MessageDrivenPort> MessageService<R> {
    fn validate(&self, steps: &[FermentationStep]) -> Result<bool, MessageServiceError> {
        if steps.is_empty() {
            return Err(MessageServiceError::NoFermentationStep);
        }

        if steps
            .iter()
            .find(|s| s.position == 0)
            .is_some_and(|step| step.rate.is_some())
        {
            return Err(MessageServiceError::InvalidStepConfiguration(format!("Rate can't be defined for the first fermentation step")));
        }

        //TODO must be tested
        steps
            .iter()
            .filter(|x| x.rate.is_some())
            .try_fold(true, |_, step| {
                let previous_step = steps.iter().find(|s| s.position == step.position - 1);
                let previous_step = match previous_step {
                    Some(prev_step) => prev_step,
                    None => return Err(MessageServiceError::InvalidPosition(step.position - 1)) 
                };
            let rate_is_valid = self.validate_rate(previous_step, step)?;
            if !rate_is_valid {
                warn!( "Rate configuration for {:?} is not valid considering the previous step's target temperature of {:?}", step, previous_step.target_temperature);
                return Err(MessageServiceError::InvalidRateConfiguration(format!("{:?}",step)));
            }
            Ok(rate_is_valid)
            })
    }
    fn validate_rate(
        &self,
        previous_step: &FermentationStep,
        step: &FermentationStep,
    ) -> Result<bool, MessageServiceError> {
        let rate_temp_value = step.rate.as_ref().map(|rate| i32::from(rate.value)).ok_or(
            MessageServiceError::NotFound(format!("rate for step {:?}", step)),
        )?;
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

    fn build_command(
        &self,
        session_id: Uuid,
        position: usize,
        target_temp: f32,
        duration: u8,
    ) -> NewCommand {
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
    fn build_commands(
        &self,
        data: &ScheduleMessageData,
    ) -> Result<Vec<NewCommand>, MessageServiceError> {
        Ok(data
            .steps
            .iter()
            .flat_map(|step| -> Result<Vec<NewCommand>, MessageServiceError> {
                match step.rate.as_ref() {
                    Some(rate) => {
                        let prev_step = data
                            .steps
                            .iter()
                            // FIXME if step.position is 0 it will panic as u8 cannot be -1, this should never happen as the first step can't have rate but this is handled just in case and must be tested .
                            .filter(|s| s.position > 0) 
                            .find(|s| s.position == step.position - 1)
                            .ok_or(MessageServiceError::InvalidPosition(step.position -1))?;

                        let number_of_commands = self.calculate_rate_commands_number( prev_step.target_temperature, step.target_temperature, f32::from(rate.value),);
                        Ok((0..number_of_commands)
                            .map(|r| {
                                let delta = (r + 1) as f32 * rate.value as f32;
                                let target_temp =
                                    if prev_step.target_temperature > step.target_temperature {
                                        prev_step.target_temperature - delta
                                    } else {
                                        prev_step.target_temperature + delta
                                    };
                                self.build_command(
                                    data.session_id,
                                    step.position,
                                    target_temp,
                                    rate.duration,
                                )
                            })
                            .collect())
                    },
                    None => {

                    Ok(vec![self.build_command(
                        data.session_id,
                        step.position,
                        step.target_temperature,
                        step.duration,
                    )])
}
,
                }
            }).flatten()
            .collect())
    }
}

impl<R: MessageDrivenPort> MessageService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
