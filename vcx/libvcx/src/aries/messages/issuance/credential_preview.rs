use serde_json::Value;
use crate::aries::messages::mime_type::MimeType;
use crate::aries::messages::a2a::message_type::MessageType;
use crate::aries::messages::a2a::message_type::{MessageTypePrefix, MessageTypeVersion};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::error::VcxResult;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialPreviewData {
    #[serde(rename = "@type")]
    pub _type: MessageType,
    pub attributes: Vec<CredentialValue>,
}

impl CredentialPreviewData {
    pub fn new() -> Self {
        CredentialPreviewData::default()
    }

    pub fn add_value(mut self, name: &str, value: &Value, mime_type: MimeType) -> VcxResult<CredentialPreviewData> {
        let data_value = match mime_type {
            MimeType::Plain => {
                CredentialValue {
                    name: name.to_string(),
                    value: value.clone(),
                    _type: None,
                }
            }
        };
        self.attributes.push(data_value);
        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialValue {
    pub name: String,
    pub value: Value,
    #[serde(rename = "mime-type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _type: Option<MimeType>,
}

impl Default for CredentialPreviewData {
    fn default() -> CredentialPreviewData {
        CredentialPreviewData {
            _type: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::CredentialIssuance,
                version: MessageTypeVersion::V10,
                type_: "credential-preview".to_string(),
            },
            attributes: vec![],
        }
    }
}
