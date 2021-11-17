use crate::messages::thread::Thread;
use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::a2a::protocol_registry::Actors;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Disclose {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    pub protocols: Vec<ProtocolDescriptor>,
    #[serde(rename = "~thread")]
    pub thread: Thread
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProtocolDescriptor {
    pub pid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Actors>>
}

impl Disclose {
    pub fn create() -> Disclose {
        Disclose::default()
    }

    pub fn set_protocols(mut self, protocols: Vec<ProtocolDescriptor>) -> Self {
        self.protocols = protocols;
        self
    }

    pub fn add_protocol(&mut self, protocol: ProtocolDescriptor) {
        self.protocols.push(protocol);
    }

    pub fn set_thread_id(mut self, id: String) -> Self {
        self.thread.thid = Some(id);
        self
    }
}

impl Default for Disclose {
    fn default() -> Disclose {
        Disclose {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::DiscoveryFeatures,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::DISCLOSE.to_string()
            },
            protocols: Default::default(),
            thread: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::connection::response::tests::*;

    fn _protocol_descriptor() -> ProtocolDescriptor {
        ProtocolDescriptor { pid: String::from("did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/"), roles: None }
    }

    pub fn _disclose() -> Disclose {
        Disclose {
            id: MessageId::id(),
            protocols: vec![_protocol_descriptor()],
            thread: _thread(),
            ..Disclose::default()
        }
    }

    #[test]
    fn test_disclose_build_works() {
        let mut disclose: Disclose = Disclose::default()
            .set_thread_id(_thread_id());

        disclose.add_protocol(_protocol_descriptor());

        assert_eq!(_disclose(), disclose);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/discover-features/1.0/disclose","protocols":[{"pid":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/"}],"~thread":{"received_orders":{},"sender_order":0,"thid":"test_id"}}"#;
        assert_eq!(expected, json!(disclose).to_string());
    }
}
