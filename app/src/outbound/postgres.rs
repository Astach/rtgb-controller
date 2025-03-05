use std::future::Future;

use async_trait::async_trait;
use sqlx::{Executor, PgPool, pool, query, query_scalar};
use uuid::Uuid;

use internal::core::{
    domain::{
        command::{Command, CommandType},
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

#[async_trait]
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
    ) -> anyhow::Result<i32> {
        let c = commands
            .first()
            .ok_or(anyhow::anyhow!("No command to insert"))?;
        let mut session_record_id = query_scalar!(
            "INSERT INTO session (uuid, cooling_id, heating_id) VALUES ($1,$2,$3) RETURNING id",
            c.session_data.id,
            heating_h.id,
            cooling_h.id
        )
        .fetch_one(self.pool)
        .await;
        commands.iter().for_each(|c| {
            //            query!("INSERT INTO command (uuid, command_type)"), c.id, c.command_type, c.status, c.session_data.fermentation_step_idx, )
            todo!();
        });
        todo!()
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
