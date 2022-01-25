#[macro_export]
macro_rules! deserialize_v1_v2_message(($type:ident, $type_v1:ident, $type_v2:ident) => (
    impl<'de> Deserialize<'de> for $type {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            let value = serde_json::Value::deserialize(deserializer).map_err(de::Error::custom)?;

            let message_type: MessageType = serde_json::from_value(value["@type"].clone()).map_err(de::Error::custom)?;
            match message_type.version {
                MessageTypeVersion::V10 => {
                    $type_v1::deserialize(value)
                        .map(|message| $type::V1(message))
                        .map_err(de::Error::custom)
                }
                MessageTypeVersion::V20 => {
                    $type_v2::deserialize(value)
                        .map(|message| $type::V2(message))
                        .map_err(de::Error::custom)
                }
                _ => {
                    Err(de::Error::custom(format!("Unsupported message version: {:?}", message_type.version)))
                }
            }
        }
    }
));

#[macro_use]
pub mod thread;
#[macro_use]
pub mod a2a;
#[macro_use]
pub mod ack;
#[macro_use]
pub mod transport;
pub mod connection;
pub mod error;
pub mod forward;
pub mod attachment;
pub mod attachment_format;
pub mod mime_type;
pub mod status;
pub mod issuance;
pub mod proof_presentation;
pub mod discovery;
pub mod trust_ping;
pub mod basic_message;
pub mod localization;
pub mod outofband;
pub mod questionanswer;
pub mod committedanswer;
pub mod invite_action;
pub mod message_with_attachment;
pub mod message_with_thread;