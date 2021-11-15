use crate::messages::thread::Thread;
use crate::v3::messages::a2a::{MessageId, A2AMessage};
use crate::v3::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::v3::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ping {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(default)]
    pub response_requested: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
}

impl Ping {
    pub fn create() -> Ping {
        Ping::default()
    }

    pub fn set_thread_id(mut self, id: String) -> Self {
        self.thread = Some(Thread::new().set_thid(id));
        self
    }

    pub fn set_thread(mut self, thread: Thread) -> Self {
        self.thread = Some(thread);
        self
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Ping {
        self.comment = comment;
        self
    }

    pub fn request_response(mut self) -> Ping {
        self.response_requested = true;
        self
    }
}

impl Default for Ping {
    fn default() -> Ping {
        Ping {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::TrustPing,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::PING.to_string()
            },
            response_requested: Default::default(),
            comment: Default::default(),
            thread: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::v3::messages::connection::response::tests::*;

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _ping() -> Ping {
        Ping {
            id: MessageId::id(),
            response_requested: false,
            thread: Some(_thread()),
            comment: Some(_comment()),
            ..Ping::default()
        }
    }

    #[test]
    fn test_ping_build_works() {
        let ping: Ping = Ping::default()
            .set_comment(Some(_comment()))
            .set_thread_id(_thread_id());

        assert_eq!(_ping(), ping);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/trust_ping/1.0/ping","comment":"comment","response_requested":false,"~thread":{"received_orders":{},"sender_order":0,"thid":"test_id"}}"#;
        assert_eq!(expected, json!(ping).to_string());
    }
}