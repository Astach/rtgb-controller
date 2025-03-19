
use log::{debug, error};
use sqlx::{ query, query_scalar, types::BigDecimal, PgPool};
use uuid::Uuid;

use internal::core::{
    domain::{
        command::Command,
        message::Hardware,
    },
    port::messaging::MessageDrivenPort,
};

pub struct MessageRepository<'a> {
    pub pool: &'a PgPool,
}

impl<'a> MessageRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl MessageDrivenPort for MessageRepository<'_> {
    fn fetch(&self, command_id: Uuid) -> Option<Command> {
        todo!()
    }

    fn update(&self, command_id: Uuid) -> anyhow::Result<Command> {
        todo!()
    }

    fn delete(&self, command_id: Uuid) -> anyhow::Result<Command> {
        todo!()
    }

    async fn insert(
        &self,
        commands: Vec<Command>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> anyhow::Result<u64> {
        let c = commands
            .first()
            .ok_or(anyhow::anyhow!("No command to insert"))?;
        let  session_record_id = query_scalar!(
            "INSERT INTO session (uuid, cooling_id, heating_id) VALUES ($1,$2,$3) RETURNING id",
            c.session_data.id,
            heating_h.id,
            cooling_h.id
        )
        .fetch_one(self.pool)
        .await?;
        debug!("Inserted session with id {session_record_id}");
        let futures  : Vec<_>= commands.iter().map(|c| {
            query!(
                "INSERT INTO command (uuid, session_id, fermentation_step_id, command_type, holding_duration, value) VALUES ($1,$2,$3,$4,$5,$6);", 
                c.id, 
                session_record_id, 
                c.session_data.fermentation_step_idx as i32,
                c.command_type.name(), 
                c.command_type.holding_duration().map(|v| v as i32),
                BigDecimal::from(c.command_type.target_temp() )
            )
                .execute(self.pool)
        }).collect();
        
futures::future::join_all(futures)
    .await
    .into_iter()
    .try_fold(0, |acc, result| {
        result
            .map_err(|e| {
                error!("Can't execute command insert {e}");
                anyhow::anyhow!("Can't execute command insert {}", e)
            })
            .map(|query_result| {
                debug!("Inserted command result {:?}", query_result);
                acc + query_result.rows_affected()
            })
    })

    }
}

#[derive(sqlx::FromRow)]
pub struct CommandRecord {
    command_id: Uuid,
    command_type: String,
    holding_duration: u8,
    value: u8,
    status: String,
}

#[derive(sqlx::FromRow)]
pub struct SessionRecord {
    id: i32,
    session_id: Uuid,
    cooling_id: String,
    heating_id: String,
}

#[cfg(test)]
mod tests {
    use internal::core::{domain::{command::Command, message::{Hardware, HardwareType}}, port::messaging::MessageDrivenPort};
    use sqlx::PgPool;


    use super::MessageRepository;
    #[sqlx::test(migrations = "./migrations")]
    async fn can_insert_commands(pool: PgPool) -> anyhow::Result<()>{
    env_logger::init();
        let repo = MessageRepository::new(&pool);
        let cmds = vec![Command::default()];
        let heating_h = Hardware::new(String::from("heating_id"), HardwareType::Heating);
        let cooling_h = Hardware::new(String::from("cooling_id"), HardwareType::Cooling);
        let result = repo.insert(cmds, heating_h, cooling_h).await;
        assert_eq!(result.unwrap(), 1);
        Ok(())
    }
}
