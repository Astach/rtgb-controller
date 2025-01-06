use sqlx::PgPool;
use uuid::Uuid;

use crate::core::port::messaging::MessageDrivenPort;

pub struct MessageRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> MessageRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl MessageDrivenPort for MessageRepository<'_> {
    fn fetch(&self) {
        todo!()
    }

    fn insert(&self) {
        todo!()
    }

    fn update(&self) {
        todo!()
    }

    fn delete(&self) {
        todo!()
    }
}
#[derive(sqlx::FromRow)]
pub struct MessageRecord {
    id: Uuid,
    command_id: Uuid,
    session_id: Uuid,
    target_id: String,
    command_type: String,
    value: u16,
    status: String,
}
