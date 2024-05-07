use std::collections::HashMap;
use std::str::FromStr;

use reqwest::Url;
use rune::Source;

use crate::data::DynamicTaskMessage;
use crate::config::TaskHandler;

use super::handler::Handler;
use super::rune::RuneScript;
use super::worker::Worker;

#[derive(PartialEq, Eq, Hash, Clone)]
enum HandlerDef {
    Endpoint(Url),
    Pipeline(String),
}

pub struct HandlerRepo {
    handlers: HashMap<HandlerDef, Box<dyn Handler>>,
    matchers: Vec<Box<dyn Fn(&DynamicTaskMessage) -> Option<HandlerDef>>>,
}

impl HandlerRepo {
    pub fn new(config: &[TaskHandler]) -> Result<HandlerRepo, anyhow::Error> {
        // config.to_vec() is needed because Box<dyn Trait> is implicitly + 'static
        // and config.to_vec() does a deep clone that avoids requiring
        // &'static on config although that could also be fine
        let handler_defs = config.to_vec().into_iter()
            .map(|c| 
                match (&c.endpoint, &c.pipeline) {
                    (Some(endpoint), _) => {
                        // We parse proper Url's here, early
                        // so that startup fails if any of them fail to parse
                        let url = Url::from_str(&endpoint)?;
                        Ok((c, HandlerDef::Endpoint(url)))
                    },
                    (_, Some(pipeline)) => {
                        Ok((c.clone(), HandlerDef::Pipeline(pipeline.clone())))
                    }
                    (None, None) => {
                        Err(anyhow::Error::msg("no endpoint or pipeline specified"))
                    }
                } 
            )
            .collect::<Result<Vec<_>, _>>()?;
        let handlers = handler_defs.iter().map(|(_, def)| {
            match def {
                HandlerDef::Endpoint(url) => 
                    Ok((def.clone(), Box::new(Worker { endpoint: url.clone() }) as Box<dyn Handler>)),
                HandlerDef::Pipeline(pipeline) => {
                    // TODO: this should actually load and validate the pipeline
                    let source = Source::from_path(pipeline)?;
                    Ok((def.clone(), Box::new(RuneScript::new(source)?) as Box<dyn Handler>))
                }
            }
        }).collect::<Result<HashMap<HandlerDef, Box<dyn Handler>>, anyhow::Error>>()?;
        let matchers = handler_defs
            .into_iter()
            .map(|(c, handler_key)| { 
                Box::new(move |msg: &DynamicTaskMessage| -> Option<HandlerDef> {
                    if msg.type_name == c.task_selector.type_name {
                        Some(handler_key.clone())
                    } else {
                        None
                    }
                }) as Box<_>
            });
        Ok(HandlerRepo { handlers, matchers: Vec::from_iter(matchers) })
    }

    pub fn match_handler(&self, msg: &DynamicTaskMessage) -> Option<&Box<dyn Handler>> {
        self.matchers.iter()
            .find_map(|f| f(msg))
            .and_then(|key| self.handlers.get(&key))
    }
}

