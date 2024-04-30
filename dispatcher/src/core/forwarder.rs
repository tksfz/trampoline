use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use crate::data::DynamicTaskMessage;
use crate::core::worker_matcher::WorkerMatcher;

pub enum HandleResult {
    /// Successful processing that should continue with the tasks in the parsed WorkerResponse
    Continue { status: StatusCode, response: WorkerResponse },

    /// Successful processing with un unparseable response
    ContinueUnparseable { status: StatusCode, text: String },
}

pub struct Forwarder {
    client: Client,
    worker_matcher: WorkerMatcher,
}

#[derive(Deserialize, Debug)]
pub struct WorkerResponse {
    /// responses can contain multiple tasks, of varying types
    pub tasks: Vec<DynamicTaskMessage>,
}

impl Forwarder {
    pub fn new(client: Client, worker_matcher: WorkerMatcher) -> Forwarder {
        Forwarder { client, worker_matcher }
    }

    pub async fn process(&self, msg: &DynamicTaskMessage) -> Result<Option<HandleResult>> {
        // Map TypedMessage to some task schema that we recognize
    
        // validate the message against schema. this should maybe happen in the caller.
    
        // Map msg to the worker we should use
        let endpoint = self.worker_matcher.match_worker(&msg);

        // Make the HTTP call    
        let result = match endpoint {
            Some(url) => {
                let body = serde_json::to_string(&msg.task)?;
                let req = self.client
                    .post(url.clone())
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
                
                log::info!("sent message {} {}, worker {}, received message {} {}", &msg.type_name, &msg.task, url, status, &text);
                Some(result)
            },
            None => {
                None
            }
        };

        // Handle the result
    
        // Republish (or return the republish)
        Ok(result)
    }
}
