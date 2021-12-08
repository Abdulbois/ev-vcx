use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::ack::PleaseAck;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invite {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    pub goal_code: String,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>
}

impl Invite {
    pub fn create() -> Invite {
        Invite::default()
    }

    pub fn set_goal_code(mut self, goal_code: String) -> Invite {
        self.goal_code = goal_code;
        self
    }

    pub fn set_ack_on(mut self, ack_on: Option<Vec<String>>) -> Invite {
        if let Some(ack_on_) = ack_on {
            self.please_ack = Some(PleaseAck {
                on: Some(ack_on_)
            });
        }

        self
    }
}

please_ack!(Invite);

impl Default for Invite {
    fn default() -> Invite {
        Invite {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::InviteAction,
                version: MessageTypeVersion::V09,
                type_: A2AMessage::INVITE_FOR_ACTION.to_string()
            },
            goal_code: Default::default(),
            please_ack: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InviteActionData {
    pub goal_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_on: Option<Vec<String>>,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn _goal_code() -> String {
        String::from("automotive.inspect.tire")
    }

    pub fn _invite() -> Invite {
        Invite {
            id: MessageId::id(),
            goal_code: _goal_code(),
            please_ack: None,
            ..Invite::default()
        }
    }

    #[test]
    fn test_invite_build_works() {
        let invite: Invite = Invite::default()
            .set_goal_code(_goal_code());

        assert_eq!(_invite(), invite);

        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/invite-action/0.9/invite","goal_code":"automotive.inspect.tire"}"#;
        assert_eq!(expected, json!(invite).to_string());
    }
}