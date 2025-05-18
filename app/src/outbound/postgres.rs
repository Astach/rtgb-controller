use std::{
    fmt::{Debug, format},
    str::FromStr,
};

use anyhow::bail;
use async_nats::jetstream::new;
use bigdecimal::ToPrimitive;
use log::debug;
use sqlx::{PgPool, query, query_as, query_scalar, types::BigDecimal};
use time::{PrimitiveDateTime, UtcOffset};
use uuid::Uuid;

use internal::{
    domain::{
        command::{Command, CommandStatus, CommandTemperatureData, NewCommand},
        error::MessageServiceError,
        message::Hardware,
        sorting::{self, QueryOptions, Sorting},
    },
    port::messaging::MessageDrivenPort,
};

pub struct MessageRepository<'a> {
    pub pool: &'a PgPool,
    command_table: &'a str,
    session_table: &'a str,
}

impl<'a> MessageRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            command_table: "command",
            session_table: "session",
        }
    }
}

impl MessageDrivenPort for MessageRepository<'_> {
    async fn insert(
        &self,
        commands: Vec<NewCommand>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> anyhow::Result<u64> {
        let c = commands
            .first()
            .ok_or(anyhow::anyhow!("No command to insert"))?;
        let sql_query = format!(
            "INSERT INTO {:?} (uuid, cooling_id, heating_id) VALUES ($1,$2,$3) RETURNING id",
            self.session_table
        );
        let session_record_id = query_scalar(sql_query.as_str())
            .bind(c.session_data.id)
            .bind(heating_h.id)
            .bind(cooling_h.id)
            .fetch_one(self.pool)
            .await?;
        debug!("Inserted session with id {session_record_id}");
        let records = commands
            .iter()
            .map(|c| NewCommandRecord::from_command(c, session_record_id))
            .collect::<anyhow::Result<Vec<NewCommandRecord>>>()?;
        let sql_query = format!(
            "INSERT INTO {:?} (uuid, fermentation_step_id, status, status_date, value, value_reached_at,value_holding_duration, session_id) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
            self.command_table
        );
        let futures: Vec<_> = records
            .iter()
            .map(|rec| {
                query(sql_query.as_str())
                    .bind(rec.command_id)
                    .bind(rec.fermentation_step_id)
                    .bind(rec.status.clone())
                    .bind(rec.status_date)
                    .bind(rec.value.clone())
                    .bind(None as Option<PrimitiveDateTime>)
                    .bind(rec.value_holding_duration)
                    .bind(rec.session_id)
                    .execute(self.pool)
            })
            .collect();

        futures::future::join_all(futures)
            .await
            .into_iter()
            .try_fold(0, |acc, result| {
                result
                    .map_err(|e| anyhow::anyhow!("Can't execute command insert {}", e))
                    .map(|query_result| acc + query_result.rows_affected())
            })
    }

    async fn fetch(
        &self,
        session_uuid: Uuid,
        status: &CommandStatus,
        options: QueryOptions,
    ) -> anyhow::Result<Vec<Command>> {
        let limit = options
            .limit
            .map_or("".to_string(), |n| format!("LIMIT {}", n));
        let sql_query = format!(
            r#"SELECT 
                {command_table}.uuid,
                {command_table}.fermentation_step_id,
                {command_table}.status,
                {command_table}.status_date,
                {command_table}.value,
                {command_table}.value_reached_at,
                {command_table}.value_holding_duration,
                {command_table}.session_id
             FROM {command_table}
                INNER JOIN {session_table} ON {command_table}.session_id = {session_table}.id
                WHERE {command_table}.status = $1 AND {session_table}.uuid = $2
                {limit}
               ORDER BY 
                {command_table}.updated_at {order}
               "#,
            command_table = self.command_table,
            session_table = self.session_table,
            limit = limit,
            order = options.sorting
        );
        let res: Vec<CommandRecord> = query_as(&sql_query)
            .bind(status.name())
            .bind(session_uuid)
            .fetch_all(self.pool)
            .await?;
        res.iter().map(Command::try_from).collect()
    }

    async fn update(
        &self,
        session_uuid: Uuid,
        new_status: CommandStatus,
    ) -> anyhow::Result<Command> {
        let date = match new_status {
            CommandStatus::Planned => bail!("Command can't be updated to Planned"),
            CommandStatus::Running { since } => since,
            CommandStatus::Executed { at } => at,
        };
        let date = PrimitiveDateTime::new(date.date(), date.time());
        let sql_query = format!(
            r#"UPDATE {command_table}
        SET
            status = $1,
            status_date = $2
        FROM {session_table} 
        WHERE {command_table}.session_id = {session_table}.id 
        AND {session_table}.uuid = $3
        RETURNING {command_table}.*"#,
            command_table = self.command_table,
            session_table = self.session_table
        );

        let updated_command_record: CommandRecord = query_as(&sql_query)
            .bind(new_status.name())
            .bind(date)
            .bind(session_uuid)
            .fetch_one(self.pool)
            .await?;
        Command::try_from(&updated_command_record)
    }

    fn delete(&self, command_id: Uuid) -> anyhow::Result<u64> {
        todo!()
    }
}

