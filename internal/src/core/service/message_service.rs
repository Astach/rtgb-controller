use crate::core::domain::command::{CommandStatus, NewCommand, SessionData};
use crate::core::domain::message::{
    FermentationStep, HardwareType, Message, MessageType, ScheduleMessageData,
};
use crate::core::port::messaging::{MessageDrivenPort, MessageDriverPort};
use anyhow::anyhow;
use log::warn;
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
                return Err(anyhow::anyhow!( "Rate for {:?} is misconfigured, the final temperature after its execution would not match the whished targeted temperature", step));
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
    fn build_commands(&self, data: &ScheduleMessageData) -> anyhow::Result<Vec<NewCommand>> {
        Ok(data
            .steps
            .iter()
            .flat_map(|step| {
                step.rate.as_ref().map_or(
                    vec![self.build_command(
                        data.session_id,
                        step.position,
                        step.target_temperature,
                        step.duration,
                    )],
                    |rate| {
                        let prev_step = data
                            .steps
                            .iter()
                            .find(|s| s.position == step.position - 1)
                            .unwrap();
                        // FIXME if step.position is 0 it will panic as u8 cannot be -1, this should never happen here but handle it.
                        // handle unwrap;
                        let number_of_commands = self.calculate_rate_commands_number(
                            prev_step.target_temperature,
                            step.target_temperature,
                            f32::from(rate.value),
                        );
                        (0..number_of_commands)
                            .map(|r| {
                                let value =
                                    if prev_step.target_temperature > step.target_temperature {
                                        prev_step.target_temperature
                                            - (r + 1) as f32 * rate.value as f32
                                    } else {
                                        prev_step.target_temperature
                                            + (r + 1) as f32 * rate.value as f32
                                    };
                                self.build_command(
                                    data.session_id,
                                    step.position,
                                    value,
                                    rate.duration,
                                )
                            })
                            .collect()
                    },
                )
            })
            .collect())
    }
}

impl<R: MessageDrivenPort> MessageService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
