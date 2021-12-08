use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::attachment::{Attachments, AttachmentId};
use crate::aries::messages::connection::service::Service;
use crate::error::prelude::*;
use crate::aries::messages::thread::Thread;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;

pub use crate::legacy::messages::proof_presentation::proof_request::{ProofRequestMessage};
pub use crate::utils::libindy::anoncreds::proof_request::ProofRequest;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationRequest {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Attachments,
    #[serde(rename = "~service")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Service>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
}

impl PresentationRequest {
    pub fn create() -> Self {
        PresentationRequest::default()
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
        self
    }

    pub fn set_opt_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_request_presentations_attach(mut self, request_presentations: &ProofRequest) -> VcxResult<PresentationRequest> {
        self.request_presentations_attach.add_base64_encoded_json_attachment(AttachmentId::PresentationRequest, json!(request_presentations))?;
        Ok(self)
    }

    pub fn set_service(mut self, service: Option<Service>) -> Self {
        self.service = service;
        self

    }

    pub fn set_thread_id(mut self, id: &str) -> Self {
        self.thread = Some(Thread::new().set_thid(id.to_string()));
        self
    }

    pub fn set_thread(mut self, thread: Thread) -> Self {
        self.thread = Some(thread);
        self
    }

    pub fn to_json(&self) -> VcxResult<String> {
        serde_json::to_string(self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize PresentationRequest: {}", err)))
    }
}

impl Default for PresentationRequest {
    fn default() -> PresentationRequest {
        PresentationRequest {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::PresentProof,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::REQUEST_PRESENTATION.to_string()
            },
            comment: Default::default(),
            request_presentations_attach: Default::default(),
            service: Default::default(),
            thread: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::thread::Thread;
    use crate::aries::messages::connection::service::tests::_service;

    pub fn _presentation_request_data() -> ProofRequest {
        ProofRequest::default()
            .set_requested_attributes(json!([{"name": "name"}]).to_string()).unwrap()
    }

    fn _attachment() -> Attachments {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::PresentationRequest,json!(_presentation_request_data())).unwrap();
        attachment
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn thread_id() -> String {
        _presentation_request().id.0
    }

    pub fn thread() -> Thread {
        Thread::new().set_thid(_presentation_request().id.0)
    }

    pub fn _presentation_request() -> PresentationRequest {
        PresentationRequest {
            id: MessageId::id(),
            comment: Some(_comment()),
            request_presentations_attach: _attachment(),
            service: None,
            thread: None,
            ..PresentationRequest::default()
        }
    }

    pub fn _presentation_request_with_service() -> PresentationRequest {
        PresentationRequest {
            id: MessageId::id(),
            comment: Some(_comment()),
            request_presentations_attach: _attachment(),
            service: Some(_service()),
            thread: None,
            ..PresentationRequest::default()
        }
    }

    #[test]
    fn test_presentation_request_build_works() {
        let presentation_request: PresentationRequest = PresentationRequest::default()
            .set_comment(_comment())
            .set_request_presentations_attach(&_presentation_request_data()).unwrap();

        assert_eq!(_presentation_request(), presentation_request);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/request-presentation","comment":"comment","request_presentations~attach":[{"@id":"libindy-request-presentation-0","data":{"base64":"eyJuYW1lIjoiIiwibm9uX3Jldm9rZWQiOm51bGwsIm5vbmNlIjoiIiwicmVxdWVzdGVkX2F0dHJpYnV0ZXMiOnsiYXR0cmlidXRlXzAiOnsibmFtZSI6Im5hbWUifX0sInJlcXVlc3RlZF9wcmVkaWNhdGVzIjp7fSwidmVyIjpudWxsLCJ2ZXJzaW9uIjoiMS4wIn0="},"mime-type":"application/json"}]}"#;
        assert_eq!(expected, json!(presentation_request).to_string());
    }

    #[test]
    fn test_presentation_request_build_works_for_service() {
        let presentation_request: PresentationRequest = PresentationRequest::default()
            .set_comment(_comment())
            .set_service(Some(_service()))
            .set_request_presentations_attach(&_presentation_request_data()).unwrap();

        assert_eq!(_presentation_request_with_service(), presentation_request);
    }
}
