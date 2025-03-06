use std::future::Future;

use futures::{future, FutureExt};
use sqlx::{pool, postgres::PgQueryResult, query, query_scalar, Executor, PgPool};
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
        let mut session_record_id = query_scalar!(
            "INSERT INTO session (uuid, cooling_id, heating_id) VALUES ($1,$2,$3) RETURNING id",
            c.session_data.id,
            heating_h.id,
            cooling_h.id
        )
        .fetch_one(self.pool)
        .await?;

        let futures  : Vec<_>= commands.iter().map(|c| {
            query!(
                "INSERT INTO command (uuid, session_id, fermentation_step_id, command_type, holding_duration, value) VALUES($1,$2,$3,$4,$5,$6)", 
                c.id, 
                session_record_id, 
                c.session_data.fermentation_step_idx as i32,
                c.command_type.name(), 
                c.command_type.holding_duration().map(|v| v as i32),
                c.command_type.target_temp() as i32
            )
                .execute(self.pool)
        }).collect();

        Ok( futures::future::join_all(futures).await.iter()
            .filter_map(|result| {
                match result {
                    Ok(query_result) => Some(query_result.rows_affected()),
                    Err(_) => None,
                }
            })
            .sum()
        )

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
