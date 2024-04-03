use pulsar::{DeserializeMessage, Payload};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct DynamicMessage {
    #[serde(rename = "type")]
    pub type_name: String,
    pub value: Value,
}

impl DeserializeMessage for DynamicMessage {
    type Output = Result<DynamicMessage, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
