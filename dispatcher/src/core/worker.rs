use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, Url};

use crate::core::{WorkerResponse, HandleResult};

use super::handler::Handler;

pub struct Worker {
    pub endpoint: Url
}

#[async_trait]
impl Handler for Worker {
    async fn handle(&self, client: &Client, task: &crate::data::DynamicTaskMessage) -> Result<super::HandleResult> {
        let body = serde_json::to_string(&task.task)?;
        let req = client
            .post(self.endpoint.clone())
            .header("Content-Type", "application/json")
            .body(body);
        let res = req.send().await?;
        let status = res.status();
        let text = res.text().await?;
        let parsed_response: Result<WorkerResponse, serde_json::Error> = serde_json::from_str(&text);
        let result = match parsed_response {
            Ok(worker_response) => HandleResult::Continue { status, response: worker_response },
            Err(_) =>
                // TODO: worker declaration should include a flag that indicates strict response handling, meaning an unparseable response should go to a DLQ
                HandleResult::ContinueUnparseable { status, text: text.clone() }
        };
        
        log::info!("sent message {} {}, worker {}, received message {} {}", &task.type_name, &task.task, self.endpoint, status, &text);
        Ok(result)
    }
}