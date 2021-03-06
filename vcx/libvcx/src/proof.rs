use ::serde_json;
use serde_json::Value;
use openssl;
use openssl::bn::{BigNum, BigNumRef};
use std::convert::TryInto;

use crate::connection::Connections;
use crate::settings;
use crate::api::{VcxStateType, ProofStateType};
use crate::agent;
use crate::legacy::messages::proof_presentation::proof_message::{ProofMessage, CredInfo};
use crate::agent::messages::{RemoteMessageType, GeneralMessage};
use crate::agent::messages::payload::{Payloads, PayloadKinds};
use crate::aries::messages::thread::Thread;
use crate::agent::messages::get_message::get_ref_msg;
use crate::legacy::messages::proof_presentation::proof_request::ProofRequestMessage;
use crate::utils::error;
use crate::utils::constants::*;
use crate::utils::libindy::anoncreds::verifier::Verifier as IndyVerifier;
use crate::utils::libindy::ledger;
use crate::utils::object_cache::{ObjectCache, Handle};
use crate::error::prelude::*;
use crate::utils::openssl::encode;
use crate::utils::qualifier;
use crate::legacy::messages::proof_presentation::proof_message::get_credential_info;

use crate::aries::handlers::proof_presentation::verifier::Verifier;
use crate::agent::agent_info::{get_agent_info, MyAgentInfo, get_agent_attr};
use crate::aries::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::utils::libindy::ledger::query::Query;
use crate::utils::libindy::anoncreds::proof_request::ProofRequestVersion;


