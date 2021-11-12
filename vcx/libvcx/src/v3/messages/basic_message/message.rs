use crate::v3::messages::a2a::{MessageId, A2AMessage};
use crate::v3::messages::localization::Localization;
use crate::v3::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::v3::messages::a2a::message_family::MessageTypeFamilies;
use chrono::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BasicMessage {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    pub sent_time: String,
    pub content: String,
    #[serde(rename = "~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l10n: Option<Localization>,
}

impl BasicMessage {
    pub fn create() -> BasicMessage {
        BasicMessage::default()
    }

    pub fn set_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn set_time(mut self) -> Self {
        self.sent_time = format!("{:?}", Utc::now());
        self
    }

    pub fn set_default_localization(mut self) -> Self {
        self.l10n = Some(Localization::default());
        self
    }
}

impl Default for BasicMessage {
    fn default() -> BasicMessage {
        BasicMessage {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Basicmessage,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::BASIC_MESSAGE.to_string()
            },
            sent_time: Default::default(),
            content: Default::default(),
            l10n: Default::default()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn _content() -> String {
        String::from("Your hovercraft is full of eels.")
    }

    #[test]
    fn test_basic_message_build_works() {
        let basic_message: BasicMessage = BasicMessage::default()
            .set_content(_content())
            .set_default_localization();
        assert_eq!(_content(), basic_message.content);

        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message","content":"Your hovercraft is full of eels.","sent_time":"","~l10n":{"locale":"en"}}"#;
        assert_eq!(expected, json!(basic_message).to_string())
    }
}
