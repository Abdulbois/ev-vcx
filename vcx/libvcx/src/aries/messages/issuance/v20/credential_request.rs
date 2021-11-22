use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::attachment::{Attachments, AttachmentId};
use crate::error::VcxResult;
use crate::aries::messages::thread::Thread;
use crate::aries::messages::transport::Transport;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::aries::messages::attachment_format::{AttachmentFormats, AttachmentFormatTypes, AttachmentFormat};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialRequest {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub formats: AttachmentFormats,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~transport")]
    pub transport: Option<Transport>,
}

impl CredentialRequest {
    pub fn create() -> Self {
        CredentialRequest::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_indy_requests_attach(self, credential_request: &str) -> VcxResult<CredentialRequest> {
        self.set_requests_attach(credential_request, AttachmentFormatTypes::IndyCredentialRequest)
    }

    pub fn set_requests_attach(mut self, credential_request: &str, format: AttachmentFormatTypes) -> VcxResult<CredentialRequest> {
        let id = AttachmentId::Other(MessageId::new().to_string());
        self.requests_attach.add_base64_encoded_json_attachment(id.clone(), ::serde_json::Value::String(credential_request.to_string()))?;
        self.formats.add(id, format);
        Ok(self)
    }

    pub fn requests_attach_content(&self) -> VcxResult<(&AttachmentFormat, String)> {
        let (attach_id, content) = self.requests_attach.content()?;
        let format = self.formats.find(&attach_id)?;
        Ok((format, content))
    }
}

threadlike!(CredentialRequest);
return_route!(CredentialRequest);

impl Default for CredentialRequest {
    fn default() -> CredentialRequest {
        CredentialRequest {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::CredentialIssuance,
                version: MessageTypeVersion::V20,
                type_: A2AMessage::REQUEST_CREDENTIAL.to_string()
            },
            goal_code: Default::default(),
            comment: Default::default(),
            formats: Default::default(),
            requests_attach: Default::default(),
            thread: Default::default(),
            transport: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::issuance::v10::credential_offer::tests::{thread, thread_id};

    fn _attachment() -> ::serde_json::Value {
        json!({
            "prover_did":"VsKV7grR1BUE29mG2Fm2kX",
            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1"
        })
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _credential_request() -> CredentialRequest {
        let id = AttachmentId::Other(MessageId::new().to_string());

        let mut attachment = Attachments::new();
        let mut formats = AttachmentFormats::new();

        attachment.add_base64_encoded_json_attachment(id.clone(), _attachment()).unwrap();
        formats.add(id.clone(), AttachmentFormatTypes::IndyCredentialRequest);

        CredentialRequest {
            id: MessageId::id(),
            comment: Some(_comment()),
            requests_attach: attachment,
            thread: thread(),
            formats,
            transport: None,
            ..CredentialRequest::default()
        }
    }

    #[test]
    fn test_credential_request_build_works() {
        let credential_request: CredentialRequest = CredentialRequest::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_indy_requests_attach(&_attachment().to_string()).unwrap();

        assert_eq!(_credential_request(), credential_request);
        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/issue-credential/2.0/request-credential","comment":"comment","formats":[{"attach_id":"testid","format":"hlindy/cred-req@v2.0"}],"requests~attach":[{"@id":"testid","data":{"base64":"eyJjcmVkX2RlZl9pZCI6Ik5jWXhpRFhrcFlpNm92NUZjWURpMWU6MzpDTDpOY1l4aURYa3BZaTZvdjVGY1lEaTFlOjI6Z3Z0OjEuMDpUQUcxIiwicHJvdmVyX2RpZCI6IlZzS1Y3Z3JSMUJVRTI5bUcyRm0ya1gifQ=="},"mime-type":"application/json"}],"~thread":{"received_orders":{},"sender_order":0,"thid":"testid"},"~transport":null}"#;
        assert_eq!(expected, json!(credential_request).to_string());
    }
}
