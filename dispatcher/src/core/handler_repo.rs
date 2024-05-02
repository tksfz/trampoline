use std::str::FromStr;

use reqwest::Url;

use crate::data::DynamicTaskMessage;
use crate::config::TaskHandler;

pub struct WorkerMatcher {
    matchers: Vec<Box<dyn Fn(&DynamicTaskMessage) -> Option<Url>>>,
}

impl WorkerMatcher {
    pub fn new(config: &[TaskHandler]) -> Result<WorkerMatcher, anyhow::Error> {
        // config.to_vec() is needed because Box<dyn Trait> is implicitly + 'static
        // and config.to_vec() does a deep clone that avoids requiring
        // &'static on config although that could also be fine
        let matchers = 
            config.to_vec().into_iter()
                .map(|c| 
                    // Of course, we want to parse proper Url's here, early
                    // so that startup fails if any of them fail to parse
                    Url::from_str(&c.endpoint.clone().unwrap()).map(|u| (c, u))
                )
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|(c, url)| { 
                Box::new(move |msg: &DynamicTaskMessage| -> Option<Url> {
                    if msg.type_name == c.task_selector.type_name {
                        Some(url.to_owned())
                    } else {
                        None
                    }
                }) as Box<_>
        });
        Ok(WorkerMatcher { matchers: Vec::from_iter(matchers) })
    }

    pub fn match_worker(&self, msg: &DynamicTaskMessage) -> Option<Url> {
        self.matchers.iter().find_map(|f| f(msg))
    }
}
