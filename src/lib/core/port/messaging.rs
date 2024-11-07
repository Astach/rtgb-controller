use crate::config::{ConsumerConfig, NatsConfig};

pub trait MessageDriverPort {
    fn connect(nats_config: NatsConfig);
    fn subscribe(subject_config: Vec<ConsumerConfig>);
    fn process();
}
