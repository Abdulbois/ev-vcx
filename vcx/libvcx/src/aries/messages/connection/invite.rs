use crate::aries::messages::a2a::{A2AMessage, MessageId};
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Invitation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    pub label: String,
    #[serde(rename = "recipientKeys")]
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    #[serde(rename = "routingKeys")]
    pub routing_keys: Vec<String>,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "profileUrl")]
    pub profile_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_did: Option<String>,
}

impl Invitation {
    pub fn create() -> Invitation {
        Invitation::default()
    }

    pub fn id(&self) -> &MessageId {
        &self.id
    }

    pub fn set_label(mut self, label: String) -> Invitation {
        self.label = label;
        self
    }

    pub fn set_id(mut self, id: String) -> Invitation {
        self.id = MessageId(id);
        self
    }

    pub fn set_opt_profile_url(mut self, profile_url: Option<String>) -> Invitation {
        self.profile_url = profile_url;
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Invitation {
        self.service_endpoint = service_endpoint;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<String>) -> Invitation {
        self.recipient_keys = recipient_keys;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<String>) -> Invitation {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_opt_public_did(mut self, public_did: Option<String>) -> Invitation {
        self.public_did = public_did;
        self
    }
}

impl Default for Invitation {
    fn default() -> Invitation {
        Invitation {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Connections,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::CONNECTION_INVITATION.to_string(),
            },
            label: Default::default(),
            recipient_keys: Default::default(),
            routing_keys: Default::default(),
            service_endpoint: Default::default(),
            profile_url: Default::default(),
            public_did: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::connection::did_doc::tests::*;

    pub fn _invitation() -> Invitation {
        Invitation {
            id: MessageId::id(),
            label: _label(),
            recipient_keys: _recipient_keys(),
            routing_keys: _routing_keys(),
            service_endpoint: _service_endpoint(),
            profile_url: None,
            public_did: None,
            ..Invitation::default()
        }
    }

    pub fn _invitation_json() -> String {
        ::serde_json::to_string(&_invitation()).unwrap()
    }

    #[test]
    fn test_request_build_works() {
        let invitation: Invitation = Invitation::default()
            .set_label(_label())
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        assert_eq!(_invitation(), invitation);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation","label":"test","recipientKeys":["GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL"],"routingKeys":["Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR","3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU"],"serviceEndpoint":"http://localhost:8080"}"#;
        assert_eq!(expected, json!(invitation).to_string());
    }
}