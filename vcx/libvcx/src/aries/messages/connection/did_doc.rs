use crate::aries::messages::connection::invite::Invitation;
use crate::aries::messages::outofband::invitation::Invitation as OutofbandInvitation;

use crate::error::prelude::*;
use reqwest::Url;
use crate::utils::validation::validate_verkey;
use rust_base58::{FromBase58, ToBase58};

pub const CONTEXT: &str = "https://w3id.org/did/v1";
pub const KEY_TYPE: &str = "Ed25519VerificationKey2018";
pub const KEY_AUTHENTICATION_TYPE: &str = "Ed25519SignatureAuthentication2018";
pub const SERVICE_SUFFIX: &str = "indy";
pub const SERVICE_TYPE: &str = "IndyAgent";
pub const SERVICE_ID: &str = "#inline";
pub const OUTOFBAND_SERVICE_TYPE: &str = "did-communication";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DidDoc {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    #[serde(rename = "publicKey")]
    #[serde(alias = "verificationMethod")]
    pub public_key: Vec<Ed25519PublicKey>, // TODO: A DID document MAY include a publicKey property??? (https://w3c.github.io/did-core/#public-keys)
    #[serde(default)]
    pub authentication: Vec<Authentication>,
    pub service: Vec<Service>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Ed25519PublicKey {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String, // all list of types: https://w3c-ccg.github.io/ld-cryptosuite-registry/
    pub controller: String,
    #[serde(rename = "publicKeyBase58")]
    pub public_key_base_58: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Authentication {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub priority: u32,
    #[serde(default)]
    #[serde(rename = "recipientKeys")]
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    #[serde(rename = "routingKeys")]
    pub routing_keys: Vec<String>,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

impl Default for DidDoc {
    fn default() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: String::new(),
            public_key: vec![],
            authentication: vec![],
            service: vec![Service::default()],
        }
    }
}

impl DidDoc {
    pub fn create() -> DidDoc {
        DidDoc::default()
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_service_endpoint(&mut self, service_endpoint: String) {
        self.service.get_mut(0)
            .map(|service| {
                service.service_endpoint = service_endpoint;
                service
            });
    }

    pub fn set_keys(&mut self, recipient_keys: Vec<String>, routing_keys: Vec<String>) {
        let mut id = 0;

        recipient_keys
            .iter()
            .for_each(|key| {
                id += 1;

                let key_id = id.to_string();
                let key_reference = DidDoc::_build_key_reference(&self.id, &key_id);

                self.public_key.push(
                    Ed25519PublicKey {
                        id: key_reference.clone(),
                        type_: String::from(KEY_TYPE),
                        controller: self.id.clone(),
                        public_key_base_58: key.clone(),
                    });

                self.authentication.push(
                    Authentication {
                        type_: String::from(KEY_AUTHENTICATION_TYPE),
                        public_key: key_reference.clone(),
                    });


                self.service.get_mut(0)
                    .map(|service| {
                        service.recipient_keys.push(key.clone());
                        service
                    });
            });

        routing_keys
            .iter()
            .for_each(|key| {
                // Note: comment lines 123 - 134 and append key instead key_reference to be compatible with Streetcred
//                id += 1;
//
//                let key_id = id.to_string();
//                let key_reference = DidDoc::_build_key_reference(&self.id, &key_id);
//
//                self.public_key.push(
//                    Ed25519PublicKey {
//                        id: key_id,
//                        type_: String::from(KEY_TYPE),
//                        controller: self.id.clone(),
//                        public_key_base_58: key.clone(),
//                    });

                self.service.get_mut(0)
                    .map(|service| {
                        service.routing_keys.push(key.to_string());
                        service
                    });
            });
    }

    pub fn validate(&self) -> VcxResult<()> {
        trace!("DidDoc::validate >>> {:?}", secret!(self));

        if self.context != CONTEXT {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidDIDDoc, format!("DIDDoc validation failed: Unsupported @context value: {:?}", self.context)));
        }

//        if self.id.is_empty() {
//            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "DIDDoc validation failed: id is empty"));
//        }

        for service in self.service.iter() {
            Url::parse(&service.service_endpoint)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidDIDDoc, format!("DIDDoc validation failed: Invalid endpoint \"{:?}\", err: {:?}", service.service_endpoint, err)))?;

            service.recipient_keys
                .iter()
                .map(|key| self.validate_recipient_key(key))
                .collect::<VcxResult<()>>()?;

            service.routing_keys
                .iter()
                .map(|key| self.validate_routing_key(key))
                .collect::<VcxResult<()>>()?;
        }

        trace!("DidDoc::validate <<<");
        Ok(())
    }

    fn validate_recipient_key(&self, key: &str) -> VcxResult<()> {
        let public_key = self.validate_public_key(key)?;
        self.validate_authentication(&public_key.id)
    }

    fn validate_routing_key(&self, key: &str) -> VcxResult<()> {
        if DidDoc::_key_parts(key).len() == 2 {
            self.validate_public_key(key)?;
        } else {
            validate_verkey(key)?;
        }
        Ok(())
    }

    fn validate_public_key(&self, target_key: &str) -> VcxResult<&Ed25519PublicKey> {
        let id = DidDoc::_parse_key_reference(target_key);

        let key = self.public_key.iter().find(|key_| key_.id == id.to_string() || key_.public_key_base_58 == id.to_string() || key_.id == target_key.to_string())
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidDIDDoc, format!("DIDDoc validation failed: Cannot find PublicKey definition for key: {:?}", id)))?;

        if key.type_ != KEY_TYPE {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidDIDDoc, format!("DIDDoc validation failed: Unsupported PublicKey type: {:?}", key.type_)));
        }

        validate_verkey(&key.public_key_base_58)?;

        Ok(key)
    }

    fn validate_authentication(&self, target_key: &str) -> VcxResult<()> {
        if self.authentication.is_empty() {
            return Ok(());
        }

        let key = self.authentication.iter().find(|key_|
            key_.public_key == target_key.to_string() ||
                DidDoc::_parse_key_reference(&key_.public_key) == target_key.to_string())
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidDIDDoc, format!("DIDDoc validation failed: Cannot find Authentication section for key: {:?}", target_key)))?;

        if key.type_ != KEY_AUTHENTICATION_TYPE && key.type_ != KEY_TYPE {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidDIDDoc, format!("DIDDoc validation failed: Unsupported Authentication type: {:?}", key.type_)));
        }

        Ok(())
    }

    pub fn resolve_keys(&self) -> (Vec<String>, Vec<String>) {
        let service: Service = match self.service.get(0).cloned() {
            Some(service) => service,
            None => return (Vec::new(), Vec::new())
        };

        let recipient_keys: Vec<String> =
            service.recipient_keys
                .iter()
                .map(|key| self.key_for_reference(key))
                .collect();

        let routing_keys: Vec<String> =
            service.routing_keys
                .iter()
                .map(|key| self.key_for_reference(key))
                .collect();

        (recipient_keys, routing_keys)
    }

    pub fn recipient_keys(&self) -> Vec<String> {
        let (recipient_keys, _) = self.resolve_keys();
        recipient_keys
    }

    pub fn routing_keys(&self) -> Vec<String> {
        let (_, routing_keys) = self.resolve_keys();
        routing_keys
    }

    pub fn get_endpoint(&self) -> String {
        match self.service.get(0) {
            Some(service) => service.service_endpoint.to_string(),
            None => String::new()
        }
    }

    fn key_for_reference(&self, key_reference: &str) -> String {
        let id = DidDoc::_parse_key_reference(key_reference);

        self.public_key.iter().find(|key_| key_.id == id.to_string() || key_.public_key_base_58 == id.to_string() || key_.id == key_reference)
            .map(|key| key.public_key_base_58.clone())
            .unwrap_or(id)
    }

    fn _build_key_reference(did: &str, id: &str) -> String {
        format!("{}#{}", did, id)
    }

    fn _key_parts(key: &str) -> Vec<&str> {
        key.split("#").collect()
    }

    fn _parse_key_reference(key_reference: &str) -> String {
        let pars: Vec<&str> = DidDoc::_key_parts(key_reference);
        pars.get(1).or(pars.get(0)).map(|s| s.to_string()).unwrap_or_default()
    }
}

