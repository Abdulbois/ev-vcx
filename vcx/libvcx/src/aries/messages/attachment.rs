use std::str::from_utf8;
use serde::{de, Serialize, Serializer, Deserialize, Deserializer};
use serde_json;
use serde_json::Value;

use crate::error::prelude::*;

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

    pub fn content(&self) -> VcxResult<(AttachmentId, String)> {
        match self.get() {
            Some(Attachment::JSON(ref attach)) => attach.data(),
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

    pub fn data(&self) -> VcxResult<(AttachmentId, String)> {
        let id = self.id.clone();
        let data = self.data.get_bytes()?;
        let data = from_utf8(data.as_slice())
            .map(String::from)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAttachmentEncoding, format!("Wrong bytes in attachment. Err: {:?}",err)))?;
        Ok((id, data))
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
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAttachmentEncoding, format!("Wrong bytes in attachment. Err: {:?}",err)))
            }
        }
    }
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
        let (_, content) = json_attachment.data().unwrap();
        assert_eq!(_json().to_string(), content);
    }

    #[test]
    fn test_attachments_works() {
        {
            let mut attachments = Attachments::new();
            assert_eq!(0, attachments.0.len());

            let json: Json = Json::new(AttachmentId::Credential, _json(), AttachmentEncoding::Base64).unwrap();
            attachments.add(Attachment::JSON(json));
            assert_eq!(1, attachments.0.len());

            let (_, content) = attachments.content().unwrap();
            assert_eq!(_json().to_string(), content);
        }

        {
            let mut attachments = Attachments::new();
            attachments.add_json_attachment(AttachmentId::Credential, _json(), AttachmentEncoding::Base64).unwrap();
            let (_, content) = attachments.content().unwrap();
            assert_eq!(_json().to_string(), content);
        }
    }
}
