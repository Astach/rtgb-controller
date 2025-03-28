
use std::str::FromStr;

use log::debug;
use sqlx::{query, query_scalar, types::BigDecimal, PgPool};
use time::PrimitiveDateTime;
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
    async fn fetch(&self, command_id: Uuid) -> Option<Command> {
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
        let records = commands.iter().map(|c| NewCommandRecord::from_command(c, session_record_id)).collect::<anyhow::Result<Vec<NewCommandRecord>>>()?; 
        let futures  : Vec<_>= records.iter().map(|rec| {
            query!(
                "INSERT INTO command (uuid, session_id, fermentation_step_id, command_type, holding_duration, value, status, status_date) VALUES ($1,$2,$3,$4,$5,$6,$7,$8);", 
                rec.command_id, 
                rec.session_id,
                rec.fermentation_step_id,
                rec.command_type,
                rec.holding_duration,
                rec.value,
                rec.status,
                rec.status_date
            ).execute(self.pool)
        }).collect();

        futures::future::join_all(futures)
            .await
            .into_iter()
            .try_fold(0, |acc, result| {
                result
                    .map_err(|e| {
                        anyhow::anyhow!("Can't execute command insert {}", e)
                    })
                    .map(|query_result| {
                        acc + query_result.rows_affected()
                    })
            })

    }
}

struct NewCommandRecord{
    pub command_id: Uuid,
    pub session_id : i32,
    pub fermentation_step_id : i32,
    pub command_type: String,
    pub holding_duration: i32,
    pub value: BigDecimal,
    pub status: String,
    pub status_date: Option<PrimitiveDateTime>,
}

impl NewCommandRecord {
   fn from_command(command : &Command, session_id : i32) -> anyhow::Result<Self>{
        debug!("{:?}", &format!("{:.1}", command.command_type.target_temp()));
        Ok(Self{
            command_id: command.id,
            session_id,
            fermentation_step_id: command.session_data.fermentation_step_idx as i32,
            command_type: command.command_type.name().to_string(),
            holding_duration: command.command_type.holding_duration().map_or( 0 , |v| v as i32),
            value: BigDecimal::from_str(&format!("{:.1}", command.command_type.target_temp()))?.with_scale(1),
            status: command.status.name().into(),
            status_date: command.status.date().map(|d| PrimitiveDateTime::new(d.date(), d.time()))
        })
    } 
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use internal::core::{domain::{command::Command, message::{Hardware, HardwareType}}, port::messaging::MessageDrivenPort};
    use sqlx::{types::BigDecimal, PgPool};
    use uuid::Uuid;

#[test]
fn should_create_new_command_record(){
        let session_id = 1;
       let record = NewCommandRecord::from_command(&Command::default(), 1).unwrap();
        assert_eq!(record.command_type, "StartFermentation");
        assert_eq!(record.holding_duration, 0);
        assert_eq!(record.value, BigDecimal::from_str("20.0").unwrap());
        assert_eq!(record.fermentation_step_id, 0);
        assert_eq!(record.command_id, Uuid::default());
        assert_eq!(record.session_id, session_id);
        assert_eq!(record.status, "Planned");
        assert_eq!(record.status_date, None );
    }
    use super::{MessageRepository, NewCommandRecord};
    #[sqlx::test(migrations = "./migrations")]
    async fn should_insert_commands(pool: PgPool) -> anyhow::Result<()>{
        let repo = MessageRepository::new(&pool);
        let cmds = vec![Command::default()];
        let heating_h = Hardware::new(String::from("heating_id"), HardwareType::Heating);
        let cooling_h = Hardware::new(String::from("cooling_id"), HardwareType::Cooling);
        let result = repo.insert(cmds, heating_h, cooling_h).await;
        assert_eq!(result.unwrap(), 1);
        Ok(())
    }
}