impl Default for Service {
    fn default() -> Service {
        Service {
            // TODO: FIXME Several services????
            id: format!("did:example:123456789abcdefghi;{}", SERVICE_SUFFIX),
            type_: String::from(SERVICE_TYPE),
            priority: 0,
            service_endpoint: String::new(),
            recipient_keys: Vec::new(),
            routing_keys: Vec::new(),
        }
    }
}

impl Service {
    pub fn create() -> Self {
        Service::default()
    }

    pub fn set_id(mut self, id: String)-> Self {
        self.id = id;
        self
    }

    pub fn set_type(mut self, type_: String)-> Self {
        self.type_ = type_;
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Self {
        self.service_endpoint = service_endpoint;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<String>) -> Self {
        self.routing_keys = routing_keys;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<String>) -> Self {
        self.recipient_keys = recipient_keys;
        self
    }

    // extract key from did:key as per method spec: https://w3c-ccg.github.io/did-method-key/
    fn extract_key_from_did_key(key: &str) -> VcxResult<String> {
        debug!("Extracting public key from key reference: {}", key);
        let mut split = key.split(&['#', ':'][..]);
        let identifier = split.nth(2)
            .ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidRedirectDetail,
                                              format!("Invalid Service Key: key format unrecognized: {}.`", key)))?;
        let decoded = identifier[1..].from_base58()
            .map_err(|_| VcxError::from_msg(VcxErrorKind::InvalidRedirectDetail,
                                                          format!("Invalid Service Key: unable to decode key body: {}.`", key)))?;
        // Only ed25519 public keys are currently supported
        if decoded[0] == 0xED {
            // for ed25519, multicodec should be 2 bytes long (0xed01). Dropping this should yield the raw key bytes
            Ok(decoded[2..].to_base58())
        } else{
            Err(VcxError::from_msg(VcxErrorKind::InvalidRedirectDetail,
                                   format!("Invalid Service Key: Multicodec identifier is either not supported or not recognized. Expected: 0xED01, Found: {} in key {}.`", decoded[0], key)))
        }
    }

    pub fn transform_did_keys_to_naked_keys(keys: &mut [String]) -> VcxResult<()> {
        for key in keys.iter_mut() {
            if key.starts_with("did:key"){
                *key = Service::extract_key_from_did_key(key)?
            }
        }
        Ok(())
    }
}

