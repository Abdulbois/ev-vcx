use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::issuance::credential_preview::CredentialPreviewData;
use crate::aries::messages::attachment::{Attachments, AttachmentId};
use crate::aries::messages::mime_type::MimeType;
use crate::error::prelude::*;
use crate::aries::messages::thread::Thread;
use crate::aries::messages::connection::service::Service;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::aries::messages::alias::Alias;
use crate::aries::messages::attachment_format::{AttachmentFormats, AttachmentFormatTypes, AttachmentFormat};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialOffer {
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
    #[serde(default)]
    pub credential_preview: CredentialPreviewData,
    pub formats: AttachmentFormats,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Attachments,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~service")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Service>,
    #[serde(rename = "~alias")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<Alias>,
}

impl CredentialOffer {
    pub fn create() -> Self {
        CredentialOffer::default()
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
        self
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_goal_code(mut self, goal_code: Option<String>) -> Self {
        self.goal_code = goal_code;
        self
    }

    pub fn set_replacement_id(mut self, replacement_id: Option<String>) -> Self {
        self.replacement_id = replacement_id;
        self
    }

    pub fn set_indy_offers_attach(self, credential_offer: &str) -> VcxResult<CredentialOffer> {
        self.set_offers_attach(credential_offer, AttachmentFormatTypes::IndyCredentialOffer)
    }

    pub fn set_offers_attach(mut self, credential_offer: &str, format: AttachmentFormatTypes) -> VcxResult<CredentialOffer> {
        let id = AttachmentId::Other(MessageId::new().to_string());
        self.offers_attach.add_base64_encoded_json_attachment(id.clone(), ::serde_json::Value::String(credential_offer.to_string()))?;
        self.formats.add(id, format);
        Ok(self)
    }

    pub fn set_credential_preview_data(mut self, credential_preview: CredentialPreviewData) -> VcxResult<CredentialOffer> {
        self.credential_preview = credential_preview;
        Ok(self)
    }

    pub fn add_credential_preview_data(mut self, name: &str, value: &serde_json::Value, mime_type: MimeType) -> VcxResult<CredentialOffer> {
        self.credential_preview = self.credential_preview.add_value(name, value, mime_type)?;
        Ok(self)
    }

    pub fn set_thread_id(mut self, id: &str) -> Self {
        self.thread = Some(Thread::new().set_thid(id.to_string()));
        self
    }

    pub fn offers_attach_content(&self) -> VcxResult<(&AttachmentFormat, String)> {
        let (attach_id, content) = self.offers_attach.content()?;
        let format = self.formats.find(&attach_id)?;
        Ok((format, content))
    }
}

impl Default for CredentialOffer {
    fn default() -> CredentialOffer {
        CredentialOffer {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::Endpoint,
                family: MessageTypeFamilies::CredentialIssuance,
                version: MessageTypeVersion::V20,
                type_: A2AMessage::CREDENTIAL_OFFER.to_string()
            },
            goal_code: Default::default(),
            replacement_id: Default::default(),
            comment: Default::default(),
            credential_preview: CredentialPreviewData {
                _type: MessageType {
                    prefix: MessageTypePrefix::Endpoint,
                    family: MessageTypeFamilies::CredentialIssuance,
                    version: MessageTypeVersion::V20,
                    type_: "credential-preview".to_string(),
                },
                attributes: Default::default(),
            },
            formats: Default::default(),
            offers_attach: Default::default(),
            thread: Default::default(),
            service: Default::default(),
            alias: None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn _attachment() -> ::serde_json::Value {
        json!({
            "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1"
        })
    }

    fn _comment() -> Option<String> {
        Some(String::from("comment"))
    }

    pub fn _value() -> (&'static str, &'static str) {
        ("attribute", "value")
    }

    pub fn _preview_data() -> CredentialPreviewData {
        let (name, value) = _value();
        CredentialPreviewData::new()
            .add_value(name,  &json!(value), MimeType::Plain).unwrap()
    }

    pub fn thread() -> Thread {
        Thread::new().set_thid(_credential_offer().id.0)
    }

    pub fn thread_id() -> String {
        thread().thid.unwrap()
    }

    pub fn _credential_offer() -> CredentialOffer {
        let id = AttachmentId::Other(MessageId::new().to_string());

        let mut attachment = Attachments::new();
        let mut formats = AttachmentFormats::new();

        attachment.add_base64_encoded_json_attachment(id.clone(), _attachment()).unwrap();
        formats.add(id.clone(), AttachmentFormatTypes::IndyCredentialOffer);

        CredentialOffer {
            id: MessageId::id(),
            comment: _comment(),
            credential_preview: _preview_data(),
            offers_attach: attachment,
            formats,
            thread: None,
            service: None,
            ..CredentialOffer::default()
        }
    }

    #[test]
    fn test_credential_offer_build_works() {
        let credential_offer: CredentialOffer = CredentialOffer::create()
            .set_comment(_comment())
            .set_credential_preview_data(_preview_data()).unwrap()
            .set_indy_offers_attach(&_attachment().to_string()).unwrap();

        assert_eq!(_credential_offer(), credential_offer);
        let expected = r#"{"@id":"testid","@type":"https://didcomm.org/issue-credential/2.0/offer-credential","comment":"comment","credential_preview":{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/credential-preview","attributes":[{"name":"attribute","value":"value"}]},"formats":[{"attach_id":"testid","format":"hlindy/cred-abstract@v2.0"}],"offers~attach":[{"@id":"testid","data":{"base64":"eyJjcmVkX2RlZl9pZCI6Ik5jWXhpRFhrcFlpNm92NUZjWURpMWU6MzpDTDpOY1l4aURYa3BZaTZvdjVGY1lEaTFlOjI6Z3Z0OjEuMDpUQUcxIiwic2NoZW1hX2lkIjoiTmNZeGlEWGtwWWk2b3Y1RmNZRGkxZToyOmd2dDoxLjAifQ=="},"mime-type":"application/json"}]}"#;
        assert_eq!(expected, json!(credential_offer).to_string());
    }
}