use std::sync::Arc;

use axum::{extract::State, routing::get, routing::post, Json, Router};
use pulsar::TokioExecutor;
use reqwest::StatusCode;
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::{data::DynamicTaskMessage, producer::Producer};

/// Optional HTTP endpoints served directly from the Dispatcher

#[derive(Clone)]
struct AppState {
    producer: Arc<Mutex<Producer<TokioExecutor>>>
}

pub struct Serve {
    submit_producer: Arc<Mutex<Producer<TokioExecutor>>>
}

impl Serve {
    pub fn new(submit_producer: Producer<TokioExecutor>) -> Serve {
        let submit_producer = Arc::new(Mutex::new(submit_producer));
        Serve { submit_producer }
    }

    pub async fn start(&self) {
        let state = AppState { producer: self.submit_producer.clone() };
        // build our application with a single route
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/tasks/submit", post(Self::submit_task))
            .with_state(state);

        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    async fn submit_task(State(app_state): State<AppState>, Json(msg): Json<DynamicTaskMessage>) -> std::result::Result<Json<Value>, StatusCode> {
        let mut producer = app_state.producer.lock().await;
        producer.send(&msg).await.map_err(|_| { StatusCode::INTERNAL_SERVER_ERROR })?;
        let result = json![
            {
                "successful": true
            }
        ];
        Ok(Json::from(result))
    }
    
}