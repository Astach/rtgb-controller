use std::{fmt::format, str::FromStr};

use log::debug;
use sqlx::{PgPool, query, query_scalar, types::BigDecimal};
use time::PrimitiveDateTime;
use uuid::Uuid;

use internal::{
    domain::{command::NewCommand, message::Hardware},
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
    async fn fetch(&self, command_id: Uuid) -> Option<NewCommand> {
        todo!()
    }

    fn update(&self, command_id: Uuid) -> anyhow::Result<NewCommand> {
        todo!()
    }
    fn delete(&self, command_id: Uuid) -> anyhow::Result<NewCommand> {
        todo!()
    }

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
            "INSERT INTO {:?} (uuid, fermentation_step_id, status, status_date,value, value_reached_at,value_holding_duration, session_id) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
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
    fn from_command(command: &NewCommand, session_id: i32) -> anyhow::Result<Self> {
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
            session_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use internal::{
        domain::{
            command::NewCommand,
            message::{Hardware, HardwareType},
        },
        port::messaging::MessageDrivenPort,
    };
    use sqlx::{PgPool, types::BigDecimal};
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
    use super::{MessageRepository, NewCommandRecord};
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
}
