use anyhow::Result;
use reqwest::{Client, StatusCode};
use crate::data::DynamicMessage;
use crate::core::worker_matcher::WorkerMatcher;
use serde_json;

pub enum ForwardResult {
    Continue { status: StatusCode, text: String }
}

pub struct Forwarder {
    client: Client,
    worker_matcher: WorkerMatcher,
}

impl Forwarder {
    pub fn new(client: Client, worker_matcher: WorkerMatcher) -> Forwarder {
        Forwarder { client, worker_matcher }
    }

    pub async fn process(&self, msg: &DynamicMessage) -> Result<Option<ForwardResult>> {
        // Map TypedMessage to some task schema that we recognize
    
        // validate the message against schema. this should maybe happen in the caller.
    
        // Map msg to the worker we should use
        let endpoint = self.worker_matcher.match_worker(&msg);

        // Make the HTTP call    
        let result = match endpoint {
            Some(url) => {
                let body = serde_json::to_string(&msg.value)?;
                let req = self.client
                    .post(url.clone())
                    .body(body);
                let res = req.send().await?;

                let result = ForwardResult::Continue { status: res.status(), text: res.text().await? };
                let ForwardResult::Continue { status, ref text } = result;
                log::info!("sent message {} {}, worker {}, received message {} {}", &msg.type_name, &msg.value, url, status, &text);
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
