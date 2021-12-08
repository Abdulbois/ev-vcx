use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::attachment::{Attachments, AttachmentId};
use crate::aries::messages::transport::Transport;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypePrefix, MessageTypeVersion};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::error::VcxResult;
use crate::aries::messages::thread::Thread;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialRequest {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(skip_serializing_if = "Option::is_none")]
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

    pub fn set_requests_attach(mut self, credential_request: String) -> VcxResult<CredentialRequest> {
        self.requests_attach.add_base64_encoded_json_attachment(AttachmentId::CredentialRequest, ::serde_json::Value::String(credential_request))?;
        Ok(self)
    }
}

threadlike!(CredentialRequest);
return_route!(CredentialRequest);

impl Default for CredentialRequest {
    fn default() -> CredentialRequest {
        CredentialRequest {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::CredentialIssuance,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::REQUEST_CREDENTIAL.to_string(),
            },
            comment: Default::default(),
            requests_attach: Default::default(),
            thread: Default::default(),
            transport: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::issuance::v10::credential_offer::tests::{thread_id, thread};

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
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::CredentialRequest, _attachment()).unwrap();

        CredentialRequest {
            id: MessageId::id(),
            comment: Some(_comment()),
            requests_attach: attachment,
            thread: thread(),
            transport: None,
            ..CredentialRequest::default()
        }
    }

    #[test]
    fn test_credential_request_build_works() {
        let credential_request: CredentialRequest = CredentialRequest::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_requests_attach(_attachment().to_string()).unwrap();

        assert_eq!(_credential_request(), credential_request);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/request-credential","comment":"comment","requests~attach":[{"@id":"libindy-cred-request-0","data":{"base64":"eyJjcmVkX2RlZl9pZCI6Ik5jWXhpRFhrcFlpNm92NUZjWURpMWU6MzpDTDpOY1l4aURYa3BZaTZvdjVGY1lEaTFlOjI6Z3Z0OjEuMDpUQUcxIiwicHJvdmVyX2RpZCI6IlZzS1Y3Z3JSMUJVRTI5bUcyRm0ya1gifQ=="},"mime-type":"application/json"}],"~thread":{"received_orders":{},"sender_order":0,"thid":"testid"}}"#;
        assert_eq!(expected, json!(credential_request).to_string());
    }
}