#[derive(sqlx::FromRow)]
struct CommandRecord {
    pub uuid: Uuid,
    pub fermentation_step_id: i32,
    pub status: String,
    pub status_date: Option<PrimitiveDateTime>,
    pub value: BigDecimal,
    pub value_reached_at: Option<PrimitiveDateTime>,
    pub value_holding_duration: i32,
    pub session_id: i32,
}
impl CommandRecord {
    fn status_to_command_status(
        &self,
        date: Option<PrimitiveDateTime>,
    ) -> anyhow::Result<CommandStatus> {
        Ok(match self.status.as_str() {
            "Planned" => CommandStatus::Planned,
            "Running" => CommandStatus::Running {
                since: date.map(|d| d.assume_offset(UtcOffset::UTC)).ok_or(
                    MessageServiceError::NotFound("date for running command status".to_string()),
                )?,
            },
            "Executed" => CommandStatus::Executed {
                at: date.map(|d| d.assume_offset(UtcOffset::UTC)).ok_or(
                    MessageServiceError::NotFound("date for executed command status".to_string()),
                )?,
            },
            _ => bail!("{} is not a valid status", self.status.as_str()),
        })
    }
}
impl TryFrom<&CommandRecord> for Command {
    type Error = anyhow::Error;

    fn try_from(record: &CommandRecord) -> Result<Self, Self::Error> {
        Ok(Command {
            uuid: record.uuid,
            fermentation_step_id: record.fermentation_step_id,
            status: record.status_to_command_status(record.status_date)?,
            temparature_data: CommandTemperatureData {
                value: record
                    .value
                    .to_f32()
                    .ok_or(MessageServiceError::ConversionError("record value", "f32"))?,
                value_reached_at: record
                    .value_reached_at
                    .map(|p_date| p_date.assume_offset(UtcOffset::UTC)),
                value_holding_duration: record.value_holding_duration.to_u8().ok_or(
                    MessageServiceError::ConversionError("record.value_holding_duration", "f32"),
                )?,
            },
            session_id: record.session_id,
        })
    }
}

struct NewCommandRecord {
    pub command_id: Uuid,
    pub fermentation_step_id: i32,
    pub status: String,
    pub status_date: Option<PrimitiveDateTime>,
    pub value: BigDecimal,
    pub value_holding_duration: i32,
    pub session_id: i32,
}

