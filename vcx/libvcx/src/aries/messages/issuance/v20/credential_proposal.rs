use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::issuance::credential_preview::CredentialPreviewData;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::error::VcxResult;
use crate::aries::messages::thread::Thread;
use crate::aries::messages::attachment_format::{AttachmentFormats, AttachmentFormatTypes, AttachmentFormat};
use crate::aries::messages::attachment::{Attachments, AttachmentId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialProposal {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_preview: CredentialPreviewData,
    pub formats: AttachmentFormats,
    #[serde(rename = "filters~attach")]
    pub filters_attach: Attachments,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
}

impl CredentialProposal {
    pub fn create() -> Self {
        CredentialProposal::default()
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_indy_filters_attach(self, filters: &str) -> VcxResult<CredentialProposal> {
        self.set_filters_attach(filters, AttachmentFormatTypes::IndyCredential)
    }

    pub fn set_filters_attach(mut self, filters: &str, format: AttachmentFormatTypes) -> VcxResult<CredentialProposal> {
        let id = AttachmentId::Other(MessageId::new().to_string());
        self.filters_attach.add_base64_encoded_json_attachment(id.clone(), ::serde_json::Value::String(filters.to_string()))?;
        self.formats.add(id, format);
        Ok(self)
    }

    pub fn set_thread_id(mut self, id: &str) -> Self {
        self.thread = Some(Thread::new().set_thid(id.to_string()));
        self
    }

    pub fn filters_attach_content(&self) -> VcxResult<(&AttachmentFormat, String)> {
        let (attach_id, content) = self.filters_attach.content()?;
        let format = self.formats.find(&attach_id)?;
        Ok((format, content))
    }
}

impl Default for CredentialProposal {
    fn default() -> CredentialProposal {
        CredentialProposal {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::CredentialIssuance,
                version: MessageTypeVersion::V20,
                type_: A2AMessage::PROPOSE_CREDENTIAL.to_string()
            },
            goal_code: Default::default(),
            comment: Default::default(),
            credential_preview: Default::default(),
            formats: Default::default(),
            thread: Default::default(),
            filters_attach: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::issuance::credential_offer::tests::{thread, thread_id, _value};
    use crate::aries::messages::mime_type::MimeType;

    fn _attachment() -> ::serde_json::Value {
        json!({"credential offer": {}})
    }

    fn _comment() -> Option<String> {
        Some(String::from("comment"))
    }

    fn _schema_id() -> String { String::from("schema:id") }

    fn _cred_def_id() -> String { String::from("cred_def_id:id") }

    fn _credential_preview_data() -> CredentialPreviewData {
        let (name, value) = _value();

        CredentialPreviewData::new()
            .add_value(name,  &json!(value), MimeType::Plain).unwrap()
    }

    pub fn _credential_proposal() -> CredentialProposal {
        CredentialProposal {
            id: MessageId::id(),
            comment: _comment(),
            thread: Some(thread()),
            ..CredentialProposal::default()
        }
    }

    #[test]
    fn test_credential_proposal_build_works() {
        let credential_proposal: CredentialProposal = CredentialProposal::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id());

        assert_eq!(_credential_proposal(), credential_proposal);
        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/issue-credential/2.0/propose-credential","comment":"comment","credential_preview":{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/credential-preview","attributes":[]},"filters~attach":[],"formats":[],"~thread":{"received_orders":{},"sender_order":0,"thid":"testid"}}"#;
        assert_eq!(expected, json!(credential_proposal).to_string());
    }
}