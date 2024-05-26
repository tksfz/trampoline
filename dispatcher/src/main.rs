use futures::TryStreamExt;
use pulsar::{
    message::proto::command_subscribe::SubType, Consumer, Pulsar, TokioExecutor
};
use reqwest::Client;
use anyhow::{bail, Result};

mod config;
mod data;
mod core;
mod producer;
mod serve;

use data::DynamicTaskMessage;
use core::{Forwarder, HandleResult};
use core::HandlerRepo;

use producer::Producer;
use serve::Serve;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = config::Config::read()?;
    if config.mq.topics.is_empty() {
        // This could be relaxed in the future if we support dynamic reconfiguration
        bail!("at least one topic must be specified in config `mq.topics`");
    }

    let pulsar: Pulsar<_> = Pulsar::builder(config.mq.url, TokioExecutor).build().await?;

    let submit_producer = Producer::<TokioExecutor>::new(&pulsar, "trampoline-dispatcher-submitter");

    let serve = Serve::new(submit_producer);
    serve.spawn_start().await;

    let mut consumer: Consumer<DynamicTaskMessage, _> = pulsar
        .consumer()
        .with_topics(config.mq.topics)
        .with_consumer_name("trampoline-dispatcher")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("trampoline-dispatch")
        .build()
        .await?;

    let mut producer = Producer::<TokioExecutor>::new(&pulsar, "trampoline-disapatcher-republish");

    let client = Client::new();
    let handlers = HandlerRepo::new(&config.handlers)?;
    let processor = Forwarder::new(client, handlers);

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
            Some(HandleResult::Continue { status, response }) => {
                for response_task in &response.tasks {
                    producer.send(response_task).await?;
                }
                let message_id = format!("{}:{}:{}:{}", &msg.topic, msg.message_id.id.ledger_id, msg.message_id.id.entry_id, msg.message_id.id.partition.unwrap_or(-1));
                let plural = if response.tasks.len() == 1 { "task" } else { "tasks" };
                log::info!("messageId:<{}> task:<{}> status:<{}> result:<Continue:{} new {}>", message_id, &data.type_name, status, response.tasks.len(), plural);
            },
            Some(HandleResult::ContinueUnparseable { status, text }) => {
                log::info!("got message {} {}, result: {} {}", &data.type_name, &data.task, status, text);

            },
            None => {
                log::info!("could not find worker for {} {}", &data.type_name, &data.task)
            }
        };

        counter += 1;
    }
    log::info!("got {} messages", counter);
    Ok(())
}