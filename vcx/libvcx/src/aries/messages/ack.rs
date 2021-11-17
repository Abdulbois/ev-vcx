use crate::messages::thread::Thread;
use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ack {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    status: AckStatus,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AckStatus {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "PENDING")]
    Pending
}

impl Default for AckStatus {
    fn default() -> AckStatus {
        AckStatus::Ok
    }
}

impl Ack {
    pub fn create() -> Ack {
        Ack::default()
    }

    pub fn set_status(mut self, status: AckStatus) -> Ack {
        self.status = status;
        self
    }

    pub fn set_message_family(mut self, message_family: MessageTypeFamilies) -> Self {
        self.type_.family = message_family;
        self
    }
}

threadlike!(Ack);

impl Default for Ack {
    fn default() -> Ack {
        Ack {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Notification,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::ACK.to_string(),
            },
            status: Default::default(),
            thread: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PleaseAck {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<Vec<String>>
}

#[macro_export]
macro_rules! please_ack (($type:ident) => (
    impl $type {
        pub fn ask_for_ack(mut self) -> $type {
            self.please_ack = Some(PleaseAck { on: None });
            self
        }

        pub fn reset_ack(mut self) -> $type {
            self.please_ack = None;
            self
        }
    }
));

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::connection::response::tests::*;

    pub fn _ack() -> Ack {
        Ack {
            id: MessageId::id(),
            status: AckStatus::Fail,
            thread: _thread(),
            ..Ack::default()
        }
    }

    #[test]
    fn test_ack_build_works() {
        let ack: Ack = Ack::default()
            .set_status(AckStatus::Fail)
            .set_thread_id(&_thread_id());

        assert_eq!(_ack(), ack);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/notification/1.0/ack","status":"FAIL","~thread":{"received_orders":{},"sender_order":0,"thid":"test_id"}}"#;
        assert_eq!(expected, json!(ack).to_string());
    }
}