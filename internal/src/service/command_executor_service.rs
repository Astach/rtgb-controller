use std::sync::Arc;

use log::info;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::{
        command::{Command, CommandStatus},
        error::CommandExecutorServiceError,
        message::{HardwareType, TrackingMessageData},
        sorting::{QueryOptions, Sorting},
    },
    port::{
        command::{CommandDrivenPort, CommandExecutorDriverPort},
        publisher::{HardwareAction, PublisherDrivenPort},
    },
};

pub struct CommandExecutorService<R: CommandDrivenPort, P: PublisherDrivenPort> {
    repository: Arc<R>,
    publisher: P,
}

impl<R: CommandDrivenPort, P: PublisherDrivenPort> CommandExecutorDriverPort for CommandExecutorService<R, P> {
    async fn process(
        &self, tracking_message_data: crate::domain::message::TrackingMessageData,
    ) -> Result<(), CommandExecutorServiceError> {
        let status = CommandStatus::Running {
            since: OffsetDateTime::now_utc(),
        };
        let running_cmds = self.fetch_command(tracking_message_data.session_id, &status).await?;

        if running_cmds.is_empty() {
            self.execute_next_command(tracking_message_data).await?;
        } else {
            let cmd = running_cmds.first().cloned().unwrap();
            let active_hardware = self
                .repository
                .fetch_active_hardware_type(&tracking_message_data.session_id)
                .await
                .map_err(|e| CommandExecutorServiceError::TechnicalError(e.to_string()))?
                .ok_or(CommandExecutorServiceError::NotFound("active hardware id".to_string()))?;

            let is_target_reached = match active_hardware {
                HardwareType::Cooling => tracking_message_data.temperature <= cmd.temperature_data.value,
                HardwareType::Heating => tracking_message_data.temperature >= cmd.temperature_data.value,
            };
            if is_target_reached {
                // TODO V2:  if for some reason the condition is not true anymore (electricity outage,
                // so temp is to high again) => Reset value_reached_at to None (maybe not critical
                // for now...)
                let value_reached_at = self.mark_value_as_reached(&cmd).await?;
                if Self::is_holding_duration_matched(cmd.temperature_data.value_holding_duration, value_reached_at) {
                    self.stop_all(&cmd, tracking_message_data.session_id).await?;
                    self.execute_next_command(tracking_message_data).await?;
                } else {
                    info!("target temperature has been reached for cmd {cmd:?} but holding duration isn't matched yet");
                }
            }
        }
        Ok(())
    }
}

impl<R: CommandDrivenPort, P: PublisherDrivenPort> CommandExecutorService<R, P> {
    pub fn new(repository: Arc<R>, publisher: P) -> Self {
        CommandExecutorService { repository, publisher }
    }
    async fn fetch_command(
        &self, session_id: Uuid, status: &CommandStatus,
    ) -> Result<Vec<Command>, CommandExecutorServiceError> {
        let options = QueryOptions::new(Some(1), Sorting::ASC);
        self.repository
            .fetch_commands_by_order(session_id, status, options)
            .await
            .map_err(|err| CommandExecutorServiceError::TechnicalError(err.root_cause().to_string()))
    }

    async fn get_hardware_id(
        &self, session_id: Uuid, hardware_type: &HardwareType,
    ) -> Result<String, CommandExecutorServiceError> {
        self.repository
            .fetch_hardware_id(session_id, hardware_type)
            .await
            .map_err(|e| CommandExecutorServiceError::TechnicalError(e.root_cause().to_string()))
    }

    async fn mark_value_as_reached(&self, cmd: &Command) -> Result<OffsetDateTime, CommandExecutorServiceError> {
        if let Some(d) = cmd.temperature_data.value_reached_at {
            Ok(d)
        } else {
            let date = OffsetDateTime::now_utc();
            self.repository
                .update_value_reached_at(cmd.uuid, date)
                .await
                .map_err(|e| {
                    CommandExecutorServiceError::TechnicalError(format!(
                        "Unable to update status to {:?} {e:?}",
                        &cmd.status
                    ))
                })?;
            Ok(date)
        }
    }

    fn is_holding_duration_matched(holding_duration: Duration, value_reached_at: OffsetDateTime) -> bool {
        value_reached_at + holding_duration <= OffsetDateTime::now_utc()
    }

