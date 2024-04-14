use pulsar::{producer, DeserializeMessage, Error as PulsarError, Payload, SerializeMessage};
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

/* SerializeMessage takes input as move by default to avoid copies:
     https://github.com/streamnative/pulsar-rs/pull/109

   Here, since serde_json::to_vec takes a reference anyway, it doesn't matter
   and we impl on the more natural reference type
 */
impl SerializeMessage for &DynamicTaskMessage {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload =
            serde_json::to_vec(input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

