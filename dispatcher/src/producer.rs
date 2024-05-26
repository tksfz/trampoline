use anyhow::{Context, Result};
use pulsar::{producer::SendFuture, Executor, MultiTopicProducer, Pulsar};

use crate::data::DynamicTaskMessage;

/// Producer for task messages that routes tasks based on their content
pub struct Producer<Exe: Executor> {
    producer: MultiTopicProducer<Exe>
}

impl<Exe: Executor> Producer<Exe> {
    pub fn new<E: Executor>(pulsar: &Pulsar<E>, name: &str) -> Producer<E> {
        let producer = pulsar
            .producer()
            .with_name(name)
            .build_multi_topic();
        Producer { producer }
    }

    pub async fn send(&mut self, msg: &DynamicTaskMessage) -> Result<SendFuture> {
        let topic = &msg.type_name;
        self.producer.send(topic, msg).await.context("sending task to topic failed")
    }
}