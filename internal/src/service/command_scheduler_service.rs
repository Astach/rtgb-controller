use std::sync::Arc;

use anyhow::bail;
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
        let are_positions_valid =
            (0..steps.len()).all(|idx| steps.iter().filter(|step| step.position == idx).count() == 1);
        if are_positions_valid {
            Ok(true)
        } else {
            Err(CommandSchedulerServiceError::InvalidStepConfiguration(
                "Steps's position do not match the number of steps".to_string(),
            ))
        }
    }

    fn calculate_required_amount_of_command(previous_target_temp: f32, next_target_temp: f32, rate: f32) -> i32 {
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
            .map(|step| -> Result<Vec<NewCommand>, CommandSchedulerServiceError> {
                match step.rate.as_ref() {
                    Some(rate) => {
                        if step.position > 0 {
                            let prev_step = data.steps.iter().find(|s| s.position == step.position - 1).ok_or(
                                CommandSchedulerServiceError::InvalidPosition(step.position - 1, "doesn't exist"),
                            )?;

                            let number_of_commands = Self::calculate_required_amount_of_command(
                                prev_step.target_temperature,
                                step.target_temperature,
                                f32::from(rate.value),
                            );
                            Ok((0..number_of_commands)
                                .map(|r| {
                                    let delta = (r + 1) as f32 * rate.value as f32;
                                    let target_temp = if prev_step.target_temperature > step.target_temperature {
                                        let temp = prev_step.target_temperature - delta;
                                        if temp < step.target_temperature {
                                            step.target_temperature
                                        } else {
                                            temp
                                        }
                                    } else {
                                        let temp = prev_step.target_temperature + delta;
                                        if temp > step.target_temperature {
                                            step.target_temperature
                                        } else {
                                            temp
                                        }
                                    };
                                    Self::build_command(data.session_id, step.position, target_temp, rate.duration)
                                })
                                .collect())
                        } else {
                            Err(CommandSchedulerServiceError::InvalidPosition(
                                step.position,
                                "cannot hold a rate",
                            ))?
                        }
                    }
                    None => Ok(vec![Self::build_command(
                        data.session_id,
                        step.position,
                        step.target_temperature,
                        step.duration,
                    )]),
                }
            })
            .collect::<Result<Vec<_>, _>>() // This collects Result<Vec<Vec<NewCommand>>, Error>, so we keep errors (flat_map only yields Ok values)
            .map(|vec_of_vecs| vec_of_vecs.into_iter().flatten().collect()))? // Flatten the Vec<Vec<NewCommand>>
    }
}
impl<R: CommandDrivenPort> CommandSchedulerService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        CommandSchedulerService { repository }
    }
}
#[cfg(test)]
mod test {
    use std::sync::Arc;

    use time::Duration;

    use crate::{
        domain::{
            error::CommandSchedulerServiceError,
            message::{FermentationStep, Hardware, HardwareType, Rate, ScheduleMessageData},
        },
        port::command::MockCommandDrivenPort,
        service::command_scheduler_service::CommandSchedulerService,
    };

    #[test]
    fn should_not_validate_on_empty_step() {
        let repository = MockCommandDrivenPort::new();
        let service = CommandSchedulerService::new(Arc::new(repository));
        let err = service.validate(&[]).unwrap_err();
        assert_eq!(err, CommandSchedulerServiceError::NoFermentationStep);
    }

    #[test]
    fn should_not_validate_on_wrong_position() {
        let repository = MockCommandDrivenPort::new();
        let service = CommandSchedulerService::new(Arc::new(repository));
        let step_1 = FermentationStep {
            position: 0,
            target_temperature: 20.0,
            duration: Duration::hours(1),
            rate: None,
        };
        let step_2 = FermentationStep {
            position: 3,
            target_temperature: 20.0,
            duration: Duration::hours(1),
            rate: Some(Rate {
                value: 1,
                duration: Duration::hours(1),
            }),
        };
        let err = service.validate(&[step_1, step_2]).unwrap_err();
        assert!(matches!(
            err,
            CommandSchedulerServiceError::InvalidStepConfiguration(..)
        ));
    }
    #[test]
    fn should_not_validate_when_rate_on_first_step() {
        let repository = MockCommandDrivenPort::new();
        let service = CommandSchedulerService::new(Arc::new(repository));
        let step = FermentationStep {
            position: 0,
            target_temperature: 20.0,
            duration: Duration::hours(1),
            rate: Some(Rate {
                value: 1,
                duration: Duration::hours(1),
            }),
        };
        let steps = [step];
        let err = service.validate(&steps).unwrap_err();
        assert!(matches!(
            err,
            CommandSchedulerServiceError::InvalidStepConfiguration(..)
        ));
    }

    #[test]
    fn should_validate_steps() {
        let repository = MockCommandDrivenPort::new();
        let service = CommandSchedulerService::new(Arc::new(repository));
        let step_1 = FermentationStep {
            position: 0,
            target_temperature: 20.0,
            duration: Duration::hours(1),
            rate: None,
        };
        let step_2 = FermentationStep {
            position: 1,
            target_temperature: 20.0,
            duration: Duration::hours(1),
            rate: Some(Rate {
                value: 1,
                duration: Duration::hours(1),
            }),
        };
        service.validate(&[step_2, step_1]).unwrap();
    }