lazy_static! {
    static ref PROOF_MAP: ObjectCache<Proofs> = Default::default();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
pub enum Proofs {
    #[serde(rename = "3.0")]
    Pending(Proof),
    #[serde(rename = "1.0")]
    V1(Proof),
    #[serde(rename = "2.0")]
    V3(Verifier),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct RevocationInterval {
    from: Option<u64>,
    to: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Proof {
    source_id: String,
    requested_attrs: String,
    requested_predicates: String,
    msg_uid: String,
    ref_msg_id: String,
    state: VcxStateType,
    proof_state: ProofStateType,
    name: String,
    version: String,
    nonce: String,
    proof: Option<ProofMessage>,
    // Refactoring this name to 'proof_message' causes some tests to fail.
    proof_request: Option<ProofRequestMessage>,
    #[serde(rename = "prover_did")]
    my_did: Option<String>,
    #[serde(rename = "prover_vk")]
    my_vk: Option<String>,
    #[serde(rename = "remote_did")]
    their_did: Option<String>,
    #[serde(rename = "remote_vk")]
    their_vk: Option<String>,
    agent_did: Option<String>,
    agent_vk: Option<String>,
    revocation_interval: RevocationInterval,
    thread: Option<Thread>,
}

impl Proof {
    pub fn create(source_id: String,
                  requested_attrs: String,
                  requested_predicates: String,
                  revocation_details: String,
                  name: String) -> VcxResult<Proof> {
        trace!("Proof::create >>> source_id: {}, requested_attrs: {}, requested_predicates: {}, name: {}",
               source_id, secret!(requested_attrs), secret!(requested_predicates), secret!(name));

        // TODO: Get this to actually validate as json, not just check length.
        if requested_attrs.len() <= 0 { return Err(VcxError::from(VcxErrorKind::InvalidAttributesStructure)); }

        let revocation_details: RevocationInterval = serde_json::from_str(&revocation_details)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse RevocationInterval from JSON string. Err: {:?}", err)))?;

        debug!("Proof {}: Creating state object", source_id);

        let mut new_proof = Proof {
            source_id,
            requested_attrs,
            requested_predicates,
            name,
            msg_uid: String::new(),
            ref_msg_id: String::new(),
            state: VcxStateType::VcxStateNone,
            proof_state: ProofStateType::ProofUndefined,
            version: String::from("1.0"),
            nonce: generate_nonce()?,
            proof: None,
            proof_request: None,
            revocation_interval: revocation_details,
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
            thread: Some(Thread::new()),
        };

        new_proof.validate_proof_request()?;

        new_proof.state = VcxStateType::VcxStateInitialized;

        trace!("Proof::create <<<");

        Ok(new_proof)
    }

    // leave this returning a u32 until we actually implement this method to do something
    // other than return success.
    fn validate_proof_request(&self) -> VcxResult<u32> {
        //TODO: validate proof request
        Ok(error::SUCCESS.code_num)
    }


    pub fn validate_proof_revealed_attributes(proof_json: &str) -> VcxResult<()> {
        trace!("Proof::validate_proof_revealed_attributes >>> proof_json: {}", secret!(proof_json));
        debug!("Proof: Validating proof revealed attributes");

        if settings::indy_mocks_enabled() { return Ok(()); }

        let proof: Value = serde_json::from_str(proof_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Cannot parse libndy proof from JSON string. Err: {}", err)))?;

        let revealed_attrs = match proof["requested_proof"]["revealed_attrs"].as_object() {
            Some(revealed_attrs) => revealed_attrs,
            None => return Ok(())
        };

        for (attr1_referent, info) in revealed_attrs.iter() {
            let raw = info["raw"].as_str().ok_or(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Cannot get raw value for \"{}\" attribute", attr1_referent)))?;
            let encoded_ = info["encoded"].as_str().ok_or(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Cannot get encoded value for \"{}\" attribute", attr1_referent)))?;

            let expected_encoded = encode(&raw)?;

            if expected_encoded != encoded_.to_string() {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Encoded values are different. Expected: {}. From Proof: {}", expected_encoded, encoded_)));
            }
        }

        trace!("Proof::validate_proof_revealed_attributes <<<");

        Ok(())
    }

    pub fn build_credential_defs_json(credential_data: &[CredInfo]) -> VcxResult<String> {
        trace!("Proof::build_credential_defs_json >>> credential_data: {:?}", secret!(credential_data));
        debug!("Proof: Building credential definitions for proof validation");
        let mut credential_json = json!({});

        for ref cred_info in credential_data.iter() {
            if credential_json.get(&cred_info.cred_def_id).is_none() {
                let (id, credential_def) = Query::get_cred_def(&cred_info.cred_def_id)?;

                let credential_def = serde_json::from_str(&credential_def)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::CredentialDefinitionNotFound, format!("Cannot parse CredentialDefinition received from the Ledger. Err: {}", err)))?;

                credential_json[id] = credential_def;
            }
        }

        trace!("Proof::build_credential_defs_json >>> cred_defs: {}", secret!(credential_json));

        Ok(credential_json.to_string())
    }

    pub fn build_schemas_json(credential_data: &[CredInfo]) -> VcxResult<String> {
        trace!("Proof::build_schemas_json >>> credential_data: {:?}", secret!(credential_data));
        debug!("Proof: Building schemas for proof validation");

        let mut schemas_json = json!({});

        for cred_info in credential_data {
            if schemas_json.get(&cred_info.schema_id).is_none() {
                let (id, schema_json) = ledger::query::Query::get_schema(&cred_info.schema_id)?;

                let schema_val = serde_json::from_str(&schema_json)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidSchema, format!("Cannot parse Schema received from the Ledger. Err: {}", err)))?;

                schemas_json[id] = schema_val;
            }
        }

        trace!("Proof::build_schemas_json >>> schemas_json: {}", secret!(schemas_json));

        Ok(schemas_json.to_string())
    }

    pub fn build_rev_reg_defs_json(credential_data: &[CredInfo]) -> VcxResult<String> {
        trace!("Proof::build_rev_reg_defs_json >>> credential_data: {:?}", secret!(credential_data));
        debug!("Proof: Building revocation registry definitions for proof validation");

        let mut rev_reg_defs_json = json!({});

        for cred_info in credential_data {
            let rev_reg_id = cred_info
                .rev_reg_id
                .as_ref()
                .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationDetails))?;

            if rev_reg_defs_json.get(rev_reg_id).is_none() {
                let (id, json) = ledger::query::Query::get_rev_reg_def(rev_reg_id)
                    .or(Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails)))?;

                let rev_reg_def_json = serde_json::from_str(&json)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails,
                                                      format!("Cannot parse RevocationRegistryDefinition received from the Ledger. Err: {}", err)))?;

                rev_reg_defs_json[id] = rev_reg_def_json;
            }
        }

        trace!("Proof::build_rev_reg_defs_json >>> rev_reg_defs_json: {}", secret!(rev_reg_defs_json));

        Ok(rev_reg_defs_json.to_string())
    }

    pub fn build_rev_reg_json(credential_data: &[CredInfo]) -> VcxResult<String> {
        trace!("Proof::build_rev_reg_json >>> credential_data: {:?}", secret!(credential_data));
        debug!("Proof: building revocation registries for proof validation");

        let mut rev_regs_json = json!({});

        for cred_info in credential_data {
            let rev_reg_id = cred_info
                .rev_reg_id
                .as_ref()
                .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationDetails))?;

            let timestamp = cred_info
                .timestamp
                .as_ref()
                .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationTimestamp))?;

            if rev_regs_json.get(rev_reg_id).is_none() {
                let (id, json, timestamp) = ledger::query::Query::get_rev_reg(rev_reg_id, timestamp.to_owned())
                    .or(Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails)))?;

                let rev_reg_json: Value = serde_json::from_str(&json)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails,
                                                      format!("Cannot parse RevocationRegistry received from the Ledger. Err: {}", err)))?;

                let rev_reg_json = json!({timestamp.to_string(): rev_reg_json});
                rev_regs_json[id] = rev_reg_json;
            }
        }

        trace!("Proof::build_rev_reg_defs_json >>> rev_regs_json: {}", secret!(rev_regs_json));

        Ok(rev_regs_json.to_string())
    }

    fn build_proof_json(&self) -> VcxResult<String> {
        debug!("Proof {}: Vuilding proof json for proof validation", self.source_id);
        match self.proof {
            Some(ref x) => Ok(x.libindy_proof.clone()),
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Proof object state: `libindy_proof` not found", self.source_id)))?
        }
    }

    fn build_proof_req_json(&self) -> VcxResult<String> {
        debug!("Proof {}: Building proof request json for proof validation", self.source_id);
        match self.proof_request {
            Some(ref x) => Ok(x.get_proof_request_data()),
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Proof object state: `proof_request` not found", self.source_id)))?
        }
    }

    fn proof_validation(&mut self) -> VcxResult<u32> {
        trace!("Proof::proof_validation >>>");
        debug!("Proof {}: Validating received proof", self.source_id);

        let proof_json = self.build_proof_json()?;
        let proof_req_json = self.build_proof_req_json()?;

        let valid = Proof::validate_indy_proof(&proof_json, &proof_req_json).map_err(|err| {
            error!("Error: {}, Proof {} wasn't valid", err, self.source_id);
            self.proof_state = ProofStateType::ProofInvalid;
            err.map(VcxErrorKind::InvalidProof, error::INVALID_PROOF.as_str())
        })?;

        if !valid {
            warn!("indy returned false when validating proof {}", self.source_id);
            self.proof_state = ProofStateType::ProofInvalid;
            return Ok(error::SUCCESS.code_num);
        }

        debug!("Indy validated proof: {}", self.source_id);
        self.proof_state = ProofStateType::ProofValidated;

        trace!("Proof::proof_validation <<< proof_state: {:?}", self.proof_state);

        Ok(error::SUCCESS.code_num)
    }

    pub fn validate_indy_proof(proof_json: &str, proof_req_json: &str) -> VcxResult<bool> {
        trace!("Proof::validate_indy_proof >>> proof_json: {:?}, proof_req_json: {:?}", secret!(proof_json), secret!(proof_req_json));
        debug!("Proof: Validating indy proof");

        if settings::indy_mocks_enabled() { return Ok(true); }

        Proof::validate_proof_revealed_attributes(&proof_json)?;

        let credential_data = get_credential_info(&proof_json)?;

        let credential_defs_json = Proof::build_credential_defs_json(&credential_data)
            .unwrap_or(json!({}).to_string());
        let schemas_json = Proof::build_schemas_json(&credential_data)
            .unwrap_or(json!({}).to_string());
        let rev_reg_defs_json = Proof::build_rev_reg_defs_json(&credential_data)
            .unwrap_or(json!({}).to_string());
        let rev_regs_json = Proof::build_rev_reg_json(&credential_data)
            .unwrap_or(json!({}).to_string());

        debug!("*******\n{}\n********", secret!(credential_defs_json));
        debug!("*******\n{}\n********", secret!(schemas_json));
        debug!("*******\n{}\n********", secret!(proof_json));
        debug!("*******\n{}\n********", secret!(proof_req_json));
        debug!("*******\n{}\n********", secret!(rev_reg_defs_json));
        debug!("*******\n{}\n********", secret!(rev_regs_json));
        let valid = IndyVerifier::verify_proof(proof_req_json,
                                               proof_json,
                                               &schemas_json,
                                               &credential_defs_json,
                                               &rev_reg_defs_json,
                                               &rev_regs_json)?;

        trace!("Proof::validate_indy_proof >>> valid: {:?}", valid);
        Ok(valid)
    }

    fn generate_proof_request_msg(&mut self) -> VcxResult<String> {
        trace!("Proof::generate_proof_request_msg >>>");
        debug!("Proof {}: Generating proof request message", self.source_id);

        let their_did = self.their_did.clone().unwrap_or_default();
        let version = if qualifier::is_fully_qualified(&their_did) {
            Some(ProofRequestVersion::V2)
        } else { None };

        let data_version = "0.1";
        let mut proof_obj = agent::messages::proof_request();
        let proof_request = proof_obj
            .type_version(&self.version)?
            .proof_request_format_version(version)?
            .nonce(&self.nonce)?
            .proof_name(&self.name)?
            .proof_data_version(data_version)?
            .requested_attrs(&self.requested_attrs)?
            .requested_predicates(&self.requested_predicates)?
            .from_timestamp(self.revocation_interval.from)?
            .to_timestamp(self.revocation_interval.to)?
            .to_string()?;

        self.proof_request = Some(proof_obj);

        trace!("Proof::generate_proof_request_msg <<< proof_request: {:?}", secret!(self.proof_request ));

        Ok(proof_request)
    }

    fn send_proof_request(&mut self, connection_handle: Handle<Connections>) -> VcxResult<u32> {
        trace!("Proof::send_proof_request >>> connection_handle: {}", connection_handle);
        debug!("Proof {}: Snding proof request", self.source_id);

        if self.state != VcxStateType::VcxStateInitialized {
            warn!("proof {} has invalid state {} for sending proofRequest", self.source_id, self.state as u32);
            return Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                          format!("Proof object {} has invalid state {} for sending ProofRequest", self.source_id, self.state as u32)));
        }
        let agent_info = get_agent_info()?.pw_info(connection_handle)?;
        apply_agent_info(self, &agent_info);

        let proof_request = self.generate_proof_request_msg()?;

        let response = agent::messages::send_message()
            .to(&agent_info.my_pw_did()?)?
            .to_vk(&agent_info.my_pw_vk()?)?
            .msg_type(&RemoteMessageType::ProofReq)?
            .agent_did(&agent_info.pw_agent_did()?)?
            .agent_vk(&agent_info.pw_agent_vk()?)?
            .set_title(&self.name)?
            .set_detail(&self.name)?
            .version(agent_info.version.clone())?
            .edge_agent_payload(&agent_info.my_pw_vk()?,
                                &agent_info.their_pw_vk()?,
                                &proof_request,
                                PayloadKinds::ProofRequest,
                                self.thread.clone())?
            .send_secure()
            .map_err(|err| err.extend("Cannot send proof request"))?;


        self.msg_uid = response.get_msg_uid()?;
        self.state = VcxStateType::VcxStateOfferSent;

        debug!("Proof {}: Proof request sent", self.source_id);
        trace!("Proof::send_proof_request <<<");

        Ok(error::SUCCESS.code_num)
    }

    fn get_proof(&self) -> VcxResult<String> {
        let proof = self.proof.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Proof object state: `proof` not found", self.source_id)))?;
        Ok(proof.libindy_proof.clone())
    }

    fn get_proof_request_status(&mut self, message: Option<String>) -> VcxResult<u32> {
        trace!("Proof::get_proof_request_status >>> message: {:?}", secret!(message));
        debug!("Proof {}: Updating state", self.source_id);

        if self.state == VcxStateType::VcxStateAccepted {
            return Ok(self.get_state());
        } else if message.is_none() &&
            (self.state != VcxStateType::VcxStateOfferSent || self.msg_uid.is_empty() || self.my_did.is_none()) {
            return Ok(self.get_state());
        }

        let payload = match message {
            None => {
                // Check cloud agent for pending agent
                let (_, message) = get_ref_msg(&self.msg_uid,
                                               &get_agent_attr(&self.my_did)?,
                                               &get_agent_attr(&self.my_vk)?,
                                               &get_agent_attr(&self.agent_did)?,
                                               &get_agent_attr(&self.agent_vk)?)?;

                let (payload, thread) = Payloads::decrypt(
                    &get_agent_attr(&self.my_vk)?,
                    &message,
                )?;

                if let Some(_) = thread {
                    let remote_did = &get_agent_attr(&self.their_did)?;
                    self.thread.as_mut().map(|thread| thread.increment_receiver(&remote_did));
                }

                payload
            }
            Some(ref message) => message.clone(),
        };

        self.proof = match parse_proof_payload(&payload) {
            Err(_) => return Ok(self.get_state()),
            Ok(x) => {
                self.state = x.state.unwrap_or(VcxStateType::VcxStateAccepted);
                Some(x)
            }
        };

        if self.state == VcxStateType::VcxStateAccepted {
            match self.proof_validation() {
                Ok(_) => {
                    if self.proof_state != ProofStateType::ProofInvalid {
                        debug!("Proof format was validated for proof {}", self.source_id);
                        self.proof_state = ProofStateType::ProofValidated;
                    }
                }
                Err(x) => {
                    self.state = VcxStateType::VcxStateRequestReceived;
                    warn!("Proof {} had invalid format with err {}", self.source_id, x);
                    self.proof_state = ProofStateType::ProofInvalid;
                }
            };
        }

        let state = self.get_state();

        trace!("Proof::get_proof_request_status <<< state: {}", state);
        Ok(state)
    }

    fn update_state(&mut self, message: Option<String>) -> VcxResult<u32> {
        trace!("Proof::update_state >>>");
        self.get_proof_request_status(message)
    }

    fn get_state(&self) -> u32 {
        trace!("Proof::get_state >>>");

        let state = self.state as u32;

        debug!("Proof {} is in state {}", self.source_id, self.state as u32);
        trace!("Proof::get_state <<< state: {:?}", state);
        state
    }

    fn get_proof_state(&self) -> u32 {
        trace!("Proof::get_proof_state >>>");

        let state = self.proof_state as u32;

        debug!("Proof {} is in state {}", self.source_id, self.state as u32);
        trace!("Proof::get_proof_state <<< state: {:?}", state);
        state
    }

    fn get_proof_uuid(&self) -> &String { &self.msg_uid }

    fn get_source_id(&self) -> String { self.source_id.to_string() }

    #[cfg(test)]
    fn from_str(data: &str) -> VcxResult<Proof> {
        use crate::agent::messages::ObjectWithVersion;
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Proof>| obj.data)
            .map_err(|err| err.extend("Cannot deserialize Proof"))
    }
}

