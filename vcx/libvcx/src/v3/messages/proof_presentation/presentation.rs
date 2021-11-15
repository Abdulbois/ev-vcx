use crate::v3::messages::a2a::{MessageId, A2AMessage};
use crate::v3::messages::attachment::{Attachments, AttachmentId};
use crate::v3::messages::ack::PleaseAck;
use crate::messages::thread::Thread;
use crate::messages::proofs::proof_message::ProofMessage;
use std::convert::TryInto;
use crate::v3::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::v3::messages::a2a::message_family::MessageTypeFamilies;

use crate::error::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Presentation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>
}

impl Presentation {
    pub fn create() -> Self {
        Presentation::default()
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_presentations_attach(mut self, presentations: String) -> VcxResult<Presentation> {
        self.presentations_attach.add_base64_encoded_json_attachment(AttachmentId::Presentation,::serde_json::Value::String(presentations))?;
        Ok(self)
    }
}

please_ack!(Presentation);
threadlike!(Presentation);

impl Default for Presentation {
    fn default() -> Presentation {
        Presentation {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::PresentProof,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::PRESENTATION.to_string()
            },
            comment: Default::default(),
            presentations_attach: Default::default(),
            thread: Default::default(),
            please_ack: Default::default(),
        }
    }
}

impl TryInto<Presentation> for ProofMessage {
    type Error = VcxError;

    fn try_into(self) -> Result<Presentation, Self::Error> {
        let presentation = Presentation::create()
            .set_presentations_attach(self.libindy_proof)?
            .ask_for_ack();

        Ok(presentation)
    }
}

impl TryInto<ProofMessage> for Presentation {
    type Error = VcxError;

    fn try_into(self) -> Result<ProofMessage, Self::Error> {
        let mut proof = ProofMessage::new();
        proof.libindy_proof = self.presentations_attach.content().unwrap();
        Ok(proof)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::v3::messages::proof_presentation::presentation_request::tests::{thread, thread_id};

    fn _attachment() -> ::serde_json::Value {
        json!({"presentation": {}})
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _presentation() -> Presentation {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::Presentation,_attachment()).unwrap();

        Presentation {
            id: MessageId::id(),
            comment: Some(_comment()),
            presentations_attach: attachment,
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
            .set_presentations_attach(_attachment().to_string()).unwrap();

        assert_eq!(_presentation(), presentation);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation","comment":"comment","presentations~attach":[{"@id":"libindy-presentation-0","data":{"base64":"eyJwcmVzZW50YXRpb24iOnt9fQ=="},"mime-type":"application/json"}],"~please_ack":{},"~thread":{"received_orders":{},"sender_order":0,"thid":"testid"}}"#;
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