    #[test]
    fn should_correctly_calculate_number_of_command() {
        let previous_target_temp = 20.4;
        let next_target_temp = 3.2;
        let rate = 2.4;
        let amount = CommandSchedulerService::<MockCommandDrivenPort>::calculate_required_amount_of_command(
            previous_target_temp,
            next_target_temp,
            rate,
        );
        assert_eq!(amount, 8);
    }
    #[test]
    fn should_fail_building_command_if_rate_at_pos_0() {
        let step_1 = FermentationStep {
            position: 0,
            target_temperature: 20.0,
            duration: Duration::hours(96),
            rate: Some(Rate {
                value: 2,
                duration: Duration::hours(1),
            }),
        };
        let data = ScheduleMessageData {
            session_id: uuid::Uuid::new_v4(),
            hardwares: vec![
                Hardware {
                    hardware_type: HardwareType::Cooling,
                    id: "cool".into(),
                },
                Hardware {
                    hardware_type: HardwareType::Heating,
                    id: "heat".into(),
                },
            ],
            steps: vec![step_1],
        };
        let err = CommandSchedulerService::<MockCommandDrivenPort>::build_commands(&data).unwrap_err();
        assert!(matches!(err, CommandSchedulerServiceError::InvalidPosition(..)))
    }
    #[test]
    fn should_correctly_build_commands_without_rate() {
        let step_1 = FermentationStep {
            position: 0,
            target_temperature: 20.0,
            duration: Duration::hours(96),
            rate: None,
        };
        let step_2 = FermentationStep {
            position: 1,
            target_temperature: 24.0,
            duration: Duration::hours(72),
            rate: None,
        };
        let step_3 = FermentationStep {
            position: 2,
            target_temperature: 2.0,
            duration: Duration::hours(48),
            rate: None,
        };
        let data = ScheduleMessageData {
            session_id: uuid::Uuid::new_v4(),
            hardwares: vec![
                Hardware {
                    hardware_type: HardwareType::Cooling,
                    id: "cool".into(),
                },
                Hardware {
                    hardware_type: HardwareType::Heating,
                    id: "heat".into(),
                },
            ],
            steps: vec![step_1, step_2, step_3],
        };
        let new_commands = CommandSchedulerService::<MockCommandDrivenPort>::build_commands(&data).unwrap();
        assert_eq!(new_commands.len(), 3);
        let first = new_commands.first().unwrap();
        let second = new_commands.get(1).unwrap();
        let third = new_commands.get(2).unwrap();
        assert_eq!(first.value, 20.0);
        assert_eq!(first.session_data.step_position, 0);
        assert_eq!(second.value, 24.0);
        assert_eq!(second.session_data.step_position, 1);
        assert_eq!(third.value, 2.0);
        assert_eq!(third.session_data.step_position, 2);
    }

    #[test]
    fn should_correctly_build_mixed_commands() {
        let step_1 = FermentationStep {
            position: 0,
            target_temperature: 20.0,
            duration: Duration::hours(96),
            rate: None,
        };
        let step_2 = FermentationStep {
            position: 1,
            target_temperature: 24.0,
            duration: Duration::hours(72),
            rate: Some(Rate {
                value: 2,
                duration: Duration::hours(1),
            }),
        };
        let step_3 = FermentationStep {
            position: 2,
            target_temperature: 2.0,
            duration: Duration::hours(48),
            rate: Some(Rate {
                value: 4,
                duration: Duration::hours(6),
            }),
        };
        let data = ScheduleMessageData {
            session_id: uuid::Uuid::new_v4(),
            hardwares: vec![
                Hardware {
                    hardware_type: HardwareType::Cooling,
                    id: "cool".into(),
                },
                Hardware {
                    hardware_type: HardwareType::Heating,
                    id: "heat".into(),
                },
            ],
            steps: vec![step_1, step_2, step_3],
        };
        let new_commands = CommandSchedulerService::<MockCommandDrivenPort>::build_commands(&data).unwrap();
        let first = new_commands.first().unwrap();
        let second = new_commands.get(1).unwrap();
        let third = new_commands.get(2).unwrap();
        let fourth = new_commands.get(3).unwrap();
        let fifth = new_commands.get(4).unwrap();
        let sixth = new_commands.get(5).unwrap();
        let seventh = new_commands.get(6).unwrap();
        let eighth = new_commands.get(7).unwrap();
        let ninth = new_commands.last().unwrap();
        assert_eq!(new_commands.len(), 9);
        assert_eq!(first.value, 20.0);
        assert_eq!(first.session_data.step_position, 0);
        assert_eq!(second.value, 22.0);
        assert_eq!(second.session_data.step_position, 1);
        assert_eq!(third.value, 24.0);
        assert_eq!(third.session_data.step_position, 1);
        assert_eq!(fourth.value, 20.0);
        assert_eq!(fourth.session_data.step_position, 2);
        assert_eq!(fifth.value, 16.0);
        assert_eq!(fifth.session_data.step_position, 2);
        assert_eq!(sixth.value, 12.0);
        assert_eq!(sixth.session_data.step_position, 2);
        assert_eq!(seventh.value, 8.0);
        assert_eq!(seventh.session_data.step_position, 2);
        assert_eq!(eighth.value, 4.0);
        assert_eq!(eighth.session_data.step_position, 2);
        assert_eq!(ninth.value, 2.0); //target_temperature is the limit
        assert_eq!(ninth.session_data.step_position, 2);
    }
}
