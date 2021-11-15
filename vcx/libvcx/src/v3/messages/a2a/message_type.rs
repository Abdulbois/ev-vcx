use crate::v3::messages::a2a::message_family::MessageTypeFamilies;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use crate::error::prelude::*;
use regex::{Regex, Match};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MessageType {
    pub prefix: MessageTypePrefix,
    pub family: MessageTypeFamilies,
    pub version: MessageTypeVersion,
    pub type_: String,
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;

        match value.as_str() {
            Some(type_) => {
                let (prefix, family, version, type_) = parse_message_type(type_).map_err(de::Error::custom)?;
                Ok(MessageType {
                    prefix: MessageTypePrefix::from(prefix),
                    family: MessageTypeFamilies::from(family),
                    version: MessageTypeVersion::from(version),
                    type_,
                })
            }
            val => Err(de::Error::custom(format!("Unexpected @type field structure: {:?}", val)))
        }
    }
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = Value::String(self.to_string());
        value.serialize(serializer)
    }
}

pub fn parse_message_type(message_type: &str) -> VcxResult<(String, String, String, String)> {
    trace!("parse_message_type >>> message_type: {:?}", secret!(message_type));

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?x)
            (?P<prefix>did:\w+:\w+;spec|https://didcomm.org)/
            (?P<family>.*)/
            (?P<version>.*)/
            (?P<type>.*)").unwrap();
    }

    let message_type = RE.captures(message_type)
        .and_then(|cap| {
            let prefix = cap.name("prefix").as_ref().map(Match::as_str);
            let family = cap.name("family").as_ref().map(Match::as_str);
            let version = cap.name("version").as_ref().map(Match::as_str);
            let type_ = cap.name("type").as_ref().map(Match::as_str);

            match (prefix, family, version, type_) {
                (Some(prefix), Some(family), Some(version), Some(type_)) =>
                    Some((prefix.to_string(), family.to_string(), version.to_string(), type_.to_string())),
                _ => None
            }
        }).ok_or(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot parse @type from string: {}", message_type)))?;

    trace!("parse_message_type <<< message_type: {:?}", secret!(message_type));
    Ok(message_type)
}

impl ::std::string::ToString for MessageType {
    fn to_string(&self) -> String {
        format!("{}/{}/{}/{}",
                self.prefix.to_string(),
                self.family.to_string(),
                self.version.to_string(),
                self.type_)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, EnumIter)]
pub enum MessageTypePrefix {
    DID,
    Endpoint,
}

impl From<String> for MessageTypePrefix {
    fn from(family: String) -> Self {
        match family.as_str() {
            "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec" => MessageTypePrefix::DID,
            "https://didcomm.org" => MessageTypePrefix::Endpoint,
            _ => MessageTypePrefix::DID
        }
    }
}

impl ToString for MessageTypePrefix {
    fn to_string(&self) -> String {
        match self {
            MessageTypePrefix::DID => "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec".to_string(),
            MessageTypePrefix::Endpoint => "https://didcomm.org".to_string(),
        }
    }
}

impl Default for MessageTypePrefix {
    fn default() -> MessageTypePrefix {
        MessageTypePrefix::DID
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, EnumIter)]
pub enum MessageTypeVersion {
    V09,
    V10,
    V11,
}

impl From<String> for MessageTypeVersion {
    fn from(family: String) -> Self {
        match family.as_str() {
            "0.9" => MessageTypeVersion::V09,
            "1.0" => MessageTypeVersion::V10,
            "1.1" => MessageTypeVersion::V11,
            _ => MessageTypeVersion::V10
        }
    }
}

impl ToString for MessageTypeVersion {
    fn to_string(&self) -> String {
        match self {
            MessageTypeVersion::V09 => "0.9".to_string(),
            MessageTypeVersion::V10 => "1.0".to_string(),
            MessageTypeVersion::V11 => "1.1".to_string(),
        }
    }
}

impl Default for MessageTypeVersion {
    fn default() -> MessageTypeVersion {
        MessageTypeVersion::V10
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::v3::messages::a2a::A2AMessage;

    fn _message_type_with_did() -> &'static str {
        "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation"
    }

    fn _message_type_with_endpoint() -> &'static str {
        "https://didcomm.org/connections/1.0/invitation"
    }

    #[test]
    fn test_parse_message_type_with_did() {
        let type_ = _message_type_with_did();
        let message_type: MessageType = ::serde_json::from_value(serde_json::Value::String(type_.to_string())).unwrap();
        let expected = MessageType {
            prefix: MessageTypePrefix::DID,
            family: MessageTypeFamilies::Connections,
            version: MessageTypeVersion::V10,
            type_: A2AMessage::CONNECTION_INVITATION.to_string()
        };
        assert_eq!(expected, message_type);
        assert_eq!(type_.to_string(), message_type.to_string());
    }

    #[test]
    fn test_parse_message_type_with_endpoint() {
        let type_ = _message_type_with_endpoint();
        let message_type: MessageType = ::serde_json::from_value(serde_json::Value::String(type_.to_string())).unwrap();
        let expected = MessageType {
            prefix: MessageTypePrefix::Endpoint,
            family: MessageTypeFamilies::Connections,
            version: MessageTypeVersion::V10,
            type_: A2AMessage::CONNECTION_INVITATION.to_string()
        };
        assert_eq!(expected, message_type);
        assert_eq!(type_.to_string(), message_type.to_string());
    }
}