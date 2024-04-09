use std::fs;
use serde::Deserialize;
use toml;
use anyhow::{Context, Result};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub mq: Mq,

    // FUTURE:
    // topics
    // consumption config
    // (basic config such as round-robin, shared)
    // (advanced config such as consumer tuning settings)
    // tasks -- schemas, or known tasks at least and
    // task read selectors
    // task worker selectors
    // task publish rules

    pub workers: Vec<TaskWorker>,
}

#[derive(Deserialize, Clone)]
pub struct Mq {
    pub url: String,
    pub topics: Vec<String>
}

#[derive(Deserialize, Clone)]
pub struct TaskWorker {
    pub task_selector: Select,
    pub endpoint: String,
}

#[derive(Deserialize, Clone)]
pub struct Select {
    #[serde(rename = "type")]
    pub type_name: String,
}

impl Config {
    pub fn read() -> Result<Config> {
        let filename = "dispatcher.toml";
        let toml_str = fs::read_to_string(filename).context(format!("Unable to open config file `{}`", filename))?;
        let config: Config = toml::from_str(&toml_str).context(format!("Unable to parse TOML from `{}`", filename))?;
        Ok(config)
    }
}