impl From<Invitation> for DidDoc {
    fn from(invite: Invitation) -> DidDoc {
        let mut did_doc: DidDoc = DidDoc::default();
        did_doc.set_id(invite.id.0.clone()); // TODO: FIXME DIDDoc id always MUST be a valid DID
        did_doc.set_service_endpoint(invite.service_endpoint.clone());
        did_doc.set_keys(invite.recipient_keys, invite.routing_keys);
        did_doc
    }
}

impl From<DidDoc> for Invitation {
    fn from(did_doc: DidDoc) -> Invitation {
        let (recipient_keys, routing_keys) = did_doc.resolve_keys();

        Invitation::create()
            .set_id(did_doc.id.clone())
            .set_service_endpoint(did_doc.get_endpoint())
            .set_recipient_keys(recipient_keys)
            .set_routing_keys(routing_keys)
    }
}

impl From<Service> for DidDoc {
    fn from(service: Service) -> DidDoc {
        let mut did_doc: DidDoc = DidDoc::default();
        did_doc.set_service_endpoint(service.service_endpoint);
        did_doc.set_keys(service.recipient_keys, service.routing_keys);
        did_doc
    }
}

impl From<Service> for Invitation {
    fn from(service: Service) -> Invitation {
        Invitation::create()
            .set_id(service.id)
            .set_service_endpoint(service.service_endpoint)
            .set_recipient_keys(service.recipient_keys)
            .set_routing_keys(service.routing_keys)
    }
}

