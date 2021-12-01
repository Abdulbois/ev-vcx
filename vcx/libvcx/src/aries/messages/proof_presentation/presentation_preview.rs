use crate::aries::messages::a2a::message_type::MessageType;
use crate::aries::messages::mime_type::MimeType;
use crate::aries::messages::a2a::message_type::{MessageTypePrefix, MessageTypeVersion};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::utils::libindy::anoncreds::types::CredentialInfo;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationPreview {
    #[serde(rename = "@type")]
    #[serde(default = "default_presentation_preview_type")]
    pub _type: MessageType,
    pub attributes: Vec<Attribute>,
    pub predicates: Vec<Predicate>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred_def_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime-type")]
    pub mime_type: Option<MimeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referent: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Predicate {
    pub name: String,
    pub cred_def_id: Option<String>,
    pub predicate: String,
    pub threshold: i64,
    pub referent: Option<String>,
}

impl Default for PresentationPreview {
    fn default() -> Self {
        PresentationPreview {
            _type: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::PresentProof,
                version: MessageTypeVersion::V10,
                type_: "presentation-preview".to_string()
            },
            attributes: vec![],
            predicates: vec![]
        }
    }
}

fn default_presentation_preview_type() -> MessageType {
    MessageType {
        prefix: MessageTypePrefix::DID,
        family: MessageTypeFamilies::PresentProof,
        version: MessageTypeVersion::V10,
        type_: "presentation-preview".to_string()
    }
}

impl PresentationPreview {
    pub fn create() -> Self {
        PresentationPreview::default()
    }

    pub fn for_credential(credential: &CredentialInfo) -> PresentationPreview {
        let attributes = credential.attrs
            .iter()
            .map(|(attribute, _)| Attribute {
                name: attribute.to_string(),
                cred_def_id: Some(credential.cred_def_id.to_string()),
                mime_type: None,
                value: None,
                referent: None,
            })
            .collect();

        PresentationPreview {
            attributes,
            ..PresentationPreview::default()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn _attachment() -> ::serde_json::Value {
        json!({"presentation": {}})
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _presentation_preview() -> PresentationPreview {
        PresentationPreview {
            attributes: vec![Attribute{
                name: "account".to_string(),
                cred_def_id: Some("BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag".to_string()),
                mime_type: None,
                value: None,
                referent: None
            }],
            predicates: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn test_presentation_preview_for_credential_works() {
        // credential to use
        let credential = CredentialInfo {
            referent: "cred1".to_string(),
            attrs:  map!(
                "account".to_string() => "12345678".to_string(),
                "streetAddress".to_string() => "123 Main Street".to_string()
            ),
            schema_id: "2hoqvcwupRTUNkXn6ArYzs:2:schema_name:0.0.11".to_string(),
            cred_def_id: "BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag".to_string(),
            rev_reg_id: None,
            cred_rev_id: None
        };

        // build presentation preview
        let presentation_preview = PresentationPreview::for_credential(&credential);

        assert_eq!(2, presentation_preview.attributes.len());

        let expected_attribute_1 = Attribute{
            name: "account".to_string(),
            cred_def_id: Some("BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag".to_string()),
            mime_type: None,
            value: None,
            referent: None
        };

        let expected_attribute_2 = Attribute{
            name: "streetAddress".to_string(),
            cred_def_id: Some("BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag".to_string()),
            mime_type: None,
            value: None,
            referent: None
        };

        // check first attribute present
        presentation_preview.attributes
            .iter()
            .find(|attribute| expected_attribute_1.eq(attribute)).unwrap();

        // check second attribute present
        presentation_preview.attributes
            .iter()
            .find(|attribute| expected_attribute_2.eq(attribute)).unwrap();
    }
}