fn handle_err(err: VcxError) -> VcxError {
    if err.kind() == VcxErrorKind::InvalidHandle {
        VcxError::from(VcxErrorKind::InvalidProofHandle)
    } else {
        err
    }
}

pub fn create_proof(source_id: String,
                    requested_attrs: String,
                    requested_predicates: String,
                    revocation_details: String,
                    name: String) -> VcxResult<Handle<Proofs>> {
    // Initiate proof of new format -- redirect to aries folder
    if settings::is_strict_aries_protocol_set() {
        let verifier = Verifier::create(source_id, requested_attrs, requested_predicates, revocation_details, name)?;
        return PROOF_MAP.add(Proofs::V3(verifier))
            .or(Err(VcxError::from(VcxErrorKind::CreateProof)));
    }

    trace!("create_proof >>> source_id: {}, requested_attrs: {}, requested_predicates: {}, name: {}",
           source_id, secret!(requested_attrs), secret!(requested_predicates), secret!(name));
    debug!("creating proof state object {}", source_id);

    let proof = Proof::create(source_id, requested_attrs, requested_predicates, revocation_details, name)?;

    let handle = PROOF_MAP.add(Proofs::Pending(proof))
        .map_err(|_| (VcxError::from(VcxErrorKind::CreateProof)))?;

    debug!("created proof {} with handle {}", handle.get_source_id().unwrap_or_default(), handle);
    trace!("create_proof <<< handle: {:?}", handle);

    Ok(handle)
}