impl NewCommandRecord {
    fn from_command(command: &NewCommand, session_record_id: i32) -> anyhow::Result<Self> {
        Ok(Self {
            command_id: command.id,
            fermentation_step_id: command.session_data.step_position as i32,
            status: command.status.name().into(),
            status_date: command
                .status
                .date()
                .map(|d| PrimitiveDateTime::new(d.date(), d.time())),
            value: BigDecimal::from_str(&format!("{:.1}", command.value))?.with_scale(1),
            value_holding_duration: command.value_holding_duration as i32,
            session_id: session_record_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{env, str::FromStr};

    use crate::outbound::postgres::CommandRecord;

    use super::{MessageRepository, NewCommandRecord};
    use internal::{
        domain::{
            command::{Command, CommandStatus, NewCommand},
            message::{Hardware, HardwareType},
            sorting::{QueryOptions, Sorting},
        },
        port::messaging::MessageDrivenPort,
    };
    use sqlx::{PgPool, query, query_scalar, types::BigDecimal};
    use time::{OffsetDateTime, PrimitiveDateTime};
    use uuid::Uuid;

    #[test]
    fn should_create_new_command_record() {
        let session_id = 1;
        let record = NewCommandRecord::from_command(&NewCommand::default(), 1).unwrap();
        assert_eq!(record.value_holding_duration, 0);
        assert_eq!(record.value, BigDecimal::from_str("0.0").unwrap());
        assert_eq!(record.fermentation_step_id, 0);
        assert_eq!(record.command_id, Uuid::default());
        assert_eq!(record.session_id, session_id);
        assert_eq!(record.status, "Planned");
        assert_eq!(record.status_date, None);
    }
    #[sqlx::test(migrations = "./migrations")]
    async fn should_insert_commands(pool: PgPool) -> anyhow::Result<()> {
        let repo = MessageRepository::new(&pool);
        let cmds = vec![NewCommand::default()];
        let heating_h = Hardware::new(String::from("heating_id"), HardwareType::Heating);
        let cooling_h = Hardware::new(String::from("cooling_id"), HardwareType::Cooling);
        let result = repo.insert(cmds, heating_h, cooling_h).await;
        assert_eq!(result.unwrap(), 1);
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations", fixtures("session", "command"))]
    async fn should_fetch_commands(pool: PgPool) -> anyhow::Result<()> {
        let repo = MessageRepository::new(&pool);

        let result = repo
            .fetch(
                Uuid::parse_str("871b888e-2185-4bb8-b8b0-f87d4be4c133").unwrap(),
                &CommandStatus::Planned,
                QueryOptions::new(None, Sorting::DESC),
            )
            .await;
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.first().unwrap().status, CommandStatus::Planned);
        assert_eq!(
            result.first().unwrap().uuid,
            Uuid::parse_str("23bc0b04-05a4-4d28-a82d-2cc640fb3042").unwrap()
        );
        Ok(())
    }
    #[sqlx::test(migrations = "./migrations", fixtures("session", "command"))]
    async fn should_update_command_status(pool: PgPool) -> anyhow::Result<()> {
        let repo = MessageRepository::new(&pool);
        let since = {
            let dt = OffsetDateTime::now_utc();
            let microseconds = dt.nanosecond() / 1000;
            dt.replace_nanosecond(microseconds * 1000).unwrap()
        };
        let result = repo
            .update(
                Uuid::parse_str("871b888e-2185-4bb8-b8b0-f87d4be4c133").unwrap(),
                CommandStatus::Running { since },
            )
            .await;
        let result = result.unwrap();
        assert_eq!(result.session_id, 1);
        assert_eq!(result.fermentation_step_id, 1);
        assert_eq!(result.temparature_data.value, 20.4);
        assert_eq!(result.temparature_data.value_reached_at, None);
        assert_eq!(result.temparature_data.value_holding_duration, 1);
        assert_eq!(result.status, CommandStatus::Running { since });
        assert_eq!(
            result.uuid,
            Uuid::parse_str("23bc0b04-05a4-4d28-a82d-2cc640fb3042").unwrap()
        );
        Ok(())
    }
}
