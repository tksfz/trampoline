use anyhow::Result;
use reqwest::{Client, StatusCode};
use async_trait::async_trait;
use serde::Deserialize;

use crate::data::DynamicTaskMessage;

pub enum HandleResult {
    /// Successful processing that should continue with the tasks in the parsed WorkerResponse
    Continue { status: StatusCode, response: WorkerResponse },

    /// Successful processing with un unparseable response
    ContinueUnparseable { status: StatusCode, text: String },
}

#[derive(Deserialize, Debug)]
pub struct WorkerResponse {
    /// responses can contain multiple tasks, of varying types
    pub tasks: Vec<DynamicTaskMessage>,
}

#[async_trait]
pub trait Handler {
    async fn handle(&self, client: &Client, task: &DynamicTaskMessage) -> Result<HandleResult>;
}