pub fn create_proof_with_proposal(source_id: String,
                                  name: String,
                                  presentation_proposal: String) -> VcxResult<Handle<Proofs>> {
    debug!("create_proof >>> source_id: {}, name: {}, presentation_proposal: {}", source_id, secret!(name), secret!(presentation_proposal));
    debug!("creating proof state object with presentation proposal");

    let presentation_proposal: PresentationProposal = serde_json::from_str(&presentation_proposal)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse PresentationProposal from JSON string. Err: {:?}", err)))?;

    // let requested_attributes = presentation_proposal.to_proof_request_requested_attributes();
    // let requested_predicates = presentation_proposal.to_proof_request_requested_predicates();
    //
    // let thread = match presentation_proposal.thread {
    //     Some(thread) => Some(thread),
    //     None => Some(Thread::new().set_thid(presentation_proposal.id.to_string()))
    // };
    //
    // let proof = Proof::create(source_id,
    //                           json!(requested_attributes).to_string(),
    //                           json!(requested_predicates).to_string(),
    //                           String::from("{}"),
    //                           name,
    //                           thread)?;

    let verifier = Verifier::create_from_proposal(source_id, presentation_proposal)?;
    PROOF_MAP.add(Proofs::V3(verifier))
        .or(Err(VcxError::from(VcxErrorKind::CreateProof)))

    // PROOF_MAP.add(Proofs::Pending(proof))
    //     .or(Err(VcxError::from(VcxErrorKind::CreateProof)))
}

fn apply_agent_info(proof: &mut Proof, agent_info: &MyAgentInfo) {
    proof.my_did = agent_info.my_pw_did.clone();
    proof.my_vk = agent_info.my_pw_vk.clone();
    proof.their_did = agent_info.their_pw_did.clone();
    proof.their_vk = agent_info.their_pw_vk.clone();
    proof.agent_did = agent_info.pw_agent_did.clone();
    proof.agent_vk = agent_info.pw_agent_vk.clone();
}

pub fn release_all() {
    PROOF_MAP.drain().ok();
}

pub fn from_string(proof_data: &str) -> VcxResult<Handle<Proofs>> {
    let proof: Proofs = serde_json::from_str(proof_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse Proofs object from JSON string. Err: {:?}", err)))?;

    PROOF_MAP.add(proof)
}

impl Handle<Proofs> {
    pub fn is_valid_handle(self) -> bool {
        PROOF_MAP.has_handle(self)
    }

    pub fn update_state(self, message: Option<String>) -> VcxResult<u32> {
        PROOF_MAP.get_mut(self, |obj| {
            match obj {
                Proofs::Pending(obj) => {
                    obj.update_state(message.clone())
                        .or_else(|_| Ok(obj.get_state()))
                }
                Proofs::V1(obj) => {
                    obj.update_state(message.clone())
                        .or_else(|_| Ok(obj.get_state()))
                }
                Proofs::V3(obj) => {
                    obj.update_state(message.as_ref().map(String::as_str))
                }
            }
        }).map_err(handle_err)
    }

    pub fn get_state(self) -> VcxResult<u32> {
        PROOF_MAP.get(self, |obj| {
            match obj {
                Proofs::Pending(obj) => Ok(obj.get_state()),
                Proofs::V1(obj) => Ok(obj.get_state()),
                Proofs::V3(obj) => Ok(obj.state())
            }
        }).map_err(handle_err)
    }

    pub fn get_proof_state(self) -> VcxResult<u32> {
        PROOF_MAP.get(self, |obj| {
            match obj {
                Proofs::Pending(obj) => Ok(obj.get_proof_state()),
                Proofs::V1(obj) => Ok(obj.get_proof_state()),
                Proofs::V3(obj) => Ok(obj.presentation_status())
            }
        }).map_err(handle_err)
    }

    pub fn release(self) -> VcxResult<()> {
        PROOF_MAP.release(self).map_err(handle_err)
    }

    pub fn to_string(self) -> VcxResult<String> {
        PROOF_MAP.get(self, |obj| {
            serde_json::to_string(obj)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize Proof object. Err: {:?}", err)))
        }).map_err(handle_err)
    }

    pub fn get_source_id(self) -> VcxResult<String> {
        PROOF_MAP.get(self, |obj| {
            match obj {
                Proofs::Pending(obj) => Ok(obj.get_source_id()),
                Proofs::V1(obj) => Ok(obj.get_source_id()),
                Proofs::V3(obj) => Ok(obj.get_source_id().to_string())
            }
        }).map_err(handle_err)
    }


    pub fn generate_proof_request_msg(self) -> VcxResult<String> {
        PROOF_MAP.get_mut(self, |obj| {
            match obj {
                Proofs::Pending(obj) => obj.generate_proof_request_msg(),
                Proofs::V1(obj) => obj.generate_proof_request_msg(),
                Proofs::V3(obj) => {
                    obj.generate_presentation_request()?;

                    let presentation_request = obj.get_presentation_request()?;

                    // strict aries protocol is set. Return aries formatted Credential Offers
                    if settings::is_strict_aries_protocol_set() {
                        return Ok(json!(presentation_request).to_string());
                    }

                    let proof_request: ProofRequestMessage = presentation_request.clone().try_into()?;
                    return Ok(json!(proof_request).to_string());
                }
            }
        }).map_err(handle_err)
    }

    pub fn generate_request_attach(self) -> VcxResult<String> {
        PROOF_MAP.get_mut(self, |obj| {
            let (proof, attach) = match obj {
                Proofs::Pending(obj) => {
                    let revocation_details = serde_json::to_string(&obj.revocation_interval)
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize RevocationDetails. Err: {:?}", err)))?;

                    let mut verifier = Verifier::create(obj.source_id.to_string(),
                                                        obj.requested_attrs.to_string(),
                                                        obj.requested_predicates.to_string(),
                                                        revocation_details,
                                                        obj.name.to_string())?;

                    verifier.generate_presentation_request()?;
                    let attach = verifier.get_presentation_request_attach()?;
                    Ok((Proofs::V3(verifier), attach))
                }
                Proofs::V1(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be Pending or V3")),
                Proofs::V3(obj) => {
                    obj.generate_presentation_request()?;
                    let attach = obj.get_presentation_request_attach()?;
                    // TODO: avoid cloning
                    Ok((Proofs::V3(obj.clone()), attach))
                }
            }?;
            *obj = proof;
            Ok(attach)
        }).map_err(handle_err)
    }

    pub fn get_presentation_proposal_request(self) -> VcxResult<String> {
        PROOF_MAP.get(self, |obj| {
            match obj {
                Proofs::Pending(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Proof protocol doesn't support proposals.")),
                Proofs::V1(_) => Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Proof protocol doesn't support proposals.")),
                Proofs::V3(obj) => obj.get_presentation_proposal_request()
            }
        }).map_err(handle_err)
    }

    pub fn send_proof_request(self, connection_handle: Handle<Connections>) -> VcxResult<u32> {
        PROOF_MAP.get_mut(self, |proof| {
            let new_proof = match proof {
                Proofs::Pending(obj) => {
                    // if Aries connection is established --> Convert Pending object to V3 Aries proof
                    if connection_handle.is_aries_connection()? {
                        debug!("converting pending proof into aries object");

                        let revocation_details = serde_json::to_string(&obj.revocation_interval)
                            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize RevocationDetails. Err: {:?}", err)))?;

                        let mut verifier = Verifier::create(obj.source_id.to_string(),
                                                            obj.requested_attrs.to_string(),
                                                            obj.requested_predicates.to_string(),
                                                            revocation_details,
                                                            obj.name.to_string())?;
                        verifier.send_presentation_request(connection_handle)?;

                        Proofs::V3(verifier)
                    } else { // else - Convert Pending object to V1 proof
                        obj.send_proof_request(connection_handle)?;
                        // TODO: avoid cloning
                        Proofs::V1(obj.clone())
                    }
                }
                Proofs::V1(obj) => {
                    obj.send_proof_request(connection_handle)?;
                    // TODO: avoid cloning
                    Proofs::V1(obj.clone())
                }
                Proofs::V3(obj) => {
                    obj.send_presentation_request(connection_handle)?;
                    // TODO: avoid cloning
                    Proofs::V3(obj.clone())
                }
            };
            *proof = new_proof;
            Ok(error::SUCCESS.code_num)
        }).map_err(handle_err)
    }

    pub fn request_proof(self,
                         connection_handle: Handle<Connections>,
                         requested_attrs: String,
                         requested_predicates: String,
                         revocation_details: String,
                         name: String) -> VcxResult<u32> {
        PROOF_MAP.get_mut(self, move |proof| {
            let new_proof = match proof {
                Proofs::Pending(obj) => {
                    // if Aries connection is established --> Convert Pending object to V3 Aries proof
                    if connection_handle.is_aries_connection()? {
                        debug!("converting pending proof into aries object");

                        let revocation_interval = serde_json::to_string(&obj.revocation_interval)
                            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize RevocationDetails. Err: {:?}", err)))?;

                        let mut verifier = Verifier::create(obj.source_id.to_string(),
                                                            obj.requested_attrs.to_string(),
                                                            obj.requested_predicates.to_string(),
                                                            revocation_interval,
                                                            obj.name.to_string())?;
                        verifier.request_proof(connection_handle, requested_attrs.clone(), requested_predicates.clone(), revocation_details.clone(), name.clone())?;

                        Ok(Proofs::V3(verifier))
                    } else { // else - Convert Pending object to V1 proof
                        // obj.send_proof_request(connection_handle)?;
                        // Proofs::V1(obj.clone())
                        Err(VcxError::from(VcxErrorKind::InvalidProofHandle))
                    }
                }
                Proofs::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidProofHandle)),
                Proofs::V3(obj) => {
                    obj.request_proof(connection_handle, requested_attrs, requested_predicates, revocation_details, name)?;
                    // TODO: avoid cloning
                    Ok(Proofs::V3(obj.clone()))
                }
            }?;
            *proof = new_proof;
            Ok(error::SUCCESS.code_num)
        }).map_err(handle_err)
    }

    pub fn get_proof_uuid(self) -> VcxResult<String> {
        PROOF_MAP.get(self, |obj| {
            match obj {
                Proofs::Pending(obj) => Ok(obj.get_proof_uuid().clone()),
                Proofs::V1(obj) => Ok(obj.get_proof_uuid().clone()),
                Proofs::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidProofHandle))
            }
        }).map_err(handle_err)
    }

    pub fn get_proof(self) -> VcxResult<String> {
        PROOF_MAP.get(self, |obj| {
            match obj {
                Proofs::Pending(obj) => obj.get_proof(),
                Proofs::V1(obj) => obj.get_proof(),
                Proofs::V3(obj) => {
                    let presentation = obj.get_presentation()?;

                    // strict aries protocol is set. Return aries formatted Credential Offers
                    if settings::is_strict_aries_protocol_set() {
                        return Ok(json!(presentation).to_string());
                    }

                    let proof: ProofMessage = presentation.clone().try_into()?;
                    Ok(json!(proof).to_string())
                }
            }
        }).map_err(handle_err)
    }


    pub fn set_connection(self, connection_handle: Handle<Connections>) -> VcxResult<u32> {
        PROOF_MAP.get_mut(self, |obj| {
            match obj {
                Proofs::Pending(_) | Proofs::V1(_) =>
                    Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Non-Aries Proofs type doesn't support this action: `set_connection`.")),
                Proofs::V3(obj) => {
                    obj.set_connection(connection_handle)?;
                    Ok(error::SUCCESS.code_num)
                }
            }
        }).map_err(handle_err)
    }

    pub fn get_problem_report_message(self) -> VcxResult<String> {
        PROOF_MAP.get(self, |proof| {
            match proof {
                Proofs::Pending(_) | Proofs::V1(_) => {
                    Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Proof type doesn't support this action: `get_problem_report_message`."))
                }
                Proofs::V3(obj) => {
                    obj.get_problem_report_message()
                }
            }
        }).map_err(handle_err)
    }
}

