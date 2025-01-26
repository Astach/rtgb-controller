use sqlx::{pool, query, query_scalar, Executor, PgPool};
use uuid::Uuid;

use internal::core::{
    domain::{
        command::{Command, CommandType},
        message::Hardware,
    },
    port::messaging::MessageDrivenPort,
};

pub struct MessageRepository<'a> {
    pool: &'a PgPool,
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

    async fn insert(
        &self,
        commands: Vec<Command>,
        heating_h: Hardware,
        cooling_h: Hardware,
    ) -> anyhow::Result<i32> {
        //TODO feels weird to have heating id and cooling id in every command ?
        let c = commands
            .first()
            .ok_or(anyhow::anyhow!("No command to insert"))?;
        let mut session_record_id = query_scalar!(
            "INSERT INTO session (uuid, cooling_id, heating_id) VALUES ($1,$2,$3) RETURNING id",
            c.session_data.id,
            heating.id,
            cooling.id
        )
        .fetch_one(&pool)
        .await;
        commands.iter().for_each(|c| {
            //            self.pool.execute_many()
            todo!();
        });
        todo!()
    }

    fn update(&self, command_id: Uuid) -> anyhow::Result<Command> {
        todo!()
    }

    fn delete(&self, command_id: Uuid) -> anyhow::Result<Command> {
        todo!()
    }
}

#[derive(sqlx::FromRow)]
pub struct NewCommandRecord {
    command_id: Uuid,
    command_type: String,
    holding_duration: u8,
    value: u8,
    status: String,
    session_id: Uuid,
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
impl From<Command> for NewSessionRecord {
    fn from(value: Command) -> Self {
        Self {
            session_id: value.session_data.id,
            cooling_id: value.cooling.id,
            heating_id: value.heating.id,
        }
    }
}
