use std::collections::HashMap;

use crate::error::prelude::*;
use crate::utils::qualifier;
use crate::utils::libindy::anoncreds;
use crate::utils::libindy::anoncreds::verifier::Verifier;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProofRequest {
    pub nonce: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub requested_attributes: HashMap<String, AttributeInfo>,
    #[serde(default)]
    pub requested_predicates: HashMap<String, PredicateInfo>,
    pub non_revoked: Option<NonRevokedInterval>,
    pub ver: Option<ProofRequestVersion>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AttributeInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<Restrictions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_revoked: Option<NonRevokedInterval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_attest_allowed: Option<bool>,
}

impl AttributeInfo {
    pub fn self_attest_allowed(&self) -> bool {
        if self.names.is_some() {
            return false;
        }

        match (self.self_attest_allowed, self.restrictions.as_ref()) {
            (Some(true), Some(restrictions)) => self.check_restrictions(restrictions),
            (Some(true), None) => true,
            (Some(false), Some(_)) => false,
            (Some(false), None) => false,
            (None, Some(restrictions)) => self.check_restrictions(restrictions),
            (None, None) => true
        }
    }

    fn check_restrictions(&self, restrictions: &Restrictions) -> bool {
        match restrictions {
            Restrictions::V1(restrictions) => {
                if restrictions.is_empty() {
                    return true;
                }
                restrictions
                    .iter()
                    .all(|restriction| {
                        if restriction.schema_id.is_some() ||
                            restriction.schema_issuer_did.is_some() ||
                            restriction.schema_name.is_some() ||
                            restriction.schema_version.is_some() ||
                            restriction.issuer_did.is_some() ||
                            restriction.cred_def_id.is_some() {
                            return false;
                        }
                        return true;
                    })
            },
            Restrictions::V2(restrictions) => {
                match restrictions {
                    serde_json::Value::Object(object) => object.is_empty(),
                    serde_json::Value::Array(array) => {
                        if array.is_empty() {
                            return true;
                        }
                        array
                            .iter()
                            .all(|item| match item {
                                serde_json::Value::Object(object) => object.is_empty(),
                                _ => false,
                            })
                    }
                    _ => return false
                }
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PredicateInfo {
    pub name: String,
    pub p_type: String,
    pub p_value: i32,
    pub restrictions: Option<Restrictions>,
    pub non_revoked: Option<NonRevokedInterval>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Filter {
    pub schema_id: Option<String>,
    pub schema_issuer_did: Option<String>,
    pub schema_name: Option<String>,
    pub schema_version: Option<String>,
    pub issuer_did: Option<String>,
    pub cred_def_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Restrictions {
    V1(Vec<Filter>),
    V2(::serde_json::Value),
}

impl ProofRequest {
    const DEFAULT_VERSION: &'static str = "1.0";

    pub fn create() -> ProofRequest {
        ProofRequest::default()
    }

    pub fn set_name(mut self, name: String) -> ProofRequest {
        self.name = name;
        self
    }

    pub fn set_version(mut self, version: String) -> ProofRequest {
        self.version = version;
        self
    }

    pub fn set_format_version(mut self, version: ProofRequestVersion) -> ProofRequest {
        self.ver = Some(version);
        self
    }

    pub fn set_nonce(mut self) -> VcxResult<ProofRequest> {
        self.nonce = Verifier::generate_nonce()?;
        Ok(self)
    }

    pub fn set_requested_attributes(self, requested_attrs: String) -> VcxResult<ProofRequest> {
        trace!("set_requested_attributes >>> requested_attrs: {:?}", secret!(requested_attrs));

        let requested_attributes: Vec<AttributeInfo> = ::serde_json::from_str(&requested_attrs)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, format!("Invalid Requested Attributes: {:?}. Err: {:?}", requested_attrs, err)))?;

        Ok(self.set_requested_attributes_value(requested_attributes))
    }

    pub fn set_requested_attributes_value(mut self, requested_attrs: Vec<AttributeInfo>) -> ProofRequest {
        trace!("set_requested_attributes_value >>> requested_attrs: {:?}", secret!(requested_attrs));

        self.requested_attributes = requested_attrs
            .into_iter()
            .enumerate()
            .map(|(index, attribute)| (format!("attribute_{}", index), attribute))
            .collect();
        self
    }

    pub fn set_requested_predicates(self, requested_predicates: String) -> VcxResult<ProofRequest> {
        trace!("set_requested_predicates >>> requested_predicates: {:?}", secret!(requested_predicates));

        let requested_predicates: Vec<PredicateInfo> = ::serde_json::from_str(&requested_predicates)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidPredicatesStructure, format!("Invalid Requested Predicates: {:?}, err: {:?}", requested_predicates, err)))?;

        Ok(self.set_requested_predicates_value(requested_predicates))
    }

    pub fn set_requested_predicates_value(mut self, requested_predicates: Vec<PredicateInfo>) -> ProofRequest {
        trace!("set_requested_predicates_value >>> requested_predicates: {:?}", secret!(requested_predicates));

        self.requested_predicates = requested_predicates
            .into_iter()
            .enumerate()
            .map(|(index, attribute)| (format!("predicate_{}", index), attribute))
            .collect();
        self
    }

    pub fn set_not_revoked_interval(mut self, non_revoc_interval: String) -> VcxResult<ProofRequest> {
        trace!("set_not_revoked_interval >>> non_revoc_interval: {:?}", secret!(non_revoc_interval));

        let non_revoc_interval: NonRevokedInterval = ::serde_json::from_str(&non_revoc_interval)
            .map_err(|_| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Revocation Interval: {:?}", non_revoc_interval)))?;

        self.non_revoked = match (non_revoc_interval.from, non_revoc_interval.to) {
            (None, None) => None,
            (from, to) => Some(NonRevokedInterval { from, to })
        };

        Ok(self)
    }

    pub fn set_format_version_for_did(mut self, my_did: &str, remote_did: &str) -> VcxResult<ProofRequest> {
        if qualifier::is_fully_qualified(&my_did) && qualifier::is_fully_qualified(&remote_did) {
            self.ver = Some(ProofRequestVersion::V2)
        } else {
            let proof_request_json = serde_json::to_string(&self)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize ProofRequestData: {:?}", err)))?;

            let proof_request_json = anoncreds::libindy_to_unqualified(&proof_request_json)?;

            self = serde_json::from_str(&proof_request_json)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest, format!("Cannot deserialize ProofRequestData: {:?}", err)))?;

            self.ver = Some(ProofRequestVersion::V1)
        }
        Ok(self)
    }

    pub fn get_revocation_interval(&self, attr_name: &str) -> VcxResult<Option<NonRevokedInterval>> {
        if let Some(attr) = self.requested_attributes.get(attr_name) {
            Ok(attr.non_revoked.clone().or(self.non_revoked.clone().or(None)))
        } else if let Some(attr) = self.requested_predicates.get(attr_name) {
            // Handle case for predicates
            Ok(attr.non_revoked.clone().or(self.non_revoked.clone().or(None)))
        } else {
            Err(VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData,
                                   format!("RevocationInterval not found for: {}", attr_name)))
        }
    }
}

impl Default for ProofRequest {
    fn default() -> ProofRequest {
        ProofRequest {
            nonce: String::new(),
            name: String::new(),
            version: String::from(ProofRequest::DEFAULT_VERSION),
            requested_attributes: HashMap::new(),
            requested_predicates: HashMap::new(),
            non_revoked: None,
            ver: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ProofRequestVersion {
    #[serde(rename = "1.0")]
    V1,
    #[serde(rename = "2.0")]
    V2,
}

impl Default for ProofRequestVersion {
    fn default() -> ProofRequestVersion {
        ProofRequestVersion::V1
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct NonRevokedInterval {
    pub from: Option<u64>,
    pub to: Option<u64>,
}