    async fn execute_next_command(
        &self, tracking_message_data: TrackingMessageData,
    ) -> Result<(), CommandExecutorServiceError> {
        let planned_cmds = self
            .fetch_command(tracking_message_data.session_id, &CommandStatus::Planned)
            .await?;
        if planned_cmds.is_empty() {
            info!(
                "No more planned command to execute for session {:?}, profile execution is over.",
                tracking_message_data.session_id
            );
            Ok(())
        } else {
            let planned_command = planned_cmds.first().ok_or(CommandExecutorServiceError::TechnicalError(
                "Unable to find the first command in a non empty vec".to_string(),
            ))?;
            let hardware_type = if planned_command.temperature_data.value > tracking_message_data.temperature {
                HardwareType::Heating
            } else {
                HardwareType::Cooling
            };

            let hardware_id = self
                .get_hardware_id(tracking_message_data.session_id, &hardware_type)
                .await?;

            let action = HardwareAction::START(hardware_id);
            let status = CommandStatus::Running {
                since: OffsetDateTime::now_utc(),
            };
            self.publisher
                .publish(action)
                .await
                .map_err(|e| CommandExecutorServiceError::TechnicalError(format!("Unable to publish: {e}")))?;
            self.repository
                .update_active_hardware_type(tracking_message_data.session_id, Some(hardware_type))
                .await
                .map_err(|e| {
                    CommandExecutorServiceError::TechnicalError(format!("Unable to update active hardware type: {e}"))
                })?;
            self.repository
                .update_status(planned_command.uuid, &status)
                .await
                .map(|_| ())
                .map_err(|e| {
                    CommandExecutorServiceError::TechnicalError(format!(
                        "Unable to update status to {:?} {e:?}",
                        &status
                    ))
                })
        }
    }
    async fn stop_all(&self, cmd: &Command, session_id: Uuid) -> Result<(), CommandExecutorServiceError> {
        let heating_hw_id = self.get_hardware_id(session_id, &HardwareType::Heating).await?;
        let cooling_hw_id = self.get_hardware_id(session_id, &HardwareType::Cooling).await?;
        self.publisher
            .publish(HardwareAction::STOP(heating_hw_id))
            .await
            .map_err(|e| CommandExecutorServiceError::TechnicalError(format!("Unable to publish: {e}")))?;
        self.publisher
            .publish(HardwareAction::STOP(cooling_hw_id))
            .await
            .map_err(|e| CommandExecutorServiceError::TechnicalError(format!("Unable to publish: {e}")))?;
        let status = CommandStatus::Executed {
            at: OffsetDateTime::now_utc(),
        };
        self.repository
            .update_active_hardware_type(session_id, None)
            .await
            .map_err(|e| {
                CommandExecutorServiceError::TechnicalError(format!("Unable to update active hardware type: {e}"))
            })?;
        self.repository
            .update_status(cmd.uuid, &status)
            .await
            .map(|_| ())
            .map_err(|e| {
                CommandExecutorServiceError::TechnicalError(format!("Unable to update status to {:?} {e:?}", &status))
            })
    }
}

#[cfg(test)]
mod test {
    use std::{future::ready, mem::discriminant, sync::Arc};

    use time::{Duration, OffsetDateTime};

    use crate::{
        domain::{
            command::{Command, CommandStatus, CommandTemperatureData},
            error::CommandExecutorServiceError,
            message::{HardwareType, TrackingMessageData},
        },
        port::{
            command::{CommandExecutorDriverPort, MockCommandDrivenPort},
            publisher::{HardwareAction, MockPublisherDrivenPort},
        },
        service::command_executor_service::CommandExecutorService,
    };

    #[tokio::test]
    async fn should_not_update_value_reached_at_if_already_done() {
        let mut repository = MockCommandDrivenPort::new();
        repository.expect_update_status().never();
        repository.expect_update_value_reached_at().never();
        let publisher = MockPublisherDrivenPort::new();
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        let mut cmd = Command::default();
        let reached_date = OffsetDateTime::now_utc();
        cmd.temperature_data.value_reached_at = Some(reached_date);
        let result = service.mark_value_as_reached(&cmd).await.unwrap();
        assert_eq!(reached_date, result);
    }
    #[tokio::test]
    async fn should_update_value_reached_at() {
        let mut repository = MockCommandDrivenPort::new();
        repository.expect_update_status().never();
        repository
            .expect_update_value_reached_at()
            .once()
            .return_once(move |_, date| {
                let mut cmd = Command::default();
                cmd.temperature_data.value_reached_at = Some(date);
                Box::pin(ready(Ok(cmd)))
            });
        let cmd = Command::default();
        let publisher = MockPublisherDrivenPort::new();
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        assert!(cmd.temperature_data.value_reached_at.is_none());
        service.mark_value_as_reached(&cmd).await.unwrap();
    }
    #[test]
    fn is_holding_duration_matched_should_return_false() {
        assert!(
            !CommandExecutorService::<MockCommandDrivenPort, MockPublisherDrivenPort>::is_holding_duration_matched(
                Duration::hours(5),
                OffsetDateTime::now_utc()
            )
        );
    }
    #[test]
    fn is_holding_duration_matched_should_return_true() {
        assert!(
            CommandExecutorService::<MockCommandDrivenPort, MockPublisherDrivenPort>::is_holding_duration_matched(
                Duration::hours(5),
                OffsetDateTime::now_utc() - Duration::hours(5)
            )
        );
    }
    #[tokio::test]
    async fn execute_next_command_should_do_nothing_if_no_planned_commands() {
        let mut repository = MockCommandDrivenPort::new();
        let mut publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData::default();

        repository.expect_update_value_reached_at().never();
        repository.expect_update_status().never();
        repository
            .expect_fetch_commands_by_order()
            .return_once(|_, _, _| Box::pin(ready(Ok(vec![]))));

        publisher.expect_publish().never();
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.execute_next_command(tracking_data).await.unwrap();
    }
    #[tokio::test]
    async fn execute_next_command_should_publish_start_action_for_heating_hardware() {
        let mut repository = MockCommandDrivenPort::new();
        let mut publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData {
            temperature: 16.0,
            ..Default::default()
        };

        repository.expect_fetch_commands_by_order().return_once(|_, _, _| {
            Box::pin(ready(Ok(vec![Command {
                temperature_data: CommandTemperatureData {
                    value: 20.0,
                    ..Default::default()
                },
                ..Default::default()
            }])))
        });
        repository
            .expect_fetch_hardware_id()
            .withf(move |session_id, hardware_type| {
                *session_id == tracking_data.session_id && *hardware_type == HardwareType::Heating
            })
            .return_once(|_, _| Box::pin(ready(Ok("heating_hw_id".into()))));

        repository.expect_update_value_reached_at().never();
        repository
            .expect_update_status()
            .withf(move |&session_id, status| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Running {
                            since: OffsetDateTime::now_utc(),
                        })
            })
            .once()
            .return_once(|_, _| Box::pin(ready(Ok(Command::default()))));

        publisher
            .expect_publish()
            .withf(|hardware_action| *hardware_action == HardwareAction::START("heating_hw_id".to_string()))
            .return_once(|_| Box::pin(ready(Ok(()))))
            .once();

        repository
            .expect_update_active_hardware_type()
            .withf(|_, hardware_type| hardware_type.as_ref().is_some_and(|t| *t == HardwareType::Heating))
            .return_once(|_, _| Box::pin(ready(Ok(()))))
            .once();
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.execute_next_command(tracking_data).await.unwrap();
    }
    #[tokio::test]
    async fn execute_next_command_should_publish_start_action_for_cooling_hardware() {
        let mut repository = MockCommandDrivenPort::new();
        let mut publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData {
            temperature: 22.0,
            ..Default::default()
        };

        repository.expect_fetch_commands_by_order().return_once(|_, _, _| {
            Box::pin(ready(Ok(vec![Command {
                temperature_data: CommandTemperatureData {
                    value: 20.0,
                    ..Default::default()
                },
                ..Default::default()
            }])))
        });
        repository
            .expect_fetch_hardware_id()
            .withf(move |session_id, hardware_type| {
                *session_id == tracking_data.session_id && *hardware_type == HardwareType::Cooling
            })
            .return_once(|_, _| Box::pin(ready(Ok("cooling_hw_id".into()))));

        repository.expect_update_value_reached_at().never();
        repository
            .expect_update_status()
            .withf(move |&session_id, status| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Running {
                            since: OffsetDateTime::now_utc(),
                        })
            })
            .once()
            .return_once(|_, _| Box::pin(ready(Ok(Command::default()))));

        publisher
            .expect_publish()
            .withf(|hardware_action| *hardware_action == HardwareAction::START("cooling_hw_id".to_string()))
            .return_once(|_| Box::pin(ready(Ok(()))))
            .once();

        repository
            .expect_update_active_hardware_type()
            .withf(|_, hardware_type| hardware_type.as_ref().is_some_and(|t| *t == HardwareType::Cooling))
            .return_once(|_, _| Box::pin(ready(Ok(()))))
            .once();
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.execute_next_command(tracking_data).await.unwrap();
    }
    #[tokio::test]
    async fn stop_all_should_publish_stop_action_for_cooling_and_heating_hardware() {
        let mut repository = MockCommandDrivenPort::new();
        let mut publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData::default();
        let cmd = Command::default();
        repository
            .expect_fetch_hardware_id()
            .withf(move |session_id, hardware_type| {
                *session_id == tracking_data.session_id && *hardware_type == HardwareType::Cooling
            })
            .once()
            .return_once(|_, _| Box::pin(ready(Ok("cooling_hw_id".into()))));
        repository
            .expect_fetch_hardware_id()
            .withf(move |session_id, hardware_type| {
                *session_id == tracking_data.session_id && *hardware_type == HardwareType::Heating
            })
            .once()
            .return_once(|_, _| Box::pin(ready(Ok("heating_hw_id".into()))));
        publisher
            .expect_publish()
            .withf(|hardware_action| *hardware_action == HardwareAction::STOP("heating_hw_id".to_string()))
            .return_once(|_| Box::pin(ready(Ok(()))))
            .once();
        publisher
            .expect_publish()
            .withf(|hardware_action| *hardware_action == HardwareAction::STOP("cooling_hw_id".to_string()))
            .return_once(|_| Box::pin(ready(Ok(()))))
            .once();
        repository
            .expect_update_active_hardware_type()
            .withf(|_, hardware_type| hardware_type.is_none())
            .return_once(|_, _| Box::pin(ready(Ok(()))))
            .once();
        repository.expect_update_value_reached_at().never();
        repository
            .expect_update_status()
            .withf(move |&session_id, status| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Executed {
                            at: OffsetDateTime::now_utc(),
                        })
            })
            .once()
            .return_once(|_, _| Box::pin(ready(Ok(Command::default()))));
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.stop_all(&cmd, tracking_data.session_id).await.unwrap()
    }

    #[tokio::test]
    async fn process_should_execute_next_command_if_no_command_is_running() {
        let mut repository = MockCommandDrivenPort::new();
        let publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData::default();
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Running {
                            since: OffsetDateTime::now_utc(),
                        })
            })
            .once()
            .return_once(|_, _, _| Box::pin(ready(Ok(vec![]))));

        //Called in execute_next_command
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id && discriminant(status) == discriminant(&CommandStatus::Planned)
            })
            .once()
            .return_once(|_, _, _| Box::pin(ready(Ok(vec![]))));
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.process(tracking_data).await.unwrap();
    }

    #[tokio::test]
    async fn process_should_update_heating_command_as_executed() {
        let mut repository = MockCommandDrivenPort::new();
        let mut publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData {
            temperature: 21.0,
            ..Default::default()
        };
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Running {
                            since: OffsetDateTime::now_utc(),
                        })
            })
            .return_once(|_, _, _| {
                Box::pin(ready(Ok(vec![Command {
                    temperature_data: CommandTemperatureData {
                        value: 20.0,
                        value_holding_duration: Duration::hours(0),
                        ..Default::default()
                    },
                    ..Default::default()
                }])))
            });
        repository
            .expect_fetch_active_hardware_type()
            .return_once(|_| Box::pin(ready(Ok(Some(HardwareType::Heating)))));
        repository
            .expect_update_value_reached_at()
            .once()
            .returning(|_, _| Box::pin(ready(Ok(Command::default())))); //mark as reached
        repository
            .expect_fetch_hardware_id()
            .times(2)
            .returning(|_, _| Box::pin(ready(Ok("hardware_id".to_string())))); //stop all
        publisher
            .expect_publish()
            .times(2)
            .returning(|_| Box::pin(ready(Ok(())))); //stop all 
        repository
            .expect_update_active_hardware_type()
            .withf(|_, hardware_type| hardware_type.is_none())
            .return_once(|_, _| Box::pin(ready(Ok(()))))
            .once();

        repository
            .expect_update_status()
            .withf(|_, status| {
                discriminant(status)
                    == discriminant(&CommandStatus::Executed {
                        at: OffsetDateTime::now_utc(),
                    })
            })
            .once()
            .returning(|_, _| Box::pin(ready(Ok(Command::default())))); //stop all //stop all 
        //Called in execute_next_command
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id && discriminant(status) == discriminant(&CommandStatus::Planned)
            })
            .once()
            .return_once(|_, _, _| Box::pin(ready(Ok(vec![]))));
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.process(tracking_data).await.unwrap();
    }

    #[tokio::test]
    async fn process_should_update_cooling_command_as_executed() {
        let mut repository = MockCommandDrivenPort::new();
        let mut publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData {
            temperature: 21.0,
            ..Default::default()
        };
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Running {
                            since: OffsetDateTime::now_utc(),
                        })
            })
            .return_once(|_, _, _| {
                Box::pin(ready(Ok(vec![Command {
                    temperature_data: CommandTemperatureData {
                        value: 23.0,
                        value_holding_duration: Duration::hours(0),
                        ..Default::default()
                    },
                    ..Default::default()
                }])))
            });
        repository
            .expect_fetch_active_hardware_type()
            .return_once(|_| Box::pin(ready(Ok(Some(HardwareType::Cooling)))));
        repository
            .expect_update_value_reached_at()
            .once()
            .returning(|_, _| Box::pin(ready(Ok(Command::default())))); //mark as reached
        repository
            .expect_fetch_hardware_id()
            .times(2)
            .returning(|_, _| Box::pin(ready(Ok("hardware_id".to_string())))); //stop all
        publisher
            .expect_publish()
            .times(2)
            .returning(|_| Box::pin(ready(Ok(())))); //stop all 
        repository
            .expect_update_active_hardware_type()
            .withf(|_, hardware_type| hardware_type.is_none())
            .return_once(|_, _| Box::pin(ready(Ok(()))))
            .once();

        repository
            .expect_update_status()
            .withf(|_, status| {
                discriminant(status)
                    == discriminant(&CommandStatus::Executed {
                        at: OffsetDateTime::now_utc(),
                    })
            })
            .once()
            .returning(|_, _| Box::pin(ready(Ok(Command::default())))); //stop all //stop all 
        //Called in execute_next_command
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id && discriminant(status) == discriminant(&CommandStatus::Planned)
            })
            .once()
            .return_once(|_, _, _| Box::pin(ready(Ok(vec![]))));
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.process(tracking_data).await.unwrap();
    }

    #[tokio::test]
    async fn process_should_err_if_no_active_hardware() {
        let mut repository = MockCommandDrivenPort::new();
        let publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData {
            temperature: 18.0,
            ..Default::default()
        };
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Running {
                            since: OffsetDateTime::now_utc(),
                        })
            })
            .return_once(|_, _, _| {
                Box::pin(ready(Ok(vec![Command {
                    temperature_data: CommandTemperatureData {
                        value: 11.0,
                        value_holding_duration: Duration::hours(0),
                        ..Default::default()
                    },
                    ..Default::default()
                }])))
            });
        repository
            .expect_fetch_active_hardware_type()
            .return_once(|_| Box::pin(ready(Ok(None))));
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        let result = service.process(tracking_data).await.unwrap_err();
        assert_eq!(
            discriminant(&result),
            discriminant(&CommandExecutorServiceError::NotFound("test".to_string()))
        );
    }

    #[tokio::test]
    async fn process_should_do_nothing_if_running_command_target_temp_is_not_reached_for_cooling_hardware() {
        let mut repository = MockCommandDrivenPort::new();
        let publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData {
            temperature: 18.0,
            ..Default::default()
        };
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Running {
                            since: OffsetDateTime::now_utc(),
                        })
            })
            .return_once(|_, _, _| {
                Box::pin(ready(Ok(vec![Command {
                    temperature_data: CommandTemperatureData {
                        value: 11.0,
                        value_holding_duration: Duration::hours(0),
                        ..Default::default()
                    },
                    ..Default::default()
                }])))
            });
        repository
            .expect_fetch_active_hardware_type()
            .return_once(|_| Box::pin(ready(Ok(Some(HardwareType::Cooling)))));
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.process(tracking_data).await.unwrap();
    }

    #[tokio::test]
    async fn process_should_do_nothing_if_running_command_target_temp_is_not_reached_for_heating_hardware() {
        let mut repository = MockCommandDrivenPort::new();
        let publisher = MockPublisherDrivenPort::new();
        let tracking_data = TrackingMessageData {
            temperature: 18.0,
            ..Default::default()
        };
        repository
            .expect_fetch_commands_by_order()
            .withf(move |&session_id, status, _| {
                session_id == tracking_data.session_id
                    && discriminant(status)
                        == discriminant(&CommandStatus::Running {
                            since: OffsetDateTime::now_utc(),
                        })
            })
            .return_once(|_, _, _| {
                Box::pin(ready(Ok(vec![Command {
                    temperature_data: CommandTemperatureData {
                        value: 20.0,
                        value_holding_duration: Duration::hours(0),
                        ..Default::default()
                    },
                    ..Default::default()
                }])))
            });
        repository
            .expect_fetch_active_hardware_type()
            .return_once(|_| Box::pin(ready(Ok(Some(HardwareType::Heating)))));
        let service = CommandExecutorService::new(Arc::new(repository), publisher);
        service.process(tracking_data).await.unwrap();
    }
}
