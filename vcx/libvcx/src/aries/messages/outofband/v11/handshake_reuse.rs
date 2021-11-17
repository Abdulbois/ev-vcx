use crate::messages::thread::Thread;
use crate::aries::messages::a2a::{A2AMessage, MessageId};
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HandshakeReuse {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

impl HandshakeReuse {
    pub fn create() -> HandshakeReuse {
        HandshakeReuse::default()
    }
}

threadlike!(HandshakeReuse);

impl Default for HandshakeReuse {
    fn default() -> HandshakeReuse {
        let id = MessageId::default();
        HandshakeReuse {
            id: id.clone(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::Outofband,
                version: MessageTypeVersion::V11,
                type_: A2AMessage::OUTOFBAND_HANDSHAKE_REUSE.to_string()
            },
            thread: Thread {
                thid: Some(id.to_string()),
                pthid: Default::default(),
                sender_order: Default::default(),
                received_orders: Default::default()
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::connection::response::tests::*;

    pub fn _handshake_reuse() -> HandshakeReuse {
        HandshakeReuse {
            id: MessageId::id(),
            thread: _thread(),
            ..HandshakeReuse::default()
        }
    }

    #[test]
    fn test_handshake_reuse_build_works() {
        let handshake_reuse = HandshakeReuse::default()
            .set_thread(_thread());

        assert_eq!(_handshake_reuse(), handshake_reuse);

        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/out-of-band/1.1/handshake-reuse","~thread":{"received_orders":{},"sender_order":0,"thid":"test_id"}}"#;
        assert_eq!(expected, json!(handshake_reuse).to_string());
    }
}