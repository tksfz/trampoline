use pulsar::{DeserializeMessage, Payload};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct DynamicTaskMessage {
    #[serde(rename = "type")]
    pub type_name: String,
    // FUTURE: scheduling options go here
    pub task: Value,
}

impl DeserializeMessage for DynamicTaskMessage {
    type Output = Result<DynamicTaskMessage, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