fn parse_proof_payload(payload: &str) -> VcxResult<ProofMessage> {
    let my_credential_req = ProofMessage::from_str(&payload)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse ProofMessage from JSON string. Err: {}", err)))?;
    Ok(my_credential_req)
}

// TODO: This doesnt feel like it should be here (maybe utils?)
pub fn generate_nonce() -> VcxResult<String> {
    let mut bn = BigNum::new().map_err(|err| VcxError::from_msg(VcxErrorKind::EncodeError, format!("Cannot generate nonce: {}", err)))?;

    BigNumRef::rand(&mut bn, LARGE_NONCE as i32, openssl::bn::MsbOption::MAYBE_ZERO, false)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::EncodeError, format!("Cannot generate nonce: {}", err)))?;
    Ok(bn.to_dec_str()
        .map_err(|err| VcxError::from_msg(VcxErrorKind::EncodeError, format!("Cannot generate nonce: {}", err)))?.to_string())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::connection::tests::build_test_connection;
    use crate::utils::devsetup::*;
    use crate::utils::httpclient::AgencyMock;
    use crate::aries::messages::proof_presentation::presentation_preview::PresentationPreview;

    fn default_agent_info(connection_handle: Option<Handle<Connections>>) -> MyAgentInfo {
        if let Some(h) = connection_handle { get_agent_info().unwrap().pw_info(h).unwrap() } else {
            MyAgentInfo {
                my_pw_did: Some("GxtnGN6ypZYgEqcftSQFnC".to_string()),
                my_pw_vk: Some(VERKEY.to_string()),
                their_pw_did: Some(DID.to_string()),
                their_pw_vk: Some(VERKEY.to_string()),
                pw_agent_did: Some(DID.to_string()),
                pw_agent_vk: Some(VERKEY.to_string()),
                agent_did: DID.to_string(),
                agent_vk: VERKEY.to_string(),
                agency_did: DID.to_string(),
                agency_vk: VERKEY.to_string(),
                version: None,
                connection_handle,
            }
        }
    }

    pub fn create_default_proof(state: Option<VcxStateType>, proof_state: Option<ProofStateType>, connection_handle: Option<Handle<Connections>>) -> Proof {
        let agent_info = if let Some(h) = connection_handle {
            get_agent_info().unwrap().pw_info(h).unwrap()
        } else { default_agent_info(connection_handle) };
        let mut proof = Proof {
            source_id: "12".to_string(),
            msg_uid: String::from("1234"),
            ref_msg_id: String::new(),
            requested_attrs: String::from("[]"),
            requested_predicates: String::from("[]"),
            state: state.unwrap_or(VcxStateType::VcxStateOfferSent),
            proof_state: proof_state.unwrap_or(ProofStateType::ProofUndefined),
            name: String::new(),
            version: String::from("1.0"),
            nonce: generate_nonce().unwrap(),
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
            proof: None,
            proof_request: None,
            revocation_interval: RevocationInterval { from: None, to: None },
            thread: Some(Thread::new()),
        };
        apply_agent_info(&mut proof, &agent_info);
        proof
    }

    fn create_boxed_proof(state: Option<VcxStateType>, proof_state: Option<ProofStateType>, connection_handle: Option<Handle<Connections>>) -> Box<Proof> {
        Box::new(create_default_proof(state, proof_state, connection_handle))
    }

    #[test]
    fn test_create_proof_succeeds() {
        let _setup = SetupMocks::init();

        create_proof("1".to_string(),
                     REQUESTED_ATTRS.to_owned(),
                     REQUESTED_PREDICATES.to_owned(),
                     r#"{"support_revocation":false}"#.to_string(),
                     "Optional".to_owned()).unwrap();
    }

    #[test]
    fn test_revocation_details() {
        let _setup = SetupMocks::init();

        // No Revocation
        create_proof("1".to_string(),
                     REQUESTED_ATTRS.to_owned(),
                     REQUESTED_PREDICATES.to_owned(),
                     r#"{"support_revocation":false}"#.to_string(),
                     "Optional".to_owned()).unwrap();

        // Support Revocation Success
        let revocation_details = json!({
            "to": 1234,
        });
        create_proof("1".to_string(),
                     REQUESTED_ATTRS.to_owned(),
                     REQUESTED_PREDICATES.to_owned(),
                     revocation_details.to_string(),
                     "Optional".to_owned()).unwrap();
    }

    #[test]
    fn test_nonce() {
        let _setup = SetupDefaults::init();

        let nonce = generate_nonce().unwrap();
        assert!(BigNum::from_dec_str(&nonce).unwrap().num_bits() < 81)
    }

    #[test]
    fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        let proof_string = handle.to_string().unwrap();
        let s: Value = serde_json::from_str(&proof_string).unwrap();
        assert_eq!(s["version"], PENDING_OBJECT_SERIALIZE_VERSION);
        assert!(!proof_string.is_empty());
    }

    #[test]
    fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        let proof_data = handle.to_string().unwrap();
        let proof1: Proof = Proof::from_str(&proof_data).unwrap();
        assert!(handle.release().is_ok());

        let new_handle = from_string(&proof_data).unwrap();
        let proof2: Proof = Proof::from_str(&new_handle.to_string().unwrap()).unwrap();
        assert_eq!(proof1, proof2);
    }

    #[test]
    fn test_release_proof() {
        let _setup = SetupMocks::init();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        assert!(handle.release().is_ok());
        assert!(!handle.is_valid_handle());
    }

    #[test]
    fn test_send_proof_request() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();
        connection_handle.set_agent_verkey(VERKEY).unwrap();
        connection_handle.set_agent_did(DID).unwrap();
        connection_handle.set_their_pw_verkey(VERKEY).unwrap();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        assert_eq!(handle.send_proof_request(connection_handle).unwrap(), error::SUCCESS.code_num);
        assert_eq!(handle.get_state().unwrap(), VcxStateType::VcxStateOfferSent as u32);
        assert_eq!(handle.get_proof_uuid().unwrap(), "ntc2ytb");
    }


    #[test]
    fn test_send_proof_request_fails_with_no_pw() {
        //This test has 2 purposes:
        //1. when send_proof_request fails, Ok(c.send_proof_request(connection_handle)?) returns error instead of Ok(_)
        //2. Test that when no PW connection exists, send message fails on invalid did
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();
        connection_handle.set_pw_did("").unwrap();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();

        assert!(handle.send_proof_request(connection_handle).is_err());
    }

    #[test]
    fn test_get_proof_fails_with_no_proof() {
        let _setup = SetupMocks::init();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        assert!(handle.is_valid_handle());
        assert!(handle.get_proof().is_err())
    }

    #[test]
    fn test_update_state_with_pending_proof() {
        let _setup = SetupMocks::init();

        let connection_h = Some(build_test_connection());
        let mut proof = Proof {
            source_id: "12".to_string(),
            msg_uid: String::from("1234"),
            ref_msg_id: String::new(),
            requested_attrs: String::from("[]"),
            requested_predicates: String::from("[]"),
            state: VcxStateType::VcxStateOfferSent,
            proof_state: ProofStateType::ProofUndefined,
            name: String::new(),
            version: String::from("1.0"),
            nonce: generate_nonce().unwrap(),
            proof: None,
            proof_request: None,
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
            revocation_interval: RevocationInterval { from: None, to: None },
            thread: Some(Thread::new()),
        };

        apply_agent_info(&mut proof, &default_agent_info(connection_h));

        AgencyMock::set_next_response(PROOF_RESPONSE);
        AgencyMock::set_next_response(UPDATE_PROOF_RESPONSE);

        proof.update_state(None).unwrap();
        assert_eq!(proof.get_state(), VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_update_state_with_message() {
        let _setup = SetupMocks::init();

        let mut proof = create_boxed_proof(None, None, None);
        proof.update_state(Some(PROOF_RESPONSE_STR.to_string())).unwrap();
        assert_eq!(proof.get_state(), VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_update_state_with_reject_message() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();
        let mut proof = create_boxed_proof(Some(VcxStateType::VcxStateOfferSent),
                                           Some(ProofStateType::ProofUndefined),
                                           Some(connection_handle));

        proof.update_state(Some(PROOF_REJECT_RESPONSE_STR.to_string())).unwrap();
        assert_eq!(proof.get_state(), VcxStateType::VcxStateRejected as u32);
    }

    #[test]
    fn test_get_proof_returns_proof_when_proof_state_invalid() {
        let _setup = SetupMocks::init();

        let mut proof = create_boxed_proof(Some(VcxStateType::VcxStateOfferSent),
                                           None,
                                           Some(build_test_connection()));

        AgencyMock::set_next_response(PROOF_RESPONSE);
        AgencyMock::set_next_response(UPDATE_PROOF_RESPONSE);
        //httpclient::set_next_u8_response(GET_PROOF_OR_CREDENTIAL_RESPONSE);

        proof.update_state(None).unwrap();
        assert_eq!(proof.get_state(), VcxStateType::VcxStateRequestReceived as u32);
        assert_eq!(proof.get_proof_state(), ProofStateType::ProofInvalid as u32);
        let proof_data = proof.get_proof().unwrap();
        assert!(proof_data.contains(r#""cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0""#));
        assert!(proof_data.contains(r#""schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0""#));
    }

    #[test]
    fn test_build_credential_defs_json_with_multiple_credentials() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfo {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfo {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let credential_json = Proof::build_credential_defs_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(CRED_DEF_JSON).unwrap();
        let expected = json!({CRED_DEF_ID:json}).to_string();
        assert_eq!(credential_json, expected);
    }

    #[test]
    fn test_build_schemas_json_with_multiple_schemas() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfo {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfo {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let schema_json = Proof::build_schemas_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(SCHEMA_JSON).unwrap();
        let expected = json!({SCHEMA_ID:json}).to_string();
        assert_eq!(schema_json, expected);
    }

    #[test]
    fn test_build_rev_reg_defs_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfo {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: Some("id1".to_string()),
            timestamp: None,
        };
        let cred2 = CredInfo {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some("id2".to_string()),
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let rev_reg_defs_json = Proof::build_rev_reg_defs_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(&rev_def_json()).unwrap();
        let expected = json!({REV_REG_ID:json}).to_string();
        assert_eq!(rev_reg_defs_json, expected);
    }

    #[test]
    fn test_build_rev_reg_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfo {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: Some("id1".to_string()),
            timestamp: Some(1),
        };
        let cred2 = CredInfo {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some("id2".to_string()),
            timestamp: Some(2),
        };
        let credentials = vec![cred1, cred2];
        let rev_reg_json = Proof::build_rev_reg_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(REV_REG_JSON).unwrap();
        let expected = json!({REV_REG_ID:{"1":json}}).to_string();
        assert_eq!(rev_reg_json, expected);
    }

    #[test]
    fn test_get_proof() {
        let _setup = SetupMocks::init();

        let mut proof_msg_obj = ProofMessage::new();
        proof_msg_obj.libindy_proof = PROOF_JSON.to_string();

        let mut proof = create_boxed_proof(None, None, None);
        proof.proof = Some(proof_msg_obj);

        let proof_str = proof.get_proof().unwrap();
        assert_eq!(&proof_str, PROOF_JSON);
    }

    #[test]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let h2 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let h3 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let h4 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let h5 = create_proof("1".to_string(), REQUESTED_ATTRS.to_owned(), REQUESTED_PREDICATES.to_owned(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        release_all();
        assert_eq!(h1.release().unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(h2.release().unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(h3.release().unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(h4.release().unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(h5.release().unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
    }

    #[ignore]
    #[test]
    fn test_proof_validation_with_predicate() {
        let _setup = SetupLibraryWallet::init();

//         pool::tests::open_test_pool();
        //Generated proof from a script using libindy's python wrapper

        let proof_msg: ProofMessage = serde_json::from_str(PROOF_LIBINDY).unwrap();
        let mut proof_req_msg = ProofRequestMessage::create();
        proof_req_msg.proof_request_data = serde_json::from_str(PROOF_REQUEST).unwrap();
        let mut proof = Proof {
            source_id: "12".to_string(),
            msg_uid: String::from("1234"),
            ref_msg_id: String::new(),
            requested_attrs: String::from("[]"),
            requested_predicates: REQUESTED_PREDICATES.to_string(),
            state: VcxStateType::VcxStateRequestReceived,
            proof_state: ProofStateType::ProofUndefined,
            name: String::new(),
            version: String::from("1.0"),
            nonce: generate_nonce().unwrap(),
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
            proof: Some(proof_msg),
            proof_request: Some(proof_req_msg),
            revocation_interval: RevocationInterval { from: None, to: None },
            thread: Some(Thread::new()),
        };
        apply_agent_info(&mut proof, &default_agent_info(None));

        let rc = proof.proof_validation();
        assert!(rc.is_ok());
        assert_eq!(proof.proof_state, ProofStateType::ProofValidated);

        let proof_data = proof.get_proof().unwrap();
        assert!(proof_data.contains(r#""schema_seq_no":694,"issuer_did":"DunkM3x1y7S4ECgSL4Wkru","credential_uuid":"claim::1f927d68-8905-4188-afd6-374b93202802","attr_info":{"name":"age","value":18,"type":"predicate","predicate_type":"GE"}}"#));
    }

    #[ignore]
    #[test]
    fn test_send_proof_request_can_be_retried() {
        let _setup = SetupLibraryWallet::init();

        let connection_handle = build_test_connection();
        connection_handle.set_agent_verkey(VERKEY).unwrap();
        connection_handle.set_agent_did(DID).unwrap();
        connection_handle.set_their_pw_verkey(VERKEY).unwrap();

        let handle = create_proof("1".to_string(),
                                  REQUESTED_ATTRS.to_owned(),
                                  REQUESTED_PREDICATES.to_owned(),
                                  r#"{"support_revocation":false}"#.to_string(),
                                  "Optional".to_owned()).unwrap();
        assert_eq!(handle.send_proof_request(connection_handle).unwrap_err().kind(), VcxErrorKind::TimeoutLibindy);
        assert_eq!(handle.get_state().unwrap(), VcxStateType::VcxStateInitialized as u32);
        assert_eq!(handle.get_proof_uuid().unwrap(), "");

        // Retry sending proof request
        assert_eq!(handle.send_proof_request(connection_handle).unwrap(), 0);
        assert_eq!(handle.get_state().unwrap(), VcxStateType::VcxStateOfferSent as u32);
        assert_eq!(handle.get_proof_uuid().unwrap(), "ntc2ytb");
    }

    #[test]
    fn test_get_proof_request_status_can_be_retried() {
        let _setup = SetupMocks::init();

        let _new_handle = 1;

        let mut proof = create_boxed_proof(None, None, Some(build_test_connection()));

        AgencyMock::set_next_response(PROOF_RESPONSE);
        AgencyMock::set_next_response(UPDATE_PROOF_RESPONSE);
        //httpclient::set_next_u8_response(GET_PROOF_OR_CREDENTIAL_RESPONSE);

        proof.get_proof_request_status(None).unwrap();
        assert_eq!(proof.get_state(), VcxStateType::VcxStateRequestReceived as u32);
        assert_eq!(proof.get_proof_state(), ProofStateType::ProofInvalid as u32);

        // Changing the state and proof state to show that validation happens again
        // and resets the values to received and Invalid
        AgencyMock::set_next_response(PROOF_RESPONSE);
        AgencyMock::set_next_response(UPDATE_PROOF_RESPONSE);
        proof.state = VcxStateType::VcxStateOfferSent;
        proof.proof_state = ProofStateType::ProofUndefined;
        proof.get_proof_request_status(None).unwrap();
        proof.update_state(None).unwrap();
        assert_eq!(proof.get_state(), VcxStateType::VcxStateRequestReceived as u32);
        assert_eq!(proof.get_proof_state(), ProofStateType::ProofInvalid as u32);
    }

    #[test]
    fn test_proof_errors() {
        let _setup = SetupLibraryWallet::init();

        let mut proof = create_boxed_proof(None, None, None);

        let bad_handle = Handle::<Connections>::dummy();
        // TODO: Do something to guarantee that this handle is bad
        assert_eq!(proof.send_proof_request(bad_handle).unwrap_err().kind(), VcxErrorKind::NotReady);
        // TODO: Add test that returns a INVALID_PROOF_CREDENTIAL_DATA
        assert_eq!(proof.get_proof_request_status(None).unwrap_err().kind(), VcxErrorKind::WalletRecordNotFound);


        let empty = r#""#;

        assert_eq!(create_proof("my source id".to_string(),
                                empty.to_string(),
                                "{}".to_string(),
                                r#"{"support_revocation":false}"#.to_string(),
                                "my name".to_string()).unwrap_err().kind(), VcxErrorKind::InvalidAttributesStructure);


        let bad_handle = Handle::<Proofs>::dummy();

        assert_eq!(bad_handle.to_string().unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);

        assert_eq!(bad_handle.get_source_id().unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);

        assert_eq!(from_string(empty).unwrap_err().kind(), VcxErrorKind::InvalidJson);

        let mut proof_good = create_boxed_proof(None, None, None);
        assert_eq!(proof_good.get_proof_request_status(None).unwrap_err().kind(), VcxErrorKind::WalletRecordNotFound);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_proof_verification() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, _, proof_req, proof) = crate::utils::libindy::anoncreds::tests::create_proof();

        let mut proof_req_obj = ProofRequestMessage::create();
        proof_req_obj.proof_request_data = serde_json::from_str(&proof_req).unwrap();

        let mut proof_msg = ProofMessage::new();
        proof_msg.libindy_proof = proof;

        let mut proof = create_boxed_proof(None, None, None);
        proof.proof = Some(proof_msg);
        proof.proof_request = Some(proof_req_obj);

        let rc = proof.proof_validation();

        assert!(rc.is_ok());
        assert_eq!(proof.proof_state, ProofStateType::ProofValidated);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_self_attested_proof_verification() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (proof_req, proof) = crate::utils::libindy::anoncreds::tests::create_self_attested_proof();

        let mut proof_req_obj = ProofRequestMessage::create();
        proof_req_obj.proof_request_data = serde_json::from_str(&proof_req).unwrap();

        let mut proof_msg = ProofMessage::new();
        proof_msg.libindy_proof = proof;

        let mut proof = create_boxed_proof(None, None, None);
        proof.proof = Some(proof_msg);
        proof.proof_request = Some(proof_req_obj);

        let rc = proof.proof_validation();

        assert!(rc.is_ok());
        assert_eq!(proof.proof_state, ProofStateType::ProofValidated);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_proof_verification_restrictions() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": {
               "address1_1": {
                   "name":"address1",
                   "restrictions": [{ "issuer_did": "Not Here" }]
               },
               "zip_2": { "name":"zip", },
               "self_attest_3": { "name":"self_attest", },
           },
           "requested_predicates": {},
        }).to_string();

        let (_, _, _, proof) = crate::utils::libindy::anoncreds::tests::create_proof();

        let mut proof_req_obj = ProofRequestMessage::create();
        proof_req_obj.proof_request_data = serde_json::from_str(&proof_req).unwrap();

        let mut proof_msg = ProofMessage::new();
        proof_msg.libindy_proof = proof;

        let mut proof = create_boxed_proof(None, None, None);
        proof.proof = Some(proof_msg);
        proof.proof_request = Some(proof_req_obj);

        let rc = proof.proof_validation();

        // proof validation should fail because restriction
        rc.unwrap_err(); //FIXME check error code also
        assert_eq!(proof.proof_state, ProofStateType::ProofInvalid);

        // remove restriction, now validation should pass
        proof.proof_state = ProofStateType::ProofUndefined;
        proof.proof_request.as_mut().unwrap()
            .proof_request_data.requested_attributes
            .get_mut("address1_1").unwrap().restrictions = None;
        let rc = proof.proof_validation();

        rc.unwrap();
        assert_eq!(proof.proof_state, ProofStateType::ProofValidated);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_proof_validate_attribute() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, _, proof_req, proof_json) = crate::utils::libindy::anoncreds::tests::create_proof();

        let mut proof_req_obj = ProofRequestMessage::create();

        proof_req_obj.proof_request_data = serde_json::from_str(&proof_req).unwrap();

        let mut proof_msg = ProofMessage::new();
        let mut proof = create_boxed_proof(None, None, None);
        proof.proof_request = Some(proof_req_obj);

        // valid proof_obj
        {
            proof_msg.libindy_proof = proof_json.clone();
            proof.proof = Some(proof_msg);

            let _rc = proof.proof_validation().unwrap();
            assert_eq!(proof.proof_state, ProofStateType::ProofValidated);
        }

        let mut proof_obj: serde_json::Value = serde_json::from_str(&proof_json).unwrap();

        // change Raw value
        {
            let mut proof_msg = ProofMessage::new();
            proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["raw"] = json!("Other Value");
            let proof_json = serde_json::to_string(&proof_obj).unwrap();

            proof_msg.libindy_proof = proof_json;
            proof.proof = Some(proof_msg);

            let rc = proof.proof_validation();
            rc.unwrap_err();
            assert_eq!(proof.get_proof_state(), ProofStateType::ProofInvalid as u32);
        }

        // change Encoded value
        {
            let mut proof_msg = ProofMessage::new();
            proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["encoded"] = json!("1111111111111111111111111111111111111111111111111111111111");
            let proof_json = serde_json::to_string(&proof_obj).unwrap();

            proof_msg.libindy_proof = proof_json;
            proof.proof = Some(proof_msg);

            let rc = proof.proof_validation();
            rc.unwrap_err(); //FIXME check error code also
            assert_eq!(proof.get_proof_state(), ProofStateType::ProofInvalid as u32);
        }
    }

    #[test]
    fn test_create_proof_with_proposal() {
        let _setup = SetupMocks::init();

        let proposal = json!({
            "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation",
            "@id": "<uuid-presentation>",
            "comment": "some comment",
            "presentation_proposal": {
                "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation-preview",
                "attributes": [
                    {
                        "name": "account",
                        "cred_def_id": "BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag",
                        "value": "12345678",
                        "referent": "0"
                    },
                    {
                        "name": "streetAddress",
                        "cred_def_id": "BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag",
                        "value": "123 Main Street",
                        "referent": "0"
                    },
                ],
                "predicates": [
                ]
            }
        }).to_string();

        let proof_handle = create_proof_with_proposal("1".to_string(),
                                                      "Optional".to_owned(),
                                                      proposal).unwrap();

        PROOF_MAP.get_mut(proof_handle, |proof| {
            match proof {
                Proofs::Pending(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be V3")),
                Proofs::V1(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be V3")),
                Proofs::V3(verifier) => {
                    assert_eq!(VcxStateType::VcxStateRequestReceived as u32, verifier.state());
                    let proposal_json = verifier.get_presentation_proposal_request().unwrap();
                    let proposal: PresentationPreview = serde_json::from_str(&proposal_json).unwrap();
                    assert_eq!(2, proposal.attributes.len());
                    assert_eq!(0, proposal.predicates.len());
                    Ok(())
                }
            }
        }).unwrap();
    }
}
