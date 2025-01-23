pub trait CommandDrivenPort {
    fn fetch(&self, command_id: Uuid);
    fn insert(&self, message: Message);
    fn update(&self, command_id: Uuid, status: String);
    fn delete(&self, command_id: Uuid);
}
