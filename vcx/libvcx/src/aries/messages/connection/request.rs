use crate::aries::messages::a2a::{A2AMessage, MessageId};
use crate::aries::messages::connection::did_doc::*;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Request {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    pub label: String,
    pub connection: ConnectionData
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: DidDoc,
}

impl Request {
    pub fn create() -> Request {
        Request::default()
    }

    pub fn set_did(mut self, did: String) -> Request {
        self.connection.did = did.clone();
        self.connection.did_doc.set_id(did);
        self
    }

    pub fn set_label(mut self, label: String) -> Request {
        self.label = label;
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Request {
        self.connection.did_doc.set_service_endpoint(service_endpoint);
        self
    }

    pub fn set_keys(mut self, recipient_keys: Vec<String>, routing_keys: Vec<String>) -> Request {
        self.connection.did_doc.set_keys(recipient_keys, routing_keys);
        self
    }
}

impl Default for Request {
    fn default() -> Request {
        Request {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Connections,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::CONNECTION_REQUEST.to_string()
            },
            label: Default::default(),
            connection: Default::default()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::connection::did_doc::tests::*;

    fn _did() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    pub fn _request() -> Request {
        Request {
            id: MessageId::id(),
            label: _label(),
            connection: ConnectionData {
                did: _did(),
                did_doc: _did_doc()
            },
            ..Request::default()
        }
    }

    #[test]
    fn test_request_build_works() {
        let request: Request = Request::default()
            .set_did(_did())
            .set_label(_label())
            .set_service_endpoint(_service_endpoint())
            .set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(_request(), request);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/request","connection":{"DID":"VsKV7grR1BUE29mG2Fm2kX","DIDDoc":{"@context":"https://w3id.org/did/v1","authentication":[{"publicKey":"VsKV7grR1BUE29mG2Fm2kX#1","type":"Ed25519SignatureAuthentication2018"}],"id":"VsKV7grR1BUE29mG2Fm2kX","publicKey":[{"controller":"VsKV7grR1BUE29mG2Fm2kX","id":"VsKV7grR1BUE29mG2Fm2kX#1","publicKeyBase58":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL","type":"Ed25519VerificationKey2018"}],"service":[{"id":"did:example:123456789abcdefghi;indy","priority":0,"recipientKeys":["GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL"],"routingKeys":["Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR","3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU"],"serviceEndpoint":"http://localhost:8080","type":"IndyAgent"}]}},"label":"test"}"#;
        assert_eq!(expected, json!(request).to_string());
    }
}