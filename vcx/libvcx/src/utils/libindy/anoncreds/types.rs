use std::collections::HashMap;

use crate::utils::libindy::anoncreds::proof_request::NonRevokedInterval;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CredentialOffer {
    pub schema_id: String,
    pub cred_def_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CredentialInfo {
    pub referent: String,
    pub attrs: HashMap<String, String>,
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Credential {
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub values: CredentialValues,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialValues(pub HashMap<String, AttributeValues>);

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct AttributeValues {
    pub raw: String,
    pub encoded: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CredentialDefinitionData {
    id: String,
    #[serde(rename = "schemaId")]
    schema_id: String,
    #[serde(rename = "type")]
    type_: String,
    value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CredentialsForProofRequest {
    V1(CredentialsForProofRequestV1),
    V2(CredentialsForProofRequestV2),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialsForProofRequestV1 {
    pub attrs: HashMap<String, Vec<SelectedCredentialInfoWithValue>>,
}

impl CredentialsForProofRequestV1 {
    pub fn new() -> CredentialsForProofRequestV1 {
        CredentialsForProofRequestV1 {
            attrs: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialsForProofRequestV2 {
    pub attributes: HashMap<String, CredentialsForProofRequestV2Attribute>,
    pub predicates: HashMap<String, CredentialsForProofRequestV2Predicate>,
}

impl CredentialsForProofRequestV2 {
    pub fn new() -> CredentialsForProofRequestV2 {
        CredentialsForProofRequestV2 {
            attributes: HashMap::new(),
            predicates: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialsForProofRequestV2Attribute {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    pub credentials: Vec<SelectedCredentialInfoWithValue>,
    pub self_attest_allowed: bool,
    pub missing: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialsForProofRequestV2Predicate {
    pub name: String,
    pub p_type: String,
    pub p_value: i32,
    pub credentials: Vec<SelectedCredentialInfoWithValue>,
    pub missing: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialsSelectedForProofRequest {
    #[serde(default)]
    pub attrs: HashMap<String, SelectedCredential>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelectedCredential {
    pub credential: SelectedCredentialInfo,
    pub tails_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelectedCredentialInfo {
    pub cred_info: CredentialInfo,
    pub interval: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelectedCredentialInfoWithValue {
    pub cred_info: CredentialInfo,
    pub interval: Option<serde_json::Value>,
    pub requested_attributes: HashMap<String, String>,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct RequestedCredentials {
    pub self_attested_attributes: HashMap<String, String>,
    pub requested_attributes: HashMap<String, RequestedAttribute>,
    pub requested_predicates: HashMap<String, ProvingCredentialKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestedAttribute {
    pub cred_id: String,
    pub timestamp: Option<u64>,
    pub revealed: bool,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Clone)]
pub struct ProvingCredentialKey {
    pub cred_id: String,
    pub timestamp: Option<u64>,
}

impl RequestedCredentials {
    pub fn new() -> RequestedCredentials {
        RequestedCredentials {
            self_attested_attributes: HashMap::new(),
            requested_attributes: HashMap::new(),
            requested_predicates: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExtendedCredentialInfo {
    pub requested_attr: String,
    pub referent: String,
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<String>,
    pub revocation_interval: Option<NonRevokedInterval>,
    pub tails_file: Option<String>,
    pub timestamp: Option<u64>,
}