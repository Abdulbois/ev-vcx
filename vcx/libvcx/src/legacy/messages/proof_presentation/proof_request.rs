use serde_json;

use std::collections::HashMap;
use std::vec::Vec;

use crate::utils::validation;
use crate::error::prelude::*;
use crate::utils::libindy::anoncreds;
use crate::aries::messages::connection::service::Service;
use crate::agent::messages::get_message::Message;
use crate::agent::messages::payload::Payloads;
use crate::aries::messages::thread::Thread;
use crate::utils::libindy::anoncreds::proof_request::*;

static PROOF_REQUEST: &str = "PROOF_REQUEST";
static PROOF_DATA: &str = "proof_request_data";
pub const PROOF_REQUEST_V2: &str = "2.0";

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
struct ProofType {
    name: String,
    #[serde(rename = "version")]
    type_version: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
struct ProofTopic {
    mid: u32,
    tid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProofRequestMessage {
    #[serde(rename = "@type")]
    type_header: ProofType,
    #[serde(rename = "@topic")]
    topic: ProofTopic,
    pub proof_request_data: ProofRequest,
    pub msg_ref_id: Option<String>,
    from_timestamp: Option<u64>,
    to_timestamp: Option<u64>,
    pub thread_id: Option<String>,
    pub comment: Option<String>,
    #[serde(rename = "~service")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Service>,
}

impl ProofRequestMessage {
    pub fn create() -> ProofRequestMessage {
        ProofRequestMessage {
            type_header: ProofType {
                name: String::from(PROOF_REQUEST),
                type_version: String::new(),
            },
            topic: ProofTopic {
                tid: 0,
                mid: 0,
            },
            proof_request_data: ProofRequest {
                nonce: String::new(),
                name: String::new(),
                version: String::new(),
                requested_attributes: HashMap::new(),
                requested_predicates: HashMap::new(),
                non_revoked: None,
                ver: None,
            },
            msg_ref_id: None,
            from_timestamp: None,
            to_timestamp: None,
            thread_id: None,
            comment: None,
            service: None,
        }
    }

    pub fn type_version(&mut self, version: &str) -> VcxResult<&mut Self> {
        self.type_header.type_version = String::from(version);
        Ok(self)
    }

    pub fn tid(&mut self, tid: u32) -> VcxResult<&mut Self> {
        self.topic.tid = tid;
        Ok(self)
    }

    pub fn mid(&mut self, mid: u32) -> VcxResult<&mut Self> {
        self.topic.mid = mid;
        Ok(self)
    }

    pub fn nonce(&mut self, nonce: &str) -> VcxResult<&mut Self> {
        let nonce = validation::validate_nonce(nonce)?;
        self.proof_request_data.nonce = nonce;
        Ok(self)
    }

    pub fn proof_name(&mut self, name: &str) -> VcxResult<&mut Self> {
        self.proof_request_data.name = String::from(name);
        Ok(self)
    }

    pub fn proof_request_format_version(&mut self, version: Option<ProofRequestVersion>) -> VcxResult<&mut Self> {
        self.proof_request_data.ver = version;
        Ok(self)
    }

    pub fn proof_data_version(&mut self, version: &str) -> VcxResult<&mut Self> {
        self.proof_request_data.version = String::from(version);
        Ok(self)
    }


    pub fn requested_attrs(&mut self, attrs: &str) -> VcxResult<&mut Self> {
        trace!("ProofRequestMessage::requested_attrs >>> attrs: {:?}", secret!(attrs));

        let mut check_req_attrs: HashMap<String, AttributeInfo> = HashMap::new();
        let proof_attrs: Vec<AttributeInfo> = serde_json::from_str(attrs)
            .map_err(|err| {
                debug!("Cannot parse attributes: {}", err);
                VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, format!("Cannot parse attributes: {}", err))
            })?;

        let mut index = 1;
        for mut attr in proof_attrs.into_iter() {
            let attr_name = match (attr.name.as_ref(), attr.names.as_ref()) {
                (Some(name), None) => { name.clone() }
                (None, Some(names)) => {
                    if names.is_empty() {
                        return Err(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, "Requested Attributes validation failed: there is empty request attribute names"));
                    }
                    names.join(",")
                }
                (Some(_), Some(_)) => {
                    return Err(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure,
                                                  format!("Requested Attributes validation failed: there is empty requested attribute: {:?}", attrs)));
                }
                (None, None) => {
                    return Err(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure,
                                                  format!("Requested Attributes validation failed: there is a requested attribute with both name and names: {:?}", attrs)));
                }
            };

            attr.restrictions = self.process_restrictions(attr.restrictions);

            if check_req_attrs.contains_key(&attr_name) {
                check_req_attrs.insert(format!("{}_{}", attr_name, index), attr);
            } else {
                check_req_attrs.insert(attr_name, attr);
            }
            index = index + 1;
        }
        self.proof_request_data.requested_attributes = check_req_attrs;
        Ok(self)
    }

    pub fn requested_predicates(&mut self, predicates: &str) -> VcxResult<&mut Self> {
        trace!("ProofRequestMessage::requested_predicates >>> predicates: {:?}", secret!(predicates));

        let mut check_predicates: HashMap<String, PredicateInfo> = HashMap::new();
        let attr_values: Vec<PredicateInfo> = serde_json::from_str(predicates)
            .map_err(|err| {
                debug!("Cannot parse predicates: {}", err);
                VcxError::from_msg(VcxErrorKind::InvalidPredicatesStructure, format!("Cannot parse predicates: {}", err))
            })?;

        let mut index = 1;
        for mut attr in attr_values.into_iter() {
            attr.restrictions = self.process_restrictions(attr.restrictions);

            if check_predicates.contains_key(&attr.name) {
                check_predicates.insert(format!("{}_{}", attr.name, index), attr);
            } else {
                check_predicates.insert(attr.name.clone(), attr);
            }
            index = index + 1;
        }

        self.proof_request_data.requested_predicates = check_predicates;
        Ok(self)
    }

    fn process_restrictions(&self, restrictions: Option<Restrictions>) -> Option<Restrictions> {
        match restrictions {
            Some(Restrictions::V2(restrictions)) => Some(Restrictions::V2(restrictions)),
            Some(Restrictions::V1(restrictions)) => {
                Some(Restrictions::V1(
                    restrictions
                        .into_iter()
                        .map(|filter| {
                            Filter {
                                schema_id: filter.schema_id.as_ref().and_then(|schema_id| anoncreds::libindy_to_unqualified(&schema_id).ok()),
                                schema_issuer_did: filter.schema_issuer_did.as_ref().and_then(|schema_issuer_did| anoncreds::libindy_to_unqualified(&schema_issuer_did).ok()),
                                schema_name: filter.schema_name,
                                schema_version: filter.schema_version,
                                issuer_did: filter.issuer_did.as_ref().and_then(|issuer_did| anoncreds::libindy_to_unqualified(&issuer_did).ok()),
                                cred_def_id: filter.cred_def_id.as_ref().and_then(|cred_def_id| anoncreds::libindy_to_unqualified(&cred_def_id).ok()),
                            }
                        })
                        .collect()
                ))
            }
            None => None
        }
    }

    pub fn from_timestamp(&mut self, from: Option<u64>) -> VcxResult<&mut Self> {
        self.from_timestamp = from;
        Ok(self)
    }

    pub fn to_timestamp(&mut self, to: Option<u64>) -> VcxResult<&mut Self> {
        self.to_timestamp = to;
        Ok(self)
    }

    pub fn set_proof_request_data(&mut self, proof_request_data: ProofRequest) -> VcxResult<&mut Self> {
        self.proof_request_data = proof_request_data;
        Ok(self)
    }


    pub fn set_thread_id(&mut self, thid: String) -> VcxResult<&mut Self> {
        self.thread_id = Some(thid);
        Ok(self)
    }

    pub fn set_service(&mut self, service: Option<Service>) -> VcxResult<&mut Self> {
        self.service = service;
        Ok(self)
    }

    pub fn set_comment(&mut self, comment: Option<String>) -> VcxResult<&mut Self> {
        self.comment = comment;
        Ok(self)
    }

    pub fn get_proof_request_data(&self) -> String {
        json!(self)[PROOF_DATA].to_string()
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize proof request: {}", err)))
    }
}

pub fn set_proof_req_ref_message(request: &str, thread: Option<Thread>, msg_id: &str) -> VcxResult<ProofRequestMessage> {
    trace!("set_proof_req_ref_message >>> request: {:?}, msg_id: {:?}", secret!(request), msg_id);

    let mut request: ProofRequestMessage = serde_json::from_str(&request)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest, format!("Cannot deserialize proof request: {}", err)))?;

    request.msg_ref_id = Some(msg_id.to_owned());
    request.thread_id = thread.and_then(|tr| tr.thid.clone());

    trace!("set_proof_req_ref_message <<< request: {:?}", secret!(request));

    Ok(request)
}

pub fn parse_proof_req_message(message: &Message, my_vk: &str) -> VcxResult<ProofRequestMessage> {
    trace!("parse_proof_req_message >>> message: {:?}, my_vk: {:?}", secret!(message), secret!(my_vk));

    let payload = message.payload.as_ref()
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidProofRequest, "Message does not contain payload"))?;

    let (request, thread) = Payloads::decrypt(&my_vk, payload)?;

    let mut request: ProofRequestMessage = serde_json::from_str(&request)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest, format!("Cannot deserialize proof request: {}", err)))?;

    request.msg_ref_id = Some(message.uid.to_owned());
    request.thread_id = thread.and_then(|tr| tr.thid.clone());

    trace!("set_proof_req_ref_message <<< request: {:?}", secret!(request));

    Ok(request)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES};
    use crate::utils::devsetup::SetupDefaults;
    use crate::agent::messages::proof_request;

    #[test]
    fn test_create_proof_request_data() {
        let _setup = SetupDefaults::init();

        let request = proof_request();
        let proof_data = ProofRequest {
            nonce: String::new(),
            name: String::new(),
            version: String::new(),
            requested_attributes: HashMap::new(),
            requested_predicates: HashMap::new(),
            non_revoked: None,
            ver: None,
        };
        assert_eq!(request.proof_request_data, proof_data);
    }

    #[test]
    fn test_proof_request_msg() {
        let _setup = SetupDefaults::init();

        //proof data
        let data_name = "Test";
        let nonce = "123432421212";
        let data_version = "3.75";
        let version = "1.3";
        let tid = 89;
        let mid = 98;

        let request = proof_request()
            .type_version(version).unwrap()
            .tid(tid).unwrap()
            .mid(mid).unwrap()
            .nonce(nonce).unwrap()
            .proof_request_format_version(Some(ProofRequestVersion::V2)).unwrap()
            .proof_name(data_name).unwrap()
            .proof_data_version(data_version).unwrap()
            .requested_attrs(REQUESTED_ATTRS).unwrap()
            .requested_predicates(REQUESTED_PREDICATES).unwrap()
            .to_timestamp(Some(100)).unwrap()
            .from_timestamp(Some(1)).unwrap()
            .clone();

        let serialized_msg = request.to_string().unwrap();
        assert!(serialized_msg.contains(r#""@type":{"name":"PROOF_REQUEST","version":"1.3"}"#));
        assert!(serialized_msg.contains(r#"@topic":{"mid":98,"tid":89}"#));
        assert!(serialized_msg.contains(r#"proof_request_data":{"nonce":"123432421212","name":"Test","version":"3.75","requested_attributes""#));

        assert!(serialized_msg.contains(r#""age":{"name":"age","restrictions":[{"schema_id":"6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11","schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB","schema_name":"Faber Student Info","schema_version":"1.0","issuer_did":"8XFh8yBzrpJQmNyZzgoTqB","cred_def_id":"8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766"},{"schema_id":"5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11","schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB","schema_name":"BYU Student Info","schema_version":"1.0","issuer_did":"66Fh8yBzrpJQmNyZzgoTqB","cred_def_id":"66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}"#));
        assert!(serialized_msg.contains(r#""to_timestamp":100"#));
        assert!(serialized_msg.contains(r#""from_timestamp":1"#));
        assert!(serialized_msg.contains(r#""ver":"2.0""#));
    }

    #[test]
    fn test_requested_attrs_constructed_correctly() {
        let _setup = SetupDefaults::init();

        let mut check_req_attrs: HashMap<String, AttributeInfo> = HashMap::new();
        let attr_info1: AttributeInfo = serde_json::from_str(r#"{ "name":"age", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }"#).unwrap();
        let attr_info2: AttributeInfo = serde_json::from_str(r#"{ "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }"#).unwrap();

        check_req_attrs.insert("age".to_string(), attr_info1);
        check_req_attrs.insert("name".to_string(), attr_info2);

        let request = proof_request().requested_attrs(REQUESTED_ATTRS).unwrap().clone();
        assert_eq!(request.proof_request_data.requested_attributes, check_req_attrs);
    }

    #[test]
    fn test_requested_predicates_constructed_correctly() {
        let _setup = SetupDefaults::init();

        let mut check_predicates: HashMap<String, PredicateInfo> = HashMap::new();
        let attr_info1: PredicateInfo = serde_json::from_str(r#"{ "name":"age","p_type":"GE","p_value":22, "restrictions":[ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }"#).unwrap();
        check_predicates.insert("age".to_string(), attr_info1);

        let request = proof_request().requested_predicates(REQUESTED_PREDICATES).unwrap().clone();
        assert_eq!(request.proof_request_data.requested_predicates, check_predicates);
    }

    #[test]
    fn test_requested_attrs_constructed_correctly_for_names() {
        let _setup = SetupDefaults::init();

        let attr_info = json!({ "names":["name", "age", "email"], "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11" } ] });
        let attr_info_2 = json!({ "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11" } ] });

        let requested_attrs = json!([ attr_info, attr_info_2 ]).to_string();

        let request = proof_request().requested_attrs(&requested_attrs).unwrap().clone();

        let mut expected_req_attrs: HashMap<String, AttributeInfo> = HashMap::new();
        expected_req_attrs.insert("name,age,email".to_string(), serde_json::from_value(attr_info).unwrap());
        expected_req_attrs.insert("name".to_string(), serde_json::from_value(attr_info_2).unwrap());

        assert_eq!(request.proof_request_data.requested_attributes, expected_req_attrs);
    }

    #[test]
    fn test_requested_attrs_constructed_correctly_for_name_and_names_passed_together() {
        let _setup = SetupDefaults::init();

        let attr_info = json!({ "name":"name", "names":["name", "age", "email"], "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11" } ] });

        let requested_attrs = json!([ attr_info ]).to_string();

        let err = proof_request().requested_attrs(&requested_attrs).unwrap_err();
        assert_eq!(VcxErrorKind::InvalidAttributesStructure, err.kind());
    }

    #[test]
    fn test_indy_proof_req_parses_correctly() {
        let _setup = SetupDefaults::init();

        let _proof_req: ProofRequestMessage = serde_json::from_str(r#"{"@type":{"name":"PROOF_REQUEST","version":"1.0"},"@topic":{"mid":0,"tid":0},"proof_request_data":{"nonce":"14485060341131021134890","name":"proof_from_alice","version":"0.1","requested_attributes":{"first_name":{"name":"first_name","restrictions":[{"schema_id":null,"schema_issuer_did":null,"schema_name":null,"schema_version":null,"issuer_did":"V4SGRU86Z58d6TV7PBUe6f","cred_def_id":null}]},"last_name":{"name":"last_name","restrictions":[{"schema_id":null,"schema_issuer_did":null,"schema_name":null,"schema_version":null,"issuer_did":"V4SGRU86Z58d6TV7PBUe6f","cred_def_id":null}]},"email":{"name":"email","restrictions":[{"schema_id":null,"schema_issuer_did":null,"schema_name":null,"schema_version":null,"issuer_did":"V4SGRU86Z58d6TV7PBUe6f","cred_def_id":null}]}},"requested_predicates":{},"non_revoked":null,"ver":null},"msg_ref_id":null,"from_timestamp":null,"to_timestamp":null,"thread_id":null}"#).unwrap();
    }
}
