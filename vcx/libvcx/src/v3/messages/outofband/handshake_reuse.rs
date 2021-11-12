use crate::messages::thread::Thread;
use crate::v3::messages::a2a::{A2AMessage, MessageId};
use crate::v3::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::v3::messages::a2a::message_family::MessageTypeFamilies;

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
        HandshakeReuse {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Outofband,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::OUTOFBAND_HANDSHAKE_REUSE.to_string()
            },
            thread: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::v3::messages::connection::response::tests::*;

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

        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/out-of-band/1.0/handshake-reuse","~thread":{"received_orders":{},"sender_order":0,"thid":"test_id"}}"#;
        assert_eq!(expected, json!(handshake_reuse).to_string());
    }
}