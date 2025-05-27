use std::sync::Arc;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        command::{Command, CommandStatus},
        error::CommandExecutorServiceError,
        sorting::{QueryOptions, Sorting},
    },
    port::{
        command::{CommandDrivenPort, CommandExecutorDriverPort},
        publisher::PublisherDrivenPort,
    },
};

pub struct CommandExecutorService<R: CommandDrivenPort, P: PublisherDrivenPort> {
    repository: Arc<R>,
    publisher: P,
}

impl<R: CommandDrivenPort, P: PublisherDrivenPort> CommandExecutorDriverPort for CommandExecutorService<R, P> {
    async fn execute(&self, cmd: Command) -> Result<Command, CommandExecutorServiceError> {
        if cmd.status != CommandStatus::Planned {
            return Err(CommandExecutorServiceError::StatusError);
        }
        self.publisher
            .publish(&cmd)
            .await
            .map_err(|e| CommandExecutorServiceError::TechnicalError(format!("Unable to publish: {:?}", e)))?;
        self.repository
            .update(cmd.status(CommandStatus::Running {
                since: OffsetDateTime::now_utc(),
            }))
            .await
            .map_err(|e| {
                CommandExecutorServiceError::TechnicalError(format!("Unable to update status to running {:?}", e))
            })
    }

    async fn process(
        &self, tracking_message_data: crate::domain::message::TrackingMessageData,
    ) -> Result<(), CommandExecutorServiceError> {
        let status = CommandStatus::Running {
            since: OffsetDateTime::now_utc(),
        };
        let running_cmds = self.fetch_command(tracking_message_data.session_id, &status).await?;

        if running_cmds.is_empty() {
            self.execute_next_command(tracking_message_data.session_id).await; //TODO handle result
        } else {
            let cmd = running_cmds.first().cloned().unwrap();
            // target reached
            if tracking_message_data.temperature >= cmd.temparature_data.value {
                self.repository
                    .update(cmd.status(CommandStatus::Executed {
                        at: OffsetDateTime::now_utc(),
                    }))
                    .await
                    .map_err(|e| {
                        CommandExecutorServiceError::TechnicalError(format!(
                            "Unable to update status to running {:?}",
                            e
                        ))
                    })?;
                self.execute_next_command(tracking_message_data.session_id).await; //TODO handle result
            }
        }
        Ok(())
    }
}

impl<R: CommandDrivenPort, P: PublisherDrivenPort> CommandExecutorService<R, P> {
    async fn fetch_command(
        &self, session_id: Uuid, status: &CommandStatus,
    ) -> Result<Vec<Command>, CommandExecutorServiceError> {
        let options = QueryOptions::new(Some(1), Sorting::DESC);
        self.repository
            .fetch(session_id, status, options)
            .await
            .map_err(|err| CommandExecutorServiceError::TechnicalError(format!("{:?}", err.root_cause())))
    }

    async fn execute_next_command(&self, session_id: Uuid) -> Option<Result<Command, CommandExecutorServiceError>> {
        let planned_cmds = self.fetch_command(session_id, &CommandStatus::Planned).await.ok()?; //TODO in case of error we loose it (converted to none)
        if !planned_cmds.is_empty() {
            let cmd = planned_cmds.first().cloned().unwrap(); //todo map error and
            //validate there's only 1 cmd.
            Some(self.execute(cmd).await)
        } else {
            None
        }
    }
    pub fn new(repository: Arc<R>, publisher: P) -> Self {
        CommandExecutorService { repository, publisher }
    }
}
