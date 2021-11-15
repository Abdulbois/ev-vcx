use crate::messages::thread::Thread;
use crate::v3::messages::a2a::{A2AMessage, MessageId};
use crate::v3::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::v3::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HandshakeReuseAccepted {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

impl HandshakeReuseAccepted {
    pub fn create() -> HandshakeReuseAccepted {
        HandshakeReuseAccepted::default()
    }
}

threadlike!(HandshakeReuseAccepted);

impl Default for HandshakeReuseAccepted {
    fn default() -> HandshakeReuseAccepted {
        HandshakeReuseAccepted {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Outofband,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::OUTOFBAND_HANDSHAKE_REUSE_ACCEPTED.to_string()
            },
            thread: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::v3::messages::connection::response::tests::*;

    pub fn _handshake_reuse_accepted() -> HandshakeReuseAccepted {
        HandshakeReuseAccepted {
            id: MessageId::id(),
            thread: _thread(),
            ..HandshakeReuseAccepted::default()
        }
    }

    #[test]
    fn test_handshake_reuse_accepted_build_works() {
        let handshake_reuse_accepted = HandshakeReuseAccepted::default()
            .set_thread(_thread());

        assert_eq!(_handshake_reuse_accepted(), handshake_reuse_accepted);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/out-of-band/1.0/handshake-reuse-accepted","~thread":{"received_orders":{},"sender_order":0,"thid":"test_id"}}"#;
        assert_eq!(expected, json!(handshake_reuse_accepted).to_string());
    }
}