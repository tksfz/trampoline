use futures::TryStreamExt;
use pulsar::{
    message::{proto::command_subscribe::SubType, Payload},
    Consumer, DeserializeMessage, Pulsar, TokioExecutor,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;

mod config;
mod data;
mod core;

use data::DynamicMessage;
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

    let pulsar: Pulsar<_> = Pulsar::builder(config.mq.url, TokioExecutor).build().await?;

    let mut consumer: Consumer<DynamicMessage, _> = pulsar
        .consumer()
        .with_topic("test")
        .with_consumer_name("test_consumer")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("test_subscription")
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
                log::info!("got message {} {}, result: {} {}", &data.type_name, &data.value.to_string(), status, text);
            },
            None => {
                log::info!("could not find worker for {} {}", &data.type_name, &data.value.to_string())
            }
        };

        counter += 1;
        log::info!("got {} messages", counter);
    }

    Ok(())
}