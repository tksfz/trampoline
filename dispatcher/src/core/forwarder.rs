use anyhow::Result;
use reqwest::Client;
use crate::data::DynamicTaskMessage;
use crate::core::handler_repo::HandlerRepo;

use super::handler::HandleResult;

pub struct Forwarder {
    client: Client,
    handlers: HandlerRepo,
}

impl Forwarder {
    pub fn new(client: Client, handlers: HandlerRepo) -> Forwarder {
        Forwarder { client, handlers }
    }

    pub async fn process(&self, msg: &DynamicTaskMessage) -> Result<Option<HandleResult>> {
        // Map TypedMessage to some task schema that we recognize
    
        // validate the message against schema. this should maybe happen in the caller.
    
        // Map msg to the worker we should use
        let endpoint = self.handlers.match_handler(&msg);

        // Make the HTTP call    
        let result = match endpoint {
            Some(handler) => {
                let result = handler.handle(&self.client, &msg).await?;
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
