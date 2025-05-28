#[cfg_attr(test, mockall::automock)]
pub trait PublisherDrivenPort {
    fn publish(&self, action: HardwareAction) -> impl Future<Output = anyhow::Result<()>>;
}
#[derive(PartialEq, Debug)]
pub enum HardwareAction {
    START(String),
    STOP(String),
}
impl HardwareAction {
    pub fn get_hardware_id(&self) -> String {
        match &self {
            HardwareAction::START(id) => id.into(),
            HardwareAction::STOP(id) => id.into(),
        }
    }
}
