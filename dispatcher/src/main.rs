use futures::TryStreamExt;
use pulsar::{
    message::{proto::command_subscribe::SubType, Payload},
    Consumer, DeserializeMessage, Pulsar, TokioExecutor,
};
use serde::{Deserialize, Serialize};
use anyhow::Result;

mod config;
mod data;

use data::DynamicMessage;

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
    println!("Using Pulsar at {}", config.mq.url);

    let pulsar: Pulsar<_> = Pulsar::builder(config.mq.url, TokioExecutor).build().await?;

    let mut consumer: Consumer<DynamicMessage, _> = pulsar
        .consumer()
        .with_topic("test")
        .with_consumer_name("test_consumer")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("test_subscription")
        .build()
        .await?;

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

        log::info!("got message {} {}", &data.type_name, &data.value.to_string());

        counter += 1;
        log::info!("got {} messages", counter);
    }

    Ok(())
}