impl From<OutofbandInvitation> for DidDoc {
    fn from(invite: OutofbandInvitation) -> DidDoc {
        match invite {
            OutofbandInvitation::V10(mut invitation) => DidDoc::from(invitation.service.swap_remove(0)),
            OutofbandInvitation::V11(mut invitation) => DidDoc::from(invitation.services.swap_remove(0)),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::a2a::MessageId;
    use crate::aries::messages::connection::invite::tests::_invitation;

    pub fn _key_1() -> String {
        String::from("GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL")
    }

    pub fn _key_2() -> String {
        String::from("Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR")
    }

    pub fn _key_3() -> String {
        String::from("3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU")
    }

    pub fn _did_key_1() -> String { String::from("did:key:z6MkukGVb3mRvTu1msArDKY9UwxeZFGjmwnCKtdQttr4Fk6i")}

    pub fn _did_key_2() -> String { String::from("did:key:z6Mkw7FfEGiwh6YQbCLTNbJWAYR8boGNMt7PCjh35GLNxmMo")}

    pub fn _did_key_3() -> String { String::from("did:key:z6MkgnoxYYRk6LAgiR9RkZhnr8mBJCpso2M14zWsTJkAFMwr")}

    pub fn _id() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    pub fn _service_endpoint() -> String {
        String::from("http://localhost:8080")
    }

    pub fn _recipient_keys() -> Vec<String> {
        vec![_key_1()]
    }

    pub fn _recipient_did_keys() -> Vec<String> {
        vec![_did_key_1()]
    }

    pub fn _routing_keys() -> Vec<String> {
        vec![_key_2(), _key_3()]
    }

    pub fn _routing_did_keys() -> Vec<String> { vec![_did_key_2(), _did_key_3()]}

    pub fn _key_reference_1() -> String {
        DidDoc::_build_key_reference(&_id(), "1")
    }

    pub fn _key_reference_2() -> String {
        DidDoc::_build_key_reference(&_id(), "2")
    }

    pub fn _key_reference_3() -> String {
        DidDoc::_build_key_reference(&_id(), "3")
    }

    pub fn _label() -> String {
        String::from("test")
    }

    pub fn _service() -> Service {
        Service {
            id: _id(),
            type_: "".to_string(),
            priority: 0,
            recipient_keys: _recipient_keys(),
            service_endpoint: _service_endpoint(),
            routing_keys: _routing_keys(),
        }
    }

    pub fn _service_did_formatted() -> Service {
        Service {
            id: _id(),
            type_: "".to_string(),
            priority: 0,
            recipient_keys: _recipient_did_keys(),
            service_endpoint: _service_endpoint(),
            routing_keys: _routing_did_keys(),
        }
    }

    pub fn _did_doc_old() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: "1".to_string(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_reference_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_didkey_formatted() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![_service_did_formatted()],
        }
    }

    pub fn _did_doc_full() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_reference_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_2() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
                Ed25519PublicKey { id: _key_reference_2(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_2() },
                Ed25519PublicKey { id: _key_reference_3(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_3() }
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_3() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_2() },
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_3() }
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_4() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_5() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    #[test]
    fn test_did_doc_build_works() {
        let mut did_doc: DidDoc = DidDoc::default();
        did_doc.set_id(_id());
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(_did_doc(), did_doc);
    }

    #[test]
    fn test_did_doc_validate_works() {
        _did_doc().validate().unwrap();
        _did_doc_old().validate().unwrap();
        _did_doc_full().validate().unwrap();
        _did_doc_2().validate().unwrap();
        _did_doc_3().validate().unwrap();
        _did_doc_4().validate().unwrap();
        _did_doc_5().validate().unwrap();
    }

    #[test]
    fn test_did_doc_key_for_reference_works() {
        assert_eq!(_key_1(), _did_doc_full().key_for_reference(&_key_reference_1()));
    }

    #[test]
    fn test_did_doc_old_key_for_reference_works() {
        assert_eq!(_key_1(), _did_doc_old().key_for_reference(&_key_reference_1()));
    }

    #[test]
    fn test_did_doc_resolve_keys_works() {
        let (recipient_keys, routing_keys) = _did_doc_full().resolve_keys();
        assert_eq!(_recipient_keys(), recipient_keys);
        assert_eq!(_routing_keys(), routing_keys);

        let (recipient_keys, routing_keys) = _did_doc_2().resolve_keys();
        assert_eq!(_recipient_keys(), recipient_keys);
        assert_eq!(_routing_keys(), routing_keys);

        let (recipient_keys, routing_keys) = _did_doc_old().resolve_keys();
        assert_eq!(_recipient_keys(), recipient_keys);
        assert_eq!(_routing_keys(), routing_keys);
    }

    #[test]
    fn test_did_doc_build_key_reference_works() {
        assert_eq!(_key_reference_1(), DidDoc::_build_key_reference(&_id(), "1"));
    }

    #[test]
    fn test_did_doc_parse_key_reference_works() {
        assert_eq!(String::from("1"), DidDoc::_parse_key_reference(&_key_reference_1()));
        assert_eq!(_key_1(), DidDoc::_parse_key_reference(&_key_1()));
    }

    #[test]
    fn test_did_doc_from_invitation_works() {
        let mut did_doc = DidDoc::default();
        did_doc.set_id(MessageId::id().0);
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(did_doc, DidDoc::from(_invitation()))
    }

    #[test]
    fn test_transform_did_key_to_did_works() {
        let mut test_keys = _recipient_did_keys();
        Service::transform_did_keys_to_naked_keys(&mut test_keys).unwrap();
        assert_eq!(_key_1(), test_keys[0])
    }

    #[test]
    fn test_extract_key_from_did_key_works() {
        assert_eq!(_key_1(), Service::extract_key_from_did_key(&_did_key_1()).unwrap())
    }

}
