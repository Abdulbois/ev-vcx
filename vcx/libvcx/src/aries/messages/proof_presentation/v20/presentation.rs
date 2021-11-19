use crate::error::prelude::*;
use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::attachment::{Attachments, AttachmentId};
use crate::aries::messages::ack::PleaseAck;
use crate::aries::messages::thread::Thread;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::aries::messages::attachment_format::{AttachmentFormatTypes, AttachmentFormats, AttachmentFormat};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Presentation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default = "default_as_true")]
    pub last_presentation: bool,
    pub formats: AttachmentFormats,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
}

impl Presentation {
    pub fn create() -> Self {
        Presentation::default()
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_goal_code(mut self, goal_code: Option<String>) -> Self {
        self.goal_code = goal_code;
        self
    }

    pub fn set_indy_presentations_attach(self, presentations: &str) -> VcxResult<Presentation> {
        self.set_presentations_attach(presentations, AttachmentFormatTypes::IndyProof)
    }

    pub fn set_presentations_attach(mut self, presentations: &str, format: AttachmentFormatTypes) -> VcxResult<Presentation> {
        let id = AttachmentId::Other(MessageId::new().to_string());
        self.presentations_attach.add_base64_encoded_json_attachment(id.clone(), ::serde_json::Value::String(presentations.to_string()))?;
        self.formats.add(id.clone(), format);
        Ok(self)
    }

    pub fn presentations_attach_content(&self) -> VcxResult<(&AttachmentFormat, String)> {
        let (attach_id, content) = self.presentations_attach.content()?;
        let format = self.formats.find(&attach_id)?;
        Ok((format, content))
    }
}

fn default_as_true() -> bool {
    true
}

please_ack!(Presentation);
threadlike!(Presentation);

impl Default for Presentation {
    fn default() -> Presentation {
        Presentation {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::PresentProof,
                version: MessageTypeVersion::V20,
                type_: A2AMessage::PRESENTATION.to_string()
            },
            goal_code: Default::default(),
            comment: Default::default(),
            last_presentation: true,
            formats: Default::default(),
            presentations_attach: Default::default(),
            thread: Default::default(),
            please_ack: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::proof_presentation::v20::presentation_request::tests::{thread, thread_id};

    fn _attachment() -> serde_json::Value {
        json!({"presentation": {}})
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _presentation() -> Presentation {
        let id = AttachmentId::Other(MessageId::new().to_string());

        let mut attachment = Attachments::new();
        let mut formats = AttachmentFormats::new();

        attachment.add_base64_encoded_json_attachment(id.clone(), _attachment()).unwrap();
        formats.add(id.clone(), AttachmentFormatTypes::IndyProof);

        Presentation {
            id: MessageId::id(),
            comment: Some(_comment()),
            presentations_attach: attachment,
            formats,
            thread: thread(),
            please_ack: Some(PleaseAck { on: None }),
            ..Presentation::default()
        }
    }

    #[test]
    fn test_presentation_build_works() {
        let presentation: Presentation = Presentation::default()
            .set_comment(Some(_comment()))
            .ask_for_ack()
            .set_thread_id(&thread_id())
            .set_indy_presentations_attach(&json!(_attachment()).to_string()).unwrap();

        assert_eq!(_presentation(), presentation);
        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/present-proof/2.0/presentation","comment":"comment","formats":[{"attach_id":"testid","format":"hlindy/proof@v2.0"}],"last_presentation":true,"presentations~attach":[{"@id":"testid","data":{"base64":"eyJwcmVzZW50YXRpb24iOnt9fQ=="},"mime-type":"application/json"}],"~please_ack":{},"~thread":{"received_orders":{},"sender_order":0,"thid":"testid"}}"#;
        assert_eq!(expected, json!(presentation).to_string());
    }

    #[test]
    fn test_presentation_build_works_for_reset_ack() {
        let mut presentation: Presentation = Presentation::default().ask_for_ack();
        assert!(presentation.please_ack.is_some());

        presentation = presentation.reset_ack();
        assert!(presentation.please_ack.is_none());
    }
}
