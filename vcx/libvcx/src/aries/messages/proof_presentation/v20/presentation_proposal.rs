use crate::error::prelude::*;
use crate::aries::messages::a2a::{A2AMessage, MessageId};
use crate::aries::messages::thread::Thread;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::aries::messages::attachment_format::{AttachmentFormats, AttachmentFormatTypes, AttachmentFormat};
use crate::aries::messages::attachment::{Attachments, AttachmentId};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationProposal {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub formats: AttachmentFormats,
    #[serde(rename = "proposals~attach")]
    pub proposals_attach: Attachments,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
}

impl PresentationProposal {
    pub fn create() -> Self {
        PresentationProposal::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_goal_code(mut self, goal_code: String) -> Self {
        self.goal_code = Some(goal_code);
        self
    }

    pub fn set_indy_proposals_attach(self, presentation_proposal: &str) -> VcxResult<PresentationProposal> {
        self.set_proposals_attach(presentation_proposal, AttachmentFormatTypes::IndyProofRequest)
    }

    pub fn set_proposals_attach(mut self, presentation_proposal: &str, format: AttachmentFormatTypes) -> VcxResult<PresentationProposal> {
        let id = AttachmentId::Other(MessageId::new().to_string());
        self.proposals_attach.add_base64_encoded_json_attachment(id.clone(), ::serde_json::Value::String(presentation_proposal.to_string()))?;
        self.formats.add(id, format);
        Ok(self)
    }

    pub fn set_thread(mut self, thread: Thread) -> Self {
        self.thread = Some(thread);
        self
    }

    pub fn set_thread_id(mut self, id: &str) -> Self {
        self.thread = Some(Thread::new().set_thid(id.to_string()));
        self
    }

    pub fn proposals_attach_content(&self) -> VcxResult<(&AttachmentFormat, String)> {
        let (attach_id, content) = self.proposals_attach.content()?;
        let format = self.formats.find(&attach_id)?;
        Ok((format, content))
    }
}

impl Default for PresentationProposal {
    fn default() -> PresentationProposal {
        PresentationProposal {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::PresentProof,
                version: MessageTypeVersion::V20,
                type_: A2AMessage::PROPOSE_PRESENTATION.to_string()
            },
            goal_code: Default::default(),
            comment: Default::default(),
            formats: Default::default(),
            thread: Default::default(),
            proposals_attach: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::proof_presentation::v20::presentation_request::tests::{thread, thread_id};

    fn _attachment() -> ::serde_json::Value {
        json!({"presentation": {}})
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _presentation_proposal() -> PresentationProposal {
        PresentationProposal {
            id: MessageId::id(),
            comment: Some(_comment()),
            thread: Some(thread()),
            ..PresentationProposal::default()
        }
    }

    #[test]
    fn test_presentation_proposal_build_works() {
        let presentation_proposal: PresentationProposal = PresentationProposal::default()
            .set_comment(_comment())
            .set_thread_id(&thread_id());

        assert_eq!(_presentation_proposal(), presentation_proposal);

        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/present-proof/2.0/propose-presentation","comment":"comment","formats":[],"proposals~attach":[],"~thread":{"received_orders":{},"sender_order":0,"thid":"testid"}}"#;
        assert_eq!(expected.to_string(), json!(presentation_proposal).to_string())
    }
}
