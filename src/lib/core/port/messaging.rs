pub trait MessageDriverPort {
    async fn connect();
    async fn subscribe();
    async fn process();
}
