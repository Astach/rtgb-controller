use crate::config::{ConsumerConfig, NatsConfig};

pub trait MessageDriverPort {
    fn process();
}
