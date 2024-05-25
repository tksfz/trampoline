use axum::{
    extract::{Json, State}, routing::{get, post}, Router
};

use futures::TryStreamExt;
use pulsar::{
    message::proto::command_subscribe::SubType, Consumer, MultiTopicProducer, Pulsar, TokioExecutor
};
use reqwest::{Client, StatusCode};
use anyhow::{bail, Result};

mod config;
mod data;
mod core;

use data::DynamicTaskMessage;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use core::{Forwarder, HandleResult};
use core::HandlerRepo;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    producers: Arc<Mutex<MultiTopicProducer<TokioExecutor>>>
}

async fn submit_task(State(app_state): State<AppState>, Json(msg): Json<DynamicTaskMessage>) -> std::result::Result<Json<Value>, StatusCode> {
    let mut producer = app_state.producers.lock().await;
    producer.send("topic", &msg).await.map_err(|_| { StatusCode::INTERNAL_SERVER_ERROR })?;
    let result = json![
        {
            "successful": true
        }
    ];
    Ok(Json::from(result))
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

    let submit_producer = pulsar
        .producer()
        .with_name("trampoline-dispatcher-submitter")
        .build_multi_topic();

    let submit_producer_arc = Arc::new(Mutex::new(submit_producer));

    let state = AppState { producers: submit_producer_arc };
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/tasks/submit", post(submit_task))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    let mut consumer: Consumer<DynamicTaskMessage, _> = pulsar
        .consumer()
        .with_topics(config.mq.topics)
        .with_consumer_name("trampoline-dispatcher")
        .with_subscription_type(SubType::Exclusive)
        .with_subscription("trampoline-dispatch")
        .build()
        .await?;

    let mut producers = pulsar
        .producer()
        .with_name("trampoline-dispatcher-republish")
        .build_multi_topic();

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
                    let topic = &response_task.type_name;
                    producers.send(topic, response_task).await?;
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