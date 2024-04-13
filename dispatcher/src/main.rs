use futures::TryStreamExt;
use pulsar::{
    message::{proto::command_subscribe::SubType, Payload},
    Consumer, DeserializeMessage, Pulsar, TokioExecutor,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{bail, Result};

mod config;
mod data;
mod core;

use data::DynamicTaskMessage;
use core::WorkerMatcher;
use core::{Forwarder, ForwardResult};

#[derive(Serialize, Deserialize)]
struct TestData {
    data: String,
}

impl DeserializeMessage for TestData {
    type Output = Result<TestData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = config::Config::read()?;
    if config.mq.topics.is_empty() {
        // This could be relaxed in the future if we support dynamic reconfiguration
        bail!("at least one topic must be specified in config `mq.topics`");
    }

    let pulsar: Pulsar<_> = Pulsar::builder(config.mq.url, TokioExecutor).build().await?;

    let mut consumer: Consumer<DynamicTaskMessage, _> = pulsar
        .consumer()
        .with_topics(config.mq.topics)
        .with_consumer_name("trampoline-dispatcher")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("trampoline-dispatch")
        .build()
        .await?;

    let client = Client::new();
    let worker_matcher = WorkerMatcher::new(&config.workers)?;
    let processor = Forwarder::new(client, worker_matcher);

    let mut counter = 0usize;
    while let Some(msg) = consumer.try_next().await? {
        consumer.ack(&msg).await?;
        let data = match msg.deserialize() {
            Ok(data) => data,
            Err(e) => {
                log::error!("could not deserialize message: {:?}", e);
                break;
            }
        };

        let result = processor.process(&data).await?;
        match result {
            Some(ForwardResult::Continue { status, text }) => {
                log::info!("got message {} {}, result: {} {}", &data.type_name, &data.task.to_string(), status, text);

            },
            None => {
                log::info!("could not find worker for {} {}", &data.type_name, &data.task.to_string())
            }
        };

        counter += 1;
        log::info!("got {} messages", counter);
    }

    Ok(())
}