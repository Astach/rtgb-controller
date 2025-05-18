use crate::domain::command::Command;

pub trait PublisherDrivenPort {
    async fn publish(&self, command: &Command) -> anyhow::Result<()>;
}
