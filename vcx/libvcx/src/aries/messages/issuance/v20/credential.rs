use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::attachment::{Attachments, AttachmentId};
use crate::aries::messages::ack::PleaseAck;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypePrefix, MessageTypeVersion};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::aries::messages::attachment_format::{AttachmentFormats, AttachmentFormatTypes, AttachmentFormat};
use crate::error::prelude::*;
use crate::aries::messages::thread::Thread;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Credential {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub formats: AttachmentFormats,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
}

impl Credential {
    pub fn create() -> Self {
        Credential::default()
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_indy_credential_attach(self, credential: &str) -> VcxResult<Credential> {
        self.set_credential_attach(credential, AttachmentFormatTypes::IndyCredential)
    }

    pub fn set_credential_attach(mut self, credential: &str, format: AttachmentFormatTypes) -> VcxResult<Credential> {
        let id = AttachmentId::Other(MessageId::new().to_string());
        self.credentials_attach.add_base64_encoded_json_attachment(id.clone(), ::serde_json::Value::String(credential.to_string()))?;
        self.formats.add(id, format);
        Ok(self)
    }

    pub fn credentials_attach_content(&self) -> VcxResult<(&AttachmentFormat, String)> {
        let (attach_id, content) = self.credentials_attach.content()?;
        let format = self.formats.find(&attach_id)?;
        Ok((format, content))
    }
}

please_ack!(Credential);
threadlike!(Credential);

impl Default for Credential {
    fn default() -> Credential {
        Credential {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::CredentialIssuance,
                version: MessageTypeVersion::V20,
                type_: A2AMessage::CREDENTIAL.to_string()
            },
            goal_code: Default::default(),
            comment: Default::default(),
            formats: Default::default(),
            credentials_attach: Default::default(),
            thread: Default::default(),
            please_ack: Default::default(),
            replacement_id: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::issuance::v10::credential_offer::tests::{thread, thread_id};

    fn _attachment() -> ::serde_json::Value {
        json!({
            "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1",
            "values":{"attribute":{"raw":"value","encoded":"1139481716457488690172217916278103335"}}
        })
    }

    fn _comment() -> Option<String> {
        Some(String::from("comment"))
    }

    pub fn _credential() -> Credential {
        let id = AttachmentId::Other(MessageId::new().to_string());

        let mut attachment = Attachments::new();
        let mut formats = AttachmentFormats::new();

        attachment.add_base64_encoded_json_attachment(id.clone(), _attachment()).unwrap();
        formats.add(id.clone(), AttachmentFormatTypes::IndyCredential);

        Credential {
            id: MessageId::id(),
            comment: _comment(),
            thread: thread(),
            credentials_attach: attachment,
            please_ack: None,
            formats,
            ..Credential::default()
        }
    }

    #[test]
    fn test_credential_build_works() {
        let credential: Credential = Credential::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_credential_attach(&_attachment().to_string(), AttachmentFormatTypes::IndyCredential).unwrap();

        assert_eq!(_credential(), credential);
        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/issue-credential/2.0/issue-credential","comment":"comment","credentials~attach":[{"@id":"testid","data":{"base64":"eyJjcmVkX2RlZl9pZCI6Ik5jWXhpRFhrcFlpNm92NUZjWURpMWU6MzpDTDpOY1l4aURYa3BZaTZvdjVGY1lEaTFlOjI6Z3Z0OjEuMDpUQUcxIiwic2NoZW1hX2lkIjoiTmNZeGlEWGtwWWk2b3Y1RmNZRGkxZToyOmd2dDoxLjAiLCJ2YWx1ZXMiOnsiYXR0cmlidXRlIjp7ImVuY29kZWQiOiIxMTM5NDgxNzE2NDU3NDg4NjkwMTcyMjE3OTE2Mjc4MTAzMzM1IiwicmF3IjoidmFsdWUifX19"},"mime-type":"application/json"}],"formats":[{"attach_id":"testid","format":"hlindy/cred@v2.0"}],"~thread":{"received_orders":{},"sender_order":0,"thid":"testid"}}"#;
        assert_eq!(expected, json!(credential).to_string());
    }
}