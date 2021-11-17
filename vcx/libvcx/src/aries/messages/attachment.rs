use std::str::from_utf8;
use serde::{de, Serialize, Serializer, Deserialize, Deserializer};
use serde_json;
use serde_json::Value;

use crate::error::{VcxResult, VcxError, VcxErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Attachments(pub Vec<Attachment>);

impl Attachments {
    pub fn new() -> Attachments {
        Attachments::default()
    }

    pub fn get(&self) -> Option<&Attachment> {
        self.0.get(0)
    }

    pub fn add(&mut self, attachment: Attachment) {
        self.0.push(attachment);
    }

    pub fn add_json_attachment(&mut self, id: AttachmentId, json: serde_json::Value, encoding: AttachmentEncoding) -> VcxResult<()> {
        let json: Json = Json::new(id, json, encoding)?;
        self.add(Attachment::JSON(json));
        Ok(())
    }

    pub fn add_base64_encoded_json_attachment(&mut self, id: AttachmentId, json: serde_json::Value) -> VcxResult<()> {
        self.add_json_attachment(id, json, AttachmentEncoding::Base64)
    }

    pub fn content(&self) -> VcxResult<String> {
        match self.get() {
            Some(Attachment::JSON(ref attach)) => attach.get_data(),
            Some(Attachment::DidcomPlainJSON(ref attach)) => attach.get_data(),
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidAttachmentEncoding, "Unsupported Attachment type"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mime-type")]
pub enum Attachment {
    #[serde(rename = "application/json")]
    JSON(Json),
    #[serde(rename = "application/didcomm-plain+json")]
    DidcomPlainJSON(Json),
    Blank,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Json {
    #[serde(rename = "@id")]
    id: AttachmentId,
    data: AttachmentData,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttachmentId {
    OutofbandRequest,
    CredentialOffer,
    CredentialRequest,
    Credential,
    PresentationRequest,
    Presentation,
    Other(String),
}

impl Serialize for AttachmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            AttachmentId::OutofbandRequest => "request-0",
            AttachmentId::CredentialOffer => "libindy-cred-offer-0",
            AttachmentId::CredentialRequest => "libindy-cred-request-0",
            AttachmentId::Credential => "libindy-cred-0",
            AttachmentId::PresentationRequest => "libindy-request-presentation-0",
            AttachmentId::Presentation => "libindy-presentation-0",
            AttachmentId::Other(type_) => type_,
        };
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AttachmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("request-0") => Ok(AttachmentId::OutofbandRequest),
            Some("libindy-cred-offer-0") => Ok(AttachmentId::CredentialOffer),
            Some("libindy-cred-request-0") => Ok(AttachmentId::CredentialRequest),
            Some("libindy-cred-0") => Ok(AttachmentId::Credential),
            Some("libindy-request-presentation-0") => Ok(AttachmentId::PresentationRequest),
            Some("libindy-presentation-0") => Ok(AttachmentId::Presentation),
            Some(_type) => Ok(AttachmentId::Other(_type.to_string())),
            val => Err(de::Error::custom(format!("Unexpected Attachment id: {:?}", val)))
        }
    }
}

impl Json {
    pub fn new(id: AttachmentId, json: serde_json::Value, encoding: AttachmentEncoding) -> VcxResult<Json> {
        let data: AttachmentData = match encoding {
            AttachmentEncoding::Base64 => {
                AttachmentData::Base64(
                    base64::encode(&
                        match json {
                            ::serde_json::Value::Object(obj) => {
                                json!(obj).to_string()
                            }
                            ::serde_json::Value::String(str) => str,
                            val => return Err(VcxError::from_msg(VcxErrorKind::InvalidAttachmentEncoding, format!("Unsupported Json value: {:?}", val)))
                        }
                    )
                )
            }
        };
        Ok(Json {
            id,
            data,
        })
    }

    pub fn get_data(&self) -> VcxResult<String> {
        let data = self.data.get_bytes()?;
        from_utf8(data.as_slice())
            .map(|s| s.to_string())
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAttachmentEncoding, format!("Wrong bytes in attachment. Err: {:?}", err)))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AttachmentEncoding {
    Base64
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttachmentData {
    #[serde(rename = "base64")]
    Base64(String)
}

impl AttachmentData {
    pub fn get_bytes(&self) -> VcxResult<Vec<u8>> {
        match self {
            AttachmentData::Base64(s) => {
                base64::decode(s)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAttachmentEncoding, format!("Wrong bytes in attachment. Err: {:?}", err)))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct MessageWithAttachment {
    #[serde(rename = "filters~attach")]
    pub filters_attach: Option<Attachments>,
    #[serde(rename = "proposal~attach")]
    pub proposal_attach: Option<Attachments>,
    #[serde(rename = "did_doc~attach")]
    pub did_doc_attach: Option<Attachments>,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Option<Attachments>,
    #[serde(rename = "request~attach")]
    pub request_attach: Option<Attachments>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Option<Attachments>,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Option<Attachments>,
    #[serde(rename = "~attach")]
    pub attach: Option<Attachments>,
    #[serde(rename = "diagram~attach")]
    pub diagram_attach: Option<Attachments>,
    #[serde(rename = "messages~attach")]
    pub messages_attach: Option<Attachments>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Option<Attachments>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Option<Attachments>,
    #[serde(rename = "img~attach")]
    pub img_attach: Option<Attachments>,
}

pub fn extract_attached_message(message: &str) -> VcxResult<String> {
    trace!("Attachments::extract_attached_message >>>");
    debug!("Attachments: extracting attachment from message");

    let message_with_attachment: MessageWithAttachment = serde_json::from_str(message)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Unable to parse MessageWithAttachment from JSON string. Err: {:?}", err),
        ))?;

    let attachment: Attachments = message_with_attachment.filters_attach
        .or(message_with_attachment.proposal_attach)
        .or(message_with_attachment.did_doc_attach)
        .or(message_with_attachment.offers_attach)
        .or(message_with_attachment.request_attach)
        .or(message_with_attachment.requests_attach)
        .or(message_with_attachment.credentials_attach)
        .or(message_with_attachment.attach)
        .or(message_with_attachment.diagram_attach)
        .or(message_with_attachment.messages_attach)
        .or(message_with_attachment.request_presentations_attach)
        .or(message_with_attachment.presentations_attach)
        .or(message_with_attachment.img_attach)
        .ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            "Message does not contain attachment",
        ))?;

    let content = attachment.content()?;
    trace!("Attachments::extract_attached_message <<< content: {:?}", secret!(content));
    Ok(content)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn _json() -> serde_json::Value {
        json!({"field": "value"})
    }

    #[test]
    fn test_create_json_attachment_works() {
        let json_attachment: Json = Json::new(AttachmentId::Credential, _json(), AttachmentEncoding::Base64).unwrap();
        assert_eq!(vec![123, 34, 102, 105, 101, 108, 100, 34, 58, 34, 118, 97, 108, 117, 101, 34, 125], json_attachment.data.get_bytes().unwrap());
        assert_eq!(_json().to_string(), json_attachment.get_data().unwrap());
    }

    #[test]
    fn test_attachments_works() {
        {
            let mut attachments = Attachments::new();
            assert_eq!(0, attachments.0.len());

            let json: Json = Json::new(AttachmentId::Credential, _json(), AttachmentEncoding::Base64).unwrap();
            attachments.add(Attachment::JSON(json));
            assert_eq!(1, attachments.0.len());

            assert_eq!(_json().to_string(), attachments.content().unwrap());
        }

        {
            let mut attachments = Attachments::new();
            attachments.add_json_attachment(AttachmentId::Credential, _json(), AttachmentEncoding::Base64).unwrap();
            assert_eq!(_json().to_string(), attachments.content().unwrap());
        }
    }

    #[test]
    fn test_extract_attached_message() {
        let message = r#"{"request_presentations~attach": [{"@id": "libindy-request-presentation-0", "data": {"base64": "eyJuYW1lIjoicHJvb2ZfZnJvbV9hbGljZSIsIm5vbl9yZXZva2VkIjpudWxsLCJub25jZSI6Ijc3MzQ4MDc1MzM0NDk3MDI5ODY2MDgiLCJyZXF1ZXN0ZWRfYXR0cmlidXRlcyI6eyJhdHRyaWJ1dGVfMCI6eyJuYW1lIjoiTWVtYmVySUQifX0sInJlcXVlc3RlZF9wcmVkaWNhdGVzIjp7fSwidmVyIjoiMS4wIiwidmVyc2lvbiI6IjEuMCJ9"}, "mime-type": "application/json"}]}"#;
        let attached_message = extract_attached_message(message).unwrap();
        let expected_message = r#"{"name":"proof_from_alice","non_revoked":null,"nonce":"7734807533449702986608","requested_attributes":{"attribute_0":{"name":"MemberID"}},"requested_predicates":{},"ver":"1.0","version":"1.0"}"#;
        assert_eq!(attached_message, expected_message);
    }
}
