use crate::error::prelude::*;
use crate::v3::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::v3::messages::a2a::message_family::MessageTypeFamilies;
use crate::v3::messages::a2a::A2AMessage;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Forward {
    pub to: String,
    #[serde(rename = "msg")]
    pub msg: ::serde_json::Value,
    #[serde(rename = "@type")]
    pub type_: MessageType,
}

impl Forward {
    pub fn new(to: String, msg: Vec<u8>) -> VcxResult<Forward> {
        let msg = ::serde_json::from_slice(msg.as_slice())
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Could not parse JSON from bytes. Err: {:?}", err)))?;

        Ok(
            Forward {
                to,
                msg,
                ..Forward::default()
            }
        )
    }
}

impl Default for Forward {
    fn default() -> Forward {
        Forward {
            to: Default::default(),
            msg: Default::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Routing,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::FORWARD.to_string(),
            },
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::v3::messages::ack::tests::*;

    fn _to() -> String {
        String::from("GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL")
    }

    fn _msg() -> ::serde_json::Value {
       json!(_ack())
    }

    fn _forward() -> Forward {
        Forward {
            to: _to(),
            msg: _msg(),
            ..Forward::default()
        }
    }

    #[test]
    fn test_forward_build_works() {
        let message = ::serde_json::to_vec(&_ack()).unwrap();
        let forward: Forward = Forward::new(_to(), message).unwrap();

        assert_eq!(_forward(), forward);
        let expected = r#"{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/routing/1.0/forward","msg":{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/notification/1.0/ack","status":"FAIL","~thread":{"received_orders":{},"sender_order":0,"thid":"test_id"}},"to":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL"}"#;
        assert_eq!(expected, json!(forward).to_string());
    }
}