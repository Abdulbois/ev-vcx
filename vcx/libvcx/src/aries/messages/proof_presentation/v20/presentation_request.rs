use crate::error::prelude::*;
use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::attachment::{Attachments, AttachmentId};
use crate::aries::messages::connection::service::Service;
use crate::aries::messages::thread::Thread;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::aries::messages::attachment_format::{AttachmentFormats, AttachmentFormatTypes, AttachmentFormat};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationRequest {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default)]
    pub will_confirm: bool,
    #[serde(default)]
    pub present_multiple: bool,
    pub formats: AttachmentFormats,
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

    pub fn set_indy_request_presentations_attach(self, request_presentations: &str) -> VcxResult<PresentationRequest> {
        self.set_request_presentations_attach(request_presentations, AttachmentFormatTypes::IndyProofRequest)
    }

    pub fn set_request_presentations_attach(mut self, request_presentations: &str, format: AttachmentFormatTypes) -> VcxResult<PresentationRequest> {
        let id = AttachmentId::Other(MessageId::new().to_string());
        self.request_presentations_attach.add_base64_encoded_json_attachment(id.clone(), ::serde_json::Value::String(request_presentations.to_string()))?;
        self.formats.add(id, format);
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

    pub fn request_presentations_attach_content(&self) -> VcxResult<(&AttachmentFormat, String)> {
        let (attach_id, content) = self.request_presentations_attach.content()?;
        let format = self.formats.find(&attach_id)?;
        Ok((format, content))
    }
}

impl Default for PresentationRequest {
    fn default() -> PresentationRequest {
        PresentationRequest {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::PresentProof,
                version: MessageTypeVersion::V20,
                type_: A2AMessage::REQUEST_PRESENTATION.to_string(),
            },
            goal_code: Default::default(),
            comment: Default::default(),
            will_confirm: false,
            present_multiple: false,
            formats: Default::default(),
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
    use crate::aries::messages::proof_presentation::v10::presentation_request::tests::_presentation_request_data;

    fn _attachment() -> ::serde_json::Value {
        json!(_presentation_request_data())
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
        let id = AttachmentId::Other(MessageId::new().to_string());

        let mut attachment = Attachments::new();
        let mut formats = AttachmentFormats::new();

        attachment.add_base64_encoded_json_attachment(id.clone(), _attachment()).unwrap();
        formats.add(id.clone(), AttachmentFormatTypes::IndyProofRequest);

        PresentationRequest {
            id: MessageId::id(),
            comment: Some(_comment()),
            request_presentations_attach: attachment,
            formats,
            service: None,
            thread: None,
            ..PresentationRequest::default()
        }
    }

    #[test]
    fn test_presentation_request_build_works() {
        let attachment_content = json!(_presentation_request_data()).to_string();
        let presentation_request: PresentationRequest = PresentationRequest::default()
            .set_comment(_comment())
            .set_indy_request_presentations_attach(&attachment_content).unwrap();

        assert_eq!(_presentation_request(), presentation_request);
        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/present-proof/2.0/request-presentation","comment":"comment","formats":[{"attach_id":"testid","format":"hlindy/proof-req@v2.0"}],"present_multiple":false,"request_presentations~attach":[{"@id":"testid","data":{"base64":"eyJuYW1lIjoiIiwibm9uX3Jldm9rZWQiOm51bGwsIm5vbmNlIjoiIiwicmVxdWVzdGVkX2F0dHJpYnV0ZXMiOnsiYXR0cmlidXRlXzAiOnsibmFtZSI6Im5hbWUifX0sInJlcXVlc3RlZF9wcmVkaWNhdGVzIjp7fSwidmVyIjpudWxsLCJ2ZXJzaW9uIjoiMS4wIn0="},"mime-type":"application/json"}],"will_confirm":false}"#;
        assert_eq!(expected, json!(presentation_request).to_string());
    }
}
