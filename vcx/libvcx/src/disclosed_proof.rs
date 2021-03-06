use serde_json::Value;
use std::convert::TryInto;
use std::mem::take;

use crate::connection::Connections;
use crate::utils::object_cache::{ObjectCache, Handle};
use crate::api::VcxStateType;
use crate::error::prelude::*;
use crate::aries::messages::thread::Thread;
use crate::agent;
use crate::agent::messages::{
    GeneralMessage,
    RemoteMessageType,
};
use crate::legacy::messages::proof_presentation::{
    proof_message::ProofMessage,
    proof_request::ProofRequestMessage,
};
use crate::settings;
use crate::utils::error;
use crate::utils::constants::*;
use crate::utils::libindy::anoncreds::holder::Holder as IndyHolder;
use crate::aries::{
    messages::proof_presentation::presentation_request::PresentationRequest,
    handlers::proof_presentation::prover::Prover,
};
use crate::agent::agent_info::{get_agent_info, MyAgentInfo, get_agent_attr};
use crate::utils::httpclient::AgencyMock;
use crate::legacy::messages::proof_presentation::proof_request::parse_proof_req_message;
use crate::agent::messages::payload::PayloadKinds;
use crate::aries::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::aries::messages::proof_presentation::v10::presentation_proposal::PresentationProposal as PresentationProposalV1;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<DisclosedProofs>  = Default::default();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
pub enum DisclosedProofs {
    #[serde(rename = "3.0")]
    Pending(DisclosedProof),
    #[serde(rename = "1.0")]
    V1(DisclosedProof),
    #[serde(rename = "2.0")]
    V3(Prover),
}

impl Default for DisclosedProof {
    fn default() -> DisclosedProof
    {
        DisclosedProof {
            source_id: String::new(),
            state: VcxStateType::VcxStateNone,
            proof_request: None,
            proof: None,
            link_secret_alias: settings::DEFAULT_LINK_SECRET_ALIAS.to_string(),
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
            thread: Some(Thread::new()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DisclosedProof {
    source_id: String,
    state: VcxStateType,
    proof_request: Option<ProofRequestMessage>,
    proof: Option<ProofMessage>,
    link_secret_alias: String,
    my_did: Option<String>,
    my_vk: Option<String>,
    their_did: Option<String>,
    their_vk: Option<String>,
    agent_did: Option<String>,
    agent_vk: Option<String>,
    thread: Option<Thread>,
}

impl DisclosedProof {
    fn create_with_request(source_id: &str, proof_req: &str) -> VcxResult<DisclosedProof> {
        trace!("create_with_request >>> source_id: {}, proof_req: {}", source_id, proof_req);
        debug!("DisclosedProof {}: Creating disclosed proof object for request", source_id);

        let mut proof: DisclosedProof = Default::default();

        proof.set_source_id(source_id);
        proof.set_proof_request(proof_req)?;
        proof.set_state(VcxStateType::VcxStateRequestReceived);

        trace!("create_with_request <<<");

        Ok(proof)
    }

    fn set_proof_request(&mut self, proof_req: &str) -> VcxResult<()> {
        let proof_req: ProofRequestMessage = serde_json::from_str(proof_req)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest, format!("Cannot parse ProofRequest from `proof_req` JSON string. Err: {}", err)))?;
        self.proof_request = Some(proof_req);
        Ok(())
    }

    fn get_state(&self) -> u32 {
        trace!("DisclosedProof::get_state >>>");

        let state = self.state as u32;

        debug!("DisclosedProof {} is in state {}", self.source_id, self.state as u32);
        trace!("DisclosedProof::get_state <<< state: {:?}", state);
        state
    }

    fn set_state(&mut self, state: VcxStateType) {
        trace!("DisclosedProof::set_state >>> state: {:?}", state);
        self.state = state
    }

    fn retrieve_credentials(&self) -> VcxResult<String> {
        trace!("DisclosedProof::retrieve_credentials >>>");
        debug!("DisclosedProof {}: Retrieving credentials for request", self.source_id);

        if settings::indy_mocks_enabled() { return Ok(CREDS_FROM_PROOF_REQ.to_string()); }

        let proof_req = self.proof_request
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Disclosed Proof object state: `proof_request` not found", self.source_id)))?;

        let indy_proof_req = json!(proof_req.proof_request_data).to_string();

        let credentials = IndyHolder::get_credentials_for_proof_req(&indy_proof_req)?;

        trace!("DisclosedProof::retrieve_credentials <<< credentials: {:?}", secret!(credentials));

        Ok(credentials)
    }

    fn generate_proof(&mut self, credentials: &str, self_attested_attrs: &str) -> VcxResult<u32> {
        trace!("DisclosedProof::generate_proof >>> credentials: {}, self_attested_attrs: {}", secret!(&credentials), secret!(&self_attested_attrs));

        debug!("DisclosedProof {}: Generating proof", self.source_id);
        if settings::indy_mocks_enabled() { return Ok(error::SUCCESS.code_num); }

        let proof_req = self.proof_request.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState,
                                      format!("Invalid {} Disclosed Proof object state: `proof_request` not found", self.source_id)))?;

        let proof_req_data_json = json!(proof_req.proof_request_data).to_string();

        let proof = IndyHolder::generate_proof(credentials, self_attested_attrs, &proof_req_data_json)?;

        let mut proof_msg = ProofMessage::new();
        proof_msg.libindy_proof = proof;
        self.proof = Some(proof_msg);

        trace!("DisclosedProof::generate_proof <<<");

        Ok(error::SUCCESS.code_num)
    }

    fn generate_proof_msg(&self) -> VcxResult<String> {
        let proof = match settings::indy_mocks_enabled() {
            false => {
                let proof: &ProofMessage = self.proof.as_ref()
                    .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState,
                                              format!("Invalid {} Disclosed Proof object state: `proof` not found", self.source_id)))?;

                json!(proof).to_string()
            }
            true => DEFAULT_GENERATED_PROOF.to_string(),
        };

        Ok(proof)
    }

    fn _prep_proof_reference(&mut self, agent_info: &MyAgentInfo) -> VcxResult<String> {
        let proof_req = self.proof_request
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState,
                                      format!("Invalid {} Disclosed Proof object state: `proof_request` not found", self.source_id)))?;

        let ref_msg_uid = proof_req.msg_ref_id
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidProofRequest, "Invalid ProofRequest message.`msg_ref_id` not found"))?;

        let their_did = get_agent_attr(&agent_info.their_pw_did)?;

        self.thread
            .as_mut()
            .map(|thread| thread.increment_receiver(&their_did));

        Ok(ref_msg_uid.to_string())
    }

    fn send_proof(&mut self, connection_handle: Handle<Connections>) -> VcxResult<u32> {
        trace!("DisclosedProof::send_proof >>> connection_handle: {}", connection_handle);

        debug!("DisclosedProof {}: Sending proof", self.source_id);

        let agent_info = get_agent_info()?.pw_info(connection_handle)?;
        apply_agent_info(self, &agent_info);

        let ref_msg_uid = self._prep_proof_reference(&agent_info)?;

        let proof = self.generate_proof_msg()?;

        agent::messages::send_message()
            .to(&agent_info.my_pw_did()?)?
            .to_vk(&agent_info.my_pw_vk()?)?
            .msg_type(&RemoteMessageType::Proof)?
            .agent_did(&agent_info.pw_agent_did()?)?
            .agent_vk(&agent_info.pw_agent_vk()?)?
            .version(agent_info.version.clone())?
            .edge_agent_payload(&agent_info.my_pw_vk()?,
                                &agent_info.their_pw_vk()?,
                                &proof,
                                PayloadKinds::Proof,
                                self.thread.clone(),
            )?
            .ref_msg_id(Some(ref_msg_uid))?
            .send_secure()
            .map_err(|err| err.extend("Cannot not send proof"))?;

        self.state = VcxStateType::VcxStateAccepted;

        trace!("DisclosedProof::send_proof <<<");

        Ok(error::SUCCESS.code_num)
    }

    fn generate_reject_proof_msg(&self) -> VcxResult<String> {
        let msg = match settings::indy_mocks_enabled() {
            false => {
                let proof_reject = ProofMessage::new_reject();
                json!(proof_reject).to_string()
            }
            true => DEFAULT_REJECTED_PROOF.to_string(),
        };

        Ok(msg)
    }

    fn reject_proof(&mut self, connection_handle: Handle<Connections>) -> VcxResult<u32> {
        trace!("DisclosedProof::reject_proof >>> connection_handle: {}", connection_handle);
        debug!("DisclosedProof {}: Rejecting proof", self.source_id);

        // There feels like there's a much more rusty way to do the below.
        let agent_info = get_agent_info()?.pw_info(connection_handle)?;
        apply_agent_info(self, &agent_info);

        let ref_msg_uid = self._prep_proof_reference(&agent_info)?;

        let proof_reject = self.generate_reject_proof_msg()?;

        agent::messages::send_message()
            .to(&agent_info.my_pw_did()?)?
            .to_vk(&agent_info.my_pw_vk()?)?
            .msg_type(&RemoteMessageType::Proof)?
            .agent_did(&agent_info.pw_agent_did()?)?
            .agent_vk(&agent_info.pw_agent_vk()?)?
            .version(agent_info.version.clone())?
            .edge_agent_payload(&agent_info.my_pw_vk()?,
                                &agent_info.their_pw_vk()?,
                                &proof_reject,
                                PayloadKinds::Proof,
                                self.thread.clone())?
            .ref_msg_id(Some(ref_msg_uid))?
            .send_secure()
            .map_err(|err| err.extend("Cannot not send proof reject"))?;

        self.state = VcxStateType::VcxStateRejected;

        trace!("DisclosedProof::reject_proof <<<");

        return Ok(error::SUCCESS.code_num);
    }

    fn set_source_id(&mut self, id: &str) { self.source_id = id.to_string(); }

    fn get_source_id(&self) -> String { self.source_id.to_string() }

    #[cfg(test)] // TODO: REMOVE IT
    fn from_str(data: &str) -> VcxResult<DisclosedProof> {
        use crate::agent::messages::ObjectWithVersion;
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<DisclosedProof>| obj.data)
            .map_err(|err| err.extend("Cannot deserialize DisclosedProof"))
    }
}

//********************************************
//         HANDLE FUNCTIONS
//********************************************
fn handle_err(err: VcxError) -> VcxError {
    if err.kind() == VcxErrorKind::InvalidHandle {
        VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle)
    } else {
        err
    }
}

fn apply_agent_info(proof: &mut DisclosedProof, agent_info: &MyAgentInfo) {
    proof.my_did = agent_info.my_pw_did.clone();
    proof.my_vk = agent_info.my_pw_vk.clone();
    proof.their_did = agent_info.their_pw_did.clone();
    proof.their_vk = agent_info.their_pw_vk.clone();
    proof.agent_did = agent_info.pw_agent_did.clone();
    proof.agent_vk = agent_info.pw_agent_vk.clone();
}

fn create_proof_v3(source_id: &str, proof_req: &str) -> VcxResult<Option<DisclosedProofs>> {
    trace!("create_proof_v3 >>> source_id: {}, proof_req: {}", source_id, secret!(proof_req));
    debug!("creating aries disclosed proof object");

    // Received request of new format -- redirect to aries folder
    if let Ok(presentation_request) = serde_json::from_str::<PresentationRequest>(proof_req) {
        let proof = Prover::create(source_id, presentation_request)?;
        return Ok(Some(DisclosedProofs::V3(proof)));
    }

    trace!("create_proof_v3 <<<");

    Ok(None)
}

fn create_pending_proof(source_id: &str, proof_req: &str) -> VcxResult<DisclosedProofs> {
    trace!("create_pending_proof >>> source_id: {}, proof_req: {}", source_id, secret!(proof_req));
    debug!("creating pending disclosed proof object");

    let proof: DisclosedProof = DisclosedProof::create_with_request(source_id, proof_req)?;

    trace!("create_pending_proof <<<");

    Ok(DisclosedProofs::Pending(proof))
}

fn create_proof_v1(source_id: &str, proof_req: &str) -> VcxResult<DisclosedProofs> {
    trace!("create_proof_v1 >>> source_id: {}, proof_req: {}", source_id, secret!(proof_req));
    debug!("creating v10 disclosed proof object");

    let proof: DisclosedProof = DisclosedProof::create_with_request(source_id, proof_req)?;

    trace!("create_proof_v1 <<<");

    Ok(DisclosedProofs::V1(proof))
}

pub fn create_proof(source_id: &str, proof_req: &str) -> VcxResult<Handle<DisclosedProofs>> {
    trace!("create_proof >>> source_id: {}, proof_req: {}", source_id, secret!(proof_req));
    debug!("creating disclosed proof with id: {}", source_id);

    let proof =
        match create_proof_v3(source_id, &proof_req)? {
            Some(proof) => proof,
            None => {
                create_pending_proof(source_id, proof_req)?
            }
        };

    let handle = HANDLE_MAP.add(proof)?;

    debug!("inserting proof {} into handle map", source_id);
    trace!("create_proof <<<");

    Ok(handle)
}

pub fn create_proof_with_msgid(source_id: &str, connection_handle: Handle<Connections>, msg_id: &str) -> VcxResult<(Handle<DisclosedProofs>, String)> {
    trace!("create_proof_with_msgid >>> source_id: {}, proof_req: {}", source_id, msg_id);
    debug!("creating disclosed proof with message id: {}", source_id);

    let proof_request = get_proof_request(connection_handle, &msg_id)?;

    let proof = if connection_handle.is_aries_connection()? {
        create_proof_v3(source_id, &proof_request)?
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidConnectionHandle, format!("Connection can not be used for Proprietary Issuance protocol")))?
    } else {
        create_proof_v1(source_id, &proof_request)?
    };

    let handle = HANDLE_MAP.add(proof)?;

    debug!("inserting disclosed proof {} into handle map", source_id);
    trace!("create_proof_with_msgid <<<");

    Ok((handle, proof_request))
}

pub fn create_proposal(source_id: &str, proposal: String, comment: String) -> VcxResult<Handle<DisclosedProofs>> {
    trace!("create_proposal >>> source_id: {}, proposal: {}, comment: {}",
           source_id, secret!(proposal), comment);
    debug!("creating disclosed proof proposal with id: {}", source_id);

    let preview = serde_json::from_str(&proposal)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofProposal, format!("Cannot parse proposal from JSON string. Err: {}", err)))?;

    let proposal = PresentationProposal::V1(
        PresentationProposalV1::create()
            .set_presentation_preview(preview)
            .set_comment(comment)
    );

    let proof = DisclosedProofs::V3(Prover::create_proposal(source_id, proposal)?);

    let handle = HANDLE_MAP.add(proof)?;

    debug!("inserting disclosed proof {} into handle map", source_id);
    trace!("create_proposal <<<");

    Ok(handle)
}

impl Handle<DisclosedProofs> {
    pub fn get_state(self) -> VcxResult<u32> {
        HANDLE_MAP.get(self, |obj| {
            match obj {
                DisclosedProofs::Pending(obj) => Ok(obj.get_state()),
                DisclosedProofs::V1(obj) => Ok(obj.get_state()),
                DisclosedProofs::V3(obj) => Ok(obj.state())
            }
        }).map_err(handle_err)
    }

    pub fn update_state(self, message: Option<String>) -> VcxResult<u32> {
        HANDLE_MAP.get_mut(self, |obj| {
            match obj {
                DisclosedProofs::Pending(obj) => {
                    // update_state is just the same as get_state for disclosed_proof
                    Ok(obj.get_state())
                }
                DisclosedProofs::V1(obj) => {
                    // update_state is just the same as get_state for disclosed_proof
                    Ok(obj.get_state())
                }
                DisclosedProofs::V3(obj) => {
                    obj.update_state(message.as_ref().map(String::as_str))?;
                    Ok(obj.state())
                }
            }
        }).map_err(handle_err)
    }

    pub fn to_string(self) -> VcxResult<String> {
        HANDLE_MAP.get(self, |obj| {
            serde_json::to_string(obj)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize DisclosedProof object. Err: {:?}", err)))
        }).map_err(handle_err)
    }

    pub fn release(self) -> VcxResult<()> {
        HANDLE_MAP.release(self).map_err(handle_err)
    }

    pub fn generate_proof_msg(self) -> VcxResult<String> {
        HANDLE_MAP.get(self, |obj| {
            match obj {
                DisclosedProofs::Pending(obj) => obj.generate_proof_msg(),
                DisclosedProofs::V1(obj) => obj.generate_proof_msg(),
                DisclosedProofs::V3(obj) => {
                    let presentation = obj.generate_presentation_msg()?;

                    // strict aries protocol is set. return aries formatted Proof
                    if settings::is_strict_aries_protocol_set() {
                        return Ok(json!(presentation).to_string());
                    }

                    // convert Proof into proprietary format
                    let proof: ProofMessage = presentation.to_owned().try_into()?;
                    Ok(json!(proof).to_string())
                }
            }
        }).map_err(handle_err)
    }


    pub fn send_proof(self, connection_handle: Handle<Connections>) -> VcxResult<u32> {
        HANDLE_MAP.get_mut(self, |proof| {
            let new_proof = match proof {
                DisclosedProofs::Pending(obj) => {
                    // if Aries connection is established --> Convert DisclosedProofs object to Aries presentation
                    // if connection handle is 0 --> ephemeral Aries proof
                    if connection_handle.is_aries_connection()? || connection_handle == 0 {
                        debug!("Convert pending proof into aries proof");

                        let proof_request = take(&mut obj.proof_request)
                            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady,
                                                      format!("Disclosed Proof object {} in state {} not ready to get Proof Request message", obj.source_id, obj.state as u32)))?;

                        let proof = take(&mut obj.proof)
                            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady,
                                                      format!("Disclosed Proof object {} in state {} not ready to get Proof message", obj.source_id, obj.state as u32)))?;

                        let mut prover = Prover::create(&obj.get_source_id(), proof_request.try_into()?)?;
                        prover.set_presentation(proof.try_into()?)?;
                        prover.send_presentation(connection_handle)?;

                        DisclosedProofs::V3(prover)
                    } else { // else --> Convert DisclosedProofs object to Proprietary proof object
                        obj.send_proof(connection_handle)?;
                        DisclosedProofs::V1(take(obj))
                    }
                }
                DisclosedProofs::V1(obj) => {
                    obj.send_proof(connection_handle)?;
                    DisclosedProofs::V1(take(obj))
                }
                DisclosedProofs::V3(obj) => {
                    obj.send_presentation(connection_handle)?;
                    DisclosedProofs::V3(obj.clone())
                }
            };
            *proof = new_proof;
            Ok(error::SUCCESS.code_num)
        }).map_err(handle_err)
    }

    pub fn send_proposal(self, connection_handle: Handle<Connections>) -> VcxResult<u32> {
        HANDLE_MAP.get_mut(self, |proof| {
            let new_proof = match proof {
                DisclosedProofs::Pending(_obj) => {
                    Err(VcxError::from(VcxErrorKind::ActionNotSupported))
                }
                DisclosedProofs::V1(_) => {
                    Err(VcxError::from(VcxErrorKind::ActionNotSupported))
                }
                DisclosedProofs::V3(obj) => {
                    obj.send_proposal(connection_handle)?;
                    // TODO: avoid cloning
                    Ok(DisclosedProofs::V3(obj.clone()))
                }
            }?;
            *proof = new_proof;
            Ok(error::SUCCESS.code_num)
        }).map_err(handle_err)
    }

    pub fn generate_reject_proof_msg(self) -> VcxResult<String> {
        HANDLE_MAP.get_mut(self, |obj| {
            match obj {
                DisclosedProofs::Pending(obj) => obj.generate_reject_proof_msg(),
                DisclosedProofs::V1(obj) => obj.generate_reject_proof_msg(),
                DisclosedProofs::V3(_) => {
                    Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Aries DiscloseProof type doesn't support this action: `generate_reject_proof_msg`."))
                }
            }
        }).map_err(handle_err)
    }

    pub fn reject_proof(self, connection_handle: Handle<Connections>) -> VcxResult<u32> {
        HANDLE_MAP.get_mut(self, |proof| {
            let new_proof = match proof {
                DisclosedProofs::Pending(obj) => {
                    // if Aries connection is established --> Convert DisclosedProofs object to Aries presentation
                    if connection_handle.is_aries_connection()? {
                        debug!("Convert pending proof into aries proof");

                        let proof_request = take(&mut obj.proof_request)
                            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady,
                                                      format!("Disclosed Proof object {} in state {} not ready to get Proof Request message", obj.source_id, obj.state as u32)))?;

                        let mut prover = Prover::create(&obj.get_source_id(), proof_request.try_into()?)?;
                        prover.decline_presentation_request(connection_handle, Some(String::from("Presentation Request was rejected")), None)?;
                        DisclosedProofs::V3(prover)
                    } else { // else --> Convert DisclosedProofs object to Proprietary proof object
                        obj.reject_proof(connection_handle)?;
                        DisclosedProofs::V1(take(obj))
                    }
                }
                DisclosedProofs::V1(obj) => {
                    obj.reject_proof(connection_handle)?;
                    DisclosedProofs::V1(take(obj))
                }
                DisclosedProofs::V3(obj) => {
                    obj.decline_presentation_request(connection_handle, Some(String::from("Presentation Request was rejected")), None)?;
                    // TODO: avoid cloning
                    DisclosedProofs::V3(obj.clone())
                }
            };
            *proof = new_proof;
            Ok(error::SUCCESS.code_num)
        }).map_err(handle_err)
    }

    pub fn generate_proof(self, credentials: String, self_attested_attrs: String) -> VcxResult<u32> {
        HANDLE_MAP.get_mut(self, move |obj| {
            match obj {
                DisclosedProofs::Pending(obj) => {
                    obj.generate_proof(&credentials, &self_attested_attrs)
                }
                DisclosedProofs::V1(obj) => {
                    obj.generate_proof(&credentials, &self_attested_attrs)
                }
                DisclosedProofs::V3(obj) => {
                    obj.generate_presentation(credentials, self_attested_attrs)?;
                    Ok(error::SUCCESS.code_num)
                }
            }
        })
            .map(|_| error::SUCCESS.code_num)
            .map_err(handle_err)
    }

    pub fn decline_presentation_request(self, connection_handle: Handle<Connections>, reason: Option<String>, proposal: Option<String>) -> VcxResult<u32> {
        HANDLE_MAP.get_mut(self, move |proof| {
            let new_proof = match proof {
                DisclosedProofs::Pending(obj) => {
                    // if Aries connection is established --> Convert DisclosedProofs object to Aries presentation
                    if connection_handle.is_aries_connection()? {
                        debug!("Convert pending proof into aries proof");

                        let proof_request = obj.proof_request.clone()
                            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady,
                                                      format!("Disclosed Proof object {} in state {} not ready to get Proof Request message", obj.source_id, obj.state as u32)))?;

                        let mut prover = Prover::create(&obj.get_source_id(), proof_request.try_into()?)?;
                        prover.decline_presentation_request(connection_handle, reason.clone(), proposal.clone())?;
                        DisclosedProofs::V3(prover)
                    } else { // else --> Convert DisclosedProofs object to Proprietary proof object
                        obj.reject_proof(connection_handle)?;
                        DisclosedProofs::V1(take(obj))
                    }
                }
                DisclosedProofs::V1(obj) => {
                    obj.reject_proof(connection_handle)?;
                    DisclosedProofs::V1(take(obj))
                }
                DisclosedProofs::V3(obj) => {
                    obj.decline_presentation_request(connection_handle, reason, proposal)?;
                    // TODO: avoid cloning
                    DisclosedProofs::V3(obj.clone())
                }
            };
            *proof = new_proof;
            Ok(error::SUCCESS.code_num)
        })
            .map(|_| error::SUCCESS.code_num)
            .map_err(handle_err)
    }

    pub fn retrieve_credentials(self) -> VcxResult<String> {
        HANDLE_MAP.get_mut(self, |obj| {
            match obj {
                DisclosedProofs::Pending(obj) => obj.retrieve_credentials(),
                DisclosedProofs::V1(obj) => obj.retrieve_credentials(),
                DisclosedProofs::V3(obj) => obj.retrieve_credentials()
            }
        }).map_err(handle_err)
    }

    pub fn is_valid_handle(self) -> bool {
        HANDLE_MAP.has_handle(self)
    }

    //TODO one function with credential
    pub fn get_source_id(self) -> VcxResult<String> {
        HANDLE_MAP.get(self, |obj| {
            match obj {
                DisclosedProofs::Pending(obj) => Ok(obj.get_source_id()),
                DisclosedProofs::V1(obj) => Ok(obj.get_source_id()),
                DisclosedProofs::V3(obj) => Ok(obj.get_source_id().to_string())
            }
        }).map_err(handle_err)
    }

    pub fn get_problem_report_message(self) -> VcxResult<String> {
        HANDLE_MAP.get(self, |proof| {
            match proof {
                DisclosedProofs::Pending(_) | DisclosedProofs::V1(_) => {
                    Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Proprietary Proof type doesn't support this action: `get_problem_report_message`."))
                }
                DisclosedProofs::V3(obj) => {
                    obj.get_problem_report_message()
                }
            }
        }).map_err(handle_err)
    }
}

pub fn release_all() {
    HANDLE_MAP.drain().ok();
}


pub fn from_string(proof_data: &str) -> VcxResult<Handle<DisclosedProofs>> {
    let proof: DisclosedProofs = serde_json::from_str(proof_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse DisclosedProofs state object from JSON string. Err: {:?}", err)))?;

    HANDLE_MAP.add(proof)
}


fn get_proof_request(connection_handle: Handle<Connections>, msg_id: &str) -> VcxResult<String> {
    trace!("get_proof_request >>> connection_handle: {}, msg_id: {}", connection_handle, msg_id);
    debug!("DisclosedProof: getting proof request with id: {}", msg_id);

    if connection_handle.is_aries_connection()? {
        let presentation_request = Prover::get_presentation_request(connection_handle, msg_id)?;
        return serde_json::to_string_pretty(&presentation_request)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize Proof Request. Err: {}", err)));
    }

    let agent_info = get_agent_info()?.pw_info(connection_handle)?;

    AgencyMock::set_next_response(NEW_PROOF_REQUEST_RESPONSE);

    let message = agent::messages::get_message::get_connection_messages(&agent_info.my_pw_did()?,
                                                                        &agent_info.my_pw_vk()?,
                                                                        &agent_info.pw_agent_did()?,
                                                                        &agent_info.pw_agent_vk()?,
                                                                        Some(vec![msg_id.to_string()]),
                                                                        None,
                                                                        &agent_info.version()?)?;

    if message[0].msg_type != RemoteMessageType::ProofReq {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse,
                                      format!("Agency response contain the message of different type. Expected: ProofReq. Received: {:?}", message[0].msg_type)));
    }

    let request = parse_proof_req_message(&message[0], &agent_info.my_pw_vk()?)?;

    let proof_request = serde_json::to_string_pretty(&request)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize Proof Request. Err: {}", err)))?;

    trace!("get_proof_request <<< proof_request: {}", proof_request);
    Ok(proof_request)
}

//TODO one function with credential
pub fn get_proof_request_messages(connection_handle: Handle<Connections>, match_name: Option<&str>) -> VcxResult<String> {
    trace!("get_proof_request_messages >>> connection_handle: {}, match_name: {:?}", connection_handle, match_name);
    debug!("DisclosedProof: getting all proof request agent for connection {}", connection_handle);

    if connection_handle.is_aries_connection()? {
        let presentation_requests = Prover::get_presentation_request_messages(connection_handle, match_name)?;

        let mut msgs: Vec<Value> = Vec::new();
        for presentation_request in presentation_requests {
            if settings::is_strict_aries_protocol_set() {
                msgs.push(json!(presentation_request));
            } else {
                let presentation_request: ProofRequestMessage = presentation_request.try_into()?;
                msgs.push(json!(presentation_request));
            }
        }
        return Ok(json!(msgs).to_string());
    }

    AgencyMock::set_next_response(NEW_PROOF_REQUEST_RESPONSE);

    let agent_info = get_agent_info()?.pw_info(connection_handle)?;

    let payload = agent::messages::get_message::get_connection_messages(&agent_info.my_pw_did()?,
                                                                        &agent_info.my_pw_vk()?,
                                                                        &agent_info.pw_agent_did()?,
                                                                        &agent_info.pw_agent_vk()?,
                                                                        None,
                                                                        None,
                                                                        &agent_info.version()?)?;

    let mut messages: Vec<ProofRequestMessage> = Default::default();

    for msg in payload {
        if msg.sender_did.eq(&agent_info.my_pw_did()?) { continue; }

        if msg.msg_type == RemoteMessageType::ProofReq {
            let req = parse_proof_req_message(&msg, &agent_info.my_pw_vk()?)?;
            messages.push(req);
        }
    }

    let proof_requests = serde_json::to_string_pretty(&messages)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::SerializationError, format!("Cannot serialize ProofRequest. Err: {}", err),
        ))?;

    trace!("get_proof_request_messages <<< proof_requests: {}", proof_requests);
    Ok(proof_requests)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::connection;
    use serde_json::Value;
    use crate::utils::{
        constants::{ADDRESS_CRED_ID, LICENCE_CRED_ID, ADDRESS_SCHEMA_ID,
                    ADDRESS_CRED_DEF_ID, CRED_DEF_ID, SCHEMA_ID, ADDRESS_CRED_REV_ID,
                    ADDRESS_REV_REG_ID, REV_REG_ID, CRED_REV_ID, TEST_TAILS_FILE},
        get_temp_dir_path,
    };
    #[cfg(feature = "pool_tests")]
    use time;
    use crate::utils::devsetup::*;
    use crate::utils::libindy::anoncreds::types::ExtendedCredentialInfo;
    use crate::utils::libindy::anoncreds::proof_request::{NonRevokedInterval, ProofRequest};
    use crate::utils::libindy::cache::*;

    fn proof_req_no_interval() -> ProofRequest {
        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": { "name": "address1" },
                "zip_2": { "name": "zip" },
                "height_1": { "name": "height" }
            },
            "requested_predicates": {},
        }).to_string();

        serde_json::from_str(&proof_req).unwrap()
    }

    fn _get_proof_request_messages(connection_h: Handle<Connections>) -> String {
        let requests = get_proof_request_messages(connection_h, None).unwrap();
        let requests: Value = serde_json::from_str(&requests).unwrap();
        let requests = serde_json::to_string(&requests[0]).unwrap();
        requests
    }

    #[test]
    fn test_create_proof() {
        let _setup = SetupMocks::init();

        assert!(create_proof("1", crate::utils::constants::PROOF_REQUEST_JSON).unwrap() > 0);
    }

    #[test]
    fn test_create_fails() {
        let _setup = SetupMocks::init();

        assert_eq!(create_proof("1", "{}").unwrap_err().kind(), VcxErrorKind::InvalidProofRequest);
    }

    #[test]
    fn test_proof_cycle() {
        let _setup = SetupMocks::init();

        let connection_h = connection::tests::build_test_connection();

        let request = _get_proof_request_messages(connection_h);

        let handle = create_proof("TEST_CREDENTIAL", &request).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, handle.get_state().unwrap());

        handle.send_proof(connection_h).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, handle.get_state().unwrap());
    }

    #[test]
    fn test_proof_reject_cycle() {
        let _setup = SetupMocks::init();

        let connection_h = connection::tests::build_test_connection();

        let request = _get_proof_request_messages(connection_h);

        let handle = create_proof("TEST_CREDENTIAL", &request).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, handle.get_state().unwrap());

        handle.reject_proof(connection_h).unwrap();
        assert_eq!(VcxStateType::VcxStateRejected as u32, handle.get_state().unwrap());
    }

    #[test]
    fn test_create_proposal() {
        let _setup = SetupMocks::init();
        let proposal = r#"{"attributes":[{"name":"FirstName"}],"predicates":[]}"#;

        assert!(create_proposal("1", proposal.to_string(), "comment".to_string()).unwrap() > 0);
    }

    #[test]
    fn test_create_proposal_fails() {
        let _setup = SetupMocks::init();

        assert_eq!(create_proposal("1", "{}".to_string(), "comment".to_string()).unwrap_err().kind(), VcxErrorKind::InvalidProofProposal);
    }

    #[test]
    fn get_state_test() {
        let _setup = SetupMocks::init();

        let proof: DisclosedProof = Default::default();
        assert_eq!(VcxStateType::VcxStateNone as u32, proof.get_state());

        let handle = create_proof("id", crate::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, handle.get_state().unwrap())
    }

    #[test]
    fn to_string_test() {
        let _setup = SetupMocks::init();

        let handle = create_proof("id", crate::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let serialized = handle.to_string().unwrap();
        let j: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(j["version"], crate::utils::constants::PENDING_OBJECT_SERIALIZE_VERSION);

        let handle_2 = from_string(&serialized).unwrap();
        assert_ne!(handle, handle_2);
    }

    #[test]
    fn test_deserialize_fails() {
        let _setup = SetupDefaults::init();

        assert_eq!(from_string("{}").unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    fn test_deserialize_succeeds_with_self_attest_allowed() {
        let _setup = SetupDefaults::init();

        let handle = create_proof("id", crate::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let serialized = handle.to_string().unwrap();
        let p = DisclosedProof::from_str(&serialized).unwrap();
        assert_eq!(p.proof_request.unwrap().proof_request_data.requested_attributes.get("attr1_referent").unwrap().self_attest_allowed, Some(true))
    }

    #[test]
    fn test_find_schemas() {
        let _setup = SetupMocks::init();

        assert_eq!(IndyHolder::build_schemas_json(&[]).unwrap(), "{}");

        let cred1 = ExtendedCredentialInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let cred2 = ExtendedCredentialInfo {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let creds = vec![cred1, cred2];

        let schemas = IndyHolder::build_schemas_json(&creds).unwrap();
        assert!(schemas.len() > 0);
        assert!(schemas.contains(r#""id":"2hoqvcwupRTUNkXn6ArYzs:2:test-licence:4.4.4","name":"test-licence""#));
    }

    #[test]
    fn test_find_schemas_fails() {
        let _setup = SetupLibraryWallet::init();

        let credential_ids = vec![ExtendedCredentialInfo {
            requested_attr: "1".to_string(),
            referent: "2".to_string(),
            schema_id: "3".to_string(),
            cred_def_id: "3".to_string(),
            rev_reg_id: Some("4".to_string()),
            cred_rev_id: Some("5".to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        }];
        assert_eq!(IndyHolder::build_schemas_json(&credential_ids).unwrap_err().kind(), VcxErrorKind::InvalidSchema);
    }

    #[test]
    fn test_find_credential_def() {
        let _setup = SetupMocks::init();

        let cred1 = ExtendedCredentialInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let cred2 = ExtendedCredentialInfo {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let creds = vec![cred1, cred2];

        let credential_def = IndyHolder::build_cred_def_json(&creds).unwrap();
        assert!(credential_def.len() > 0);
        assert!(credential_def.contains(r#""id":"2hoqvcwupRTUNkXn6ArYzs:3:CL:2471","schemaId":"2471""#));
    }

    #[test]
    fn test_find_credential_def_fails() {
        let _setup = SetupLibraryWallet::init();

        let credential_ids = vec![ExtendedCredentialInfo {
            requested_attr: "1".to_string(),
            referent: "2".to_string(),
            schema_id: "3".to_string(),
            cred_def_id: "3".to_string(),
            rev_reg_id: Some("4".to_string()),
            cred_rev_id: Some("5".to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        }];
        assert_eq!(IndyHolder::build_cred_def_json(&credential_ids).unwrap_err().kind(), VcxErrorKind::CredentialDefinitionNotFound);
    }

    #[test]
    fn test_build_requested_credentials() {
        let _setup = SetupMocks::init();

        let cred1 = ExtendedCredentialInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: Some(800),
        };
        let cred2 = ExtendedCredentialInfo {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: Some(800),
        };
        let creds = vec![cred1, cred2];
        let self_attested_attrs = json!({
            "self_attested_attr_3": "my self attested 1",
            "self_attested_attr_4": "my self attested 2",
        }).to_string();

        let test: Value = json!({
              "self_attested_attributes":{
                  "self_attested_attr_3": "my self attested 1",
                  "self_attested_attr_4": "my self attested 2",
              },
              "requested_attributes":{
                  "height_1": {"cred_id": LICENCE_CRED_ID, "revealed": true, "timestamp": 800},
                  "zip_2": {"cred_id": ADDRESS_CRED_ID, "revealed": true, "timestamp": 800},
              },
              "requested_predicates":{}
        });

        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "height_1": {
                    "name": "height_1",
                    "non_revoked":  {"from": 123, "to": 456}
                },
                "zip_2": { "name": "zip_2" }
            },
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": 123}
        });
        let proof_req: ProofRequest = serde_json::from_value(proof_req).unwrap();
        let requested_credential = IndyHolder::build_requested_credentials_json(&creds, &self_attested_attrs, &proof_req).unwrap();
        assert_eq!(test.to_string(), requested_credential);
    }

    #[test]
    fn test_get_proof_request() {
        let _setup = SetupMocks::init();

        let connection_h = connection::tests::build_test_connection();

        let request = get_proof_request(connection_h, "123").unwrap();
        let _request: ProofRequestMessage = serde_json::from_str(&request).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_retrieve_credentials() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        crate::utils::libindy::anoncreds::tests::create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let (_, _, req, _) = crate::utils::libindy::anoncreds::tests::create_proof();

        let mut proof_req = ProofRequestMessage::create();
        let mut proof: DisclosedProof = Default::default();
        proof_req.proof_request_data = serde_json::from_str(&req).unwrap();
        proof.proof_request = Some(proof_req);

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds.len() > 500);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_retrieve_credentials_emtpy() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let mut req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({}),
           "requested_predicates": json!({}),
        });
        let mut proof_req = ProofRequestMessage::create();
        let mut proof: DisclosedProof = Default::default();
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req.clone());

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert_eq!(retrieved_creds, "{}".to_string());

        req["requested_attributes"]["address1_1"] = json!({"name": "address1"});
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req);
        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert_eq!(retrieved_creds, json!({"attrs":{"address1_1":[]}}).to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_case_for_proof_req_doesnt_matter_for_retrieve_creds() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        crate::utils::libindy::anoncreds::tests::create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let mut req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "zip_1": json!({
                   "name":"zip",
                   "restrictions": [json!({ "issuer_did": did })]
               })
           }),
           "requested_predicates": json!({}),
        });

        let mut proof_req = ProofRequestMessage::create();
        let mut proof: DisclosedProof = Default::default();
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req.clone());

        // All lower case
        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds.contains(r#""zip":"84000""#));
        let ret_creds_as_value: Value = serde_json::from_str(&retrieved_creds).unwrap();
        assert_eq!(ret_creds_as_value["attrs"]["zip_1"][0]["cred_info"]["attrs"]["zip"], "84000");
        // First letter upper
        req["requested_attributes"]["zip_1"]["name"] = json!("Zip");
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req.clone());
        let retrieved_creds2 = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds2.contains(r#""zip":"84000""#));

        //entire word upper
        req["requested_attributes"]["zip_1"]["name"] = json!("ZIP");
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req.clone());
        let retrieved_creds3 = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds3.contains(r#""zip":"84000""#));
    }

    #[test]
    fn test_retrieve_credentials_fails_with_no_proof_req() {
        let _setup = SetupLibraryWallet::init();

        let proof: DisclosedProof = Default::default();
        assert_eq!(proof.retrieve_credentials().unwrap_err().kind(), VcxErrorKind::InvalidState);
    }

    #[test]
    fn test_credential_def_identifiers() {
        let _setup = SetupDefaults::init();

        let cred1 = ExtendedCredentialInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: Some(NonRevokedInterval { from: Some(123), to: Some(456) }),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            timestamp: None,
        };
        let selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "schema_id": SCHEMA_ID,
                       "cred_def_id": CRED_DEF_ID,
                       "rev_reg_id":REV_REG_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string(),
              },
           },
           "predicates":{ }
        });
        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "height_1": { "name": "height", "non_revoked": {"from": 123, "to": 456} }
            },
            "requested_predicates": {},
            "non_revoked": {"to": 987}
        }).to_string();

        let creds = IndyHolder::map_selected_credentials(&selected_credentials.to_string(), &serde_json::from_str(&proof_req).unwrap()).unwrap();
        assert_eq!(creds, vec![cred1]);
    }

    #[test]
    fn test_credential_def_identifiers_failure() {
        let _setup = SetupDefaults::init();

        // selected credentials has incorrect json
        assert_eq!(IndyHolder::map_selected_credentials("", &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);


        // No Creds
        assert_eq!(IndyHolder::map_selected_credentials("{}", &proof_req_no_interval()).unwrap(), Vec::new());
        assert_eq!(IndyHolder::map_selected_credentials(r#"{"attrs":{}}"#, &proof_req_no_interval()).unwrap(), Vec::new());

        // missing cred info
        let selected_credentials: Value = json!({
           "attrs":{
              "height_1":{ "interval":null }
           },
           "predicates":{

           }
        });
        assert_eq!(IndyHolder::map_selected_credentials(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // Optional Revocation
        let mut selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "schema_id": SCHEMA_ID,
                       "cred_def_id": CRED_DEF_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string(),
              },
           },
           "predicates":{ }
        });
        let creds = vec![ExtendedCredentialInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            timestamp: None,
        }];
        assert_eq!(&IndyHolder::map_selected_credentials(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap(), &creds);

        // rev_reg_id is null
        selected_credentials["attrs"]["height_1"]["cred_info"]["rev_reg_id"] = serde_json::Value::Null;
        assert_eq!(&IndyHolder::map_selected_credentials(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap(), &creds);

        // Missing schema ID
        let mut selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "cred_def_id": CRED_DEF_ID,
                       "rev_reg_id":REV_REG_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{ }
        });
        assert_eq!(IndyHolder::map_selected_credentials(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // Schema Id is null
        selected_credentials["attrs"]["height_1"]["cred_info"]["schema_id"] = serde_json::Value::Null;
        assert_eq!(IndyHolder::map_selected_credentials(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_proof() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        crate::utils::libindy::anoncreds::tests::create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let mut proof_req = ProofRequestMessage::create();
        let to = time::get_time().sec;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": did}],
                    "non_revoked":  {"from": 123, "to": to}
                },
                "zip_2": { "name": "zip" }
            },
            "self_attested_attr_3": json!({
                   "name":"self_attested_attr",
             }),
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": to}
        }).to_string();
        proof_req.proof_request_data = serde_json::from_str(&indy_proof_req).unwrap();

        let mut proof: DisclosedProof = Default::default();
        proof.proof_request = Some(proof_req);
        proof.link_secret_alias = "main".to_string();

        let all_creds: Value = serde_json::from_str(&proof.retrieve_credentials().unwrap()).unwrap();
        let selected_credentials: Value = json!({
           "attrs":{
              "address1_1": {
                "credential": all_creds["attrs"]["address1_1"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
              "zip_2": {
                "credential": all_creds["attrs"]["zip_2"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{ }
        });

        let self_attested: Value = json!({
              "self_attested_attr_3":"attested_val"
        });

        let generated_proof = proof.generate_proof(&selected_credentials.to_string(), &self_attested.to_string());
        assert!(generated_proof.is_ok());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_self_attested_proof() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let mut proof_req = ProofRequestMessage::create();
        let indy_proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
               }),
               "zip_2": json!({
                   "name":"zip",
               }),
           }),
           "requested_predicates": json!({}),
        }).to_string();
        proof_req.proof_request_data = serde_json::from_str(&indy_proof_req).unwrap();

        let selected_credentials: Value = json!({});

        let self_attested: Value = json!({
              "address1_1":"attested_address",
              "zip_2": "attested_zip"
        });

        let mut proof: DisclosedProof = Default::default();
        proof.proof_request = Some(proof_req);
        proof.link_secret_alias = "main".to_string();
        let generated_proof = proof.generate_proof(&selected_credentials.to_string(), &self_attested.to_string());

        assert!(generated_proof.is_ok());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_proof_with_predicates() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        crate::utils::libindy::anoncreds::tests::create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let mut proof_req = ProofRequestMessage::create();
        let to = time::get_time().sec;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": did}],
                    "non_revoked":  {"from": 123, "to": to}
                },
                "zip_2": { "name": "zip" }
            },
            "self_attested_attr_3": json!({
                   "name":"self_attested_attr",
             }),
            "requested_predicates": json!({
                "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
            }),
            "non_revoked": {"from": 098, "to": to}
        }).to_string();
        proof_req.proof_request_data = serde_json::from_str(&indy_proof_req).unwrap();

        let mut proof: DisclosedProof = Default::default();
        proof.proof_request = Some(proof_req);
        proof.link_secret_alias = "main".to_string();

        let all_creds: Value = serde_json::from_str(&proof.retrieve_credentials().unwrap()).unwrap();
        let selected_credentials: Value = json!({
           "attrs":{
              "address1_1": {
                "credential": all_creds["attrs"]["address1_1"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
              "zip_2": {
                "credential": all_creds["attrs"]["zip_2"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{
               "zip_3": {
                "credential": all_creds["attrs"]["zip_3"][0],
               }
           }
        });

        let self_attested: Value = json!({
              "self_attested_attr_3":"attested_val"
        });

        let generated_proof = proof.generate_proof(&selected_credentials.to_string(), &self_attested.to_string());
        assert!(generated_proof.is_ok());
    }

    #[test]
    fn test_generate_reject_proof() {
        let _setup = SetupMocks::init();

        let proof: DisclosedProof = Default::default();
        let generated_reject = proof.generate_reject_proof_msg();
        assert!(generated_reject.is_ok());
    }

    #[test]
    fn test_build_rev_states_json() {
        let _setup = SetupMocks::init();

        let cred1 = ExtendedCredentialInfo {
            requested_attr: "height".to_string(),
            referent: "abc".to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        let mut cred_info = vec![cred1];
        let states = IndyHolder::build_rev_states_json(cred_info.as_mut()).unwrap();
        let rev_state_json: Value = serde_json::from_str(REV_STATE_JSON).unwrap();
        let expected = json!({REV_REG_ID: {"1": rev_state_json}}).to_string();
        assert_eq!(states, expected);
        assert!(cred_info[0].timestamp.is_some());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_empty() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        // empty vector
        assert_eq!(IndyHolder::build_rev_states_json(Vec::new().as_mut()).unwrap(), "{}".to_string());

        // no rev_reg_id
        let cred1 = ExtendedCredentialInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        assert_eq!(IndyHolder::build_rev_states_json(vec![cred1].as_mut()).unwrap(), "{}".to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_real_no_cache() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (schema_id, _, cred_def_id, _, _, _, _, cred_id, rev_reg_id, cred_rev_id) =
            crate::utils::libindy::anoncreds::tests::create_and_store_credential(attrs, true);
        let cred2 = ExtendedCredentialInfo {
            requested_attr: "height".to_string(),
            referent: cred_id,
            schema_id,
            cred_def_id,
            rev_reg_id: rev_reg_id.clone(),
            cred_rev_id,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        let rev_reg_id = rev_reg_id.unwrap();

        // assert cache is empty
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache.rev_state, None);

        let states = IndyHolder::build_rev_states_json(vec![cred2].as_mut()).unwrap();
        assert!(states.contains(&rev_reg_id));

        // check if this value is in cache now.
        let states: Value = serde_json::from_str(&states).unwrap();
        let state: HashMap<String, Value> = serde_json::from_value(states[&rev_reg_id].clone()).unwrap();

        let cache = get_rev_reg_cache(&rev_reg_id);
        let cache_rev_state = cache.rev_state.unwrap();
        let cache_rev_state_value: Value = serde_json::from_str(&cache_rev_state.value).unwrap();
        assert_eq!(cache_rev_state.timestamp, state.keys().next().unwrap().parse::<u64>().unwrap());
        assert_eq!(cache_rev_state_value.to_string(), state.values().next().unwrap().to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_real_cached() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let current_timestamp = time::get_time().sec as u64;
        let cached_rev_state = "{\"some\": \"json\"}".to_string();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (schema_id, _, cred_def_id, _, _, _, _, cred_id, rev_reg_id, cred_rev_id) =
            crate::utils::libindy::anoncreds::tests::create_and_store_credential(attrs, true);
        let cred2 = ExtendedCredentialInfo {
            requested_attr: "height".to_string(),
            referent: cred_id,
            schema_id,
            cred_def_id,
            rev_reg_id: rev_reg_id.clone(),
            cred_rev_id,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        let rev_reg_id = rev_reg_id.unwrap();

        let cached_data = RevRegCache {
            rev_state: Some(RevState {
                timestamp: current_timestamp,
                value: cached_rev_state.clone(),
            })
        };
        set_rev_reg_cache(&rev_reg_id, &cached_data);

        // assert data is successfully cached.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        let states = IndyHolder::build_rev_states_json(vec![cred2].as_mut()).unwrap();
        assert!(states.contains(&rev_reg_id));

        // assert cached data is unchanged.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        // check if this value is in cache now.
        let states: Value = serde_json::from_str(&states).unwrap();
        let state: HashMap<String, Value> = serde_json::from_value(states[&rev_reg_id].clone()).unwrap();

        let cache_rev_state = cache.rev_state.unwrap();
        let cache_rev_state_value: Value = serde_json::from_str(&cache_rev_state.value).unwrap();
        assert_eq!(cache_rev_state.timestamp, state.keys().next().unwrap().parse::<u64>().unwrap());
        assert_eq!(cache_rev_state_value.to_string(), state.values().next().unwrap().to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_real_with_older_cache() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let current_timestamp = time::get_time().sec as u64;
        let cached_timestamp = current_timestamp - 100;
        let cached_rev_state = "{\"witness\":{\"omega\":\"2 0BB3DE371F14384496D1F4FEB47B86A935C858BC21033B16251442FCBC5370A1 2 026F2848F2972B74079BEE16CDA9D48AD2FF7C7E39087515CB9B6E9B38D73BCB 2 10C48056D8C226141A8D7030E9FA17B7F02A39B414B9B64B6AECDDA5AFD1E538 2 11DCECD73A8FA6CFCD0468C659C2F845A9215842B69BA10355C1F4BF2D9A9557 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000\"},\"rev_reg\":{\"accum\":\"2 033C0E6FAC660DF3582EF46021FAFDD93E111D1DC9DA59C4EA9B92BB21F8E0A4 2 02E0F749312228A93CF67BB5F86CA263FAE535A0F1CA449237D736939518EFF0 2 19BB82474D0BD0A1DDE72D377C8A965D6393071118B79D4220D4C9B93D090314 2 1895AAFD8050A8FAE4A93770C6C82881AB13134EE082C64CF6A7A379B3F6B217 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000\"},\"timestamp\":100}".to_string();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (schema_id, _, cred_def_id, _, _, _, _, cred_id, rev_reg_id, cred_rev_id) =
            crate::utils::libindy::anoncreds::tests::create_and_store_credential(attrs, true);
        let cred2 = ExtendedCredentialInfo {
            requested_attr: "height".to_string(),
            referent: cred_id,
            schema_id,
            cred_def_id,
            rev_reg_id: rev_reg_id.clone(),
            cred_rev_id,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: Some(NonRevokedInterval { from: Some(cached_timestamp + 1), to: None }),
            timestamp: None,
        };
        let rev_reg_id = rev_reg_id.unwrap();

        let cached_data = RevRegCache {
            rev_state: Some(RevState {
                timestamp: cached_timestamp,
                value: cached_rev_state.clone(),
            })
        };
        set_rev_reg_cache(&rev_reg_id, &cached_data);

        // assert data is successfully cached.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        let states = IndyHolder::build_rev_states_json(vec![cred2].as_mut()).unwrap();
        assert!(states.contains(&rev_reg_id));

        // assert cached data is updated.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_ne!(cache, cached_data);

        // check if this value is in cache now.
        let states: Value = serde_json::from_str(&states).unwrap();
        let state: HashMap<String, Value> = serde_json::from_value(states[&rev_reg_id].clone()).unwrap();

        let cache_rev_state = cache.rev_state.unwrap();
        let cache_rev_state_value: Value = serde_json::from_str(&cache_rev_state.value).unwrap();
        assert_eq!(cache_rev_state.timestamp, state.keys().next().unwrap().parse::<u64>().unwrap());
        assert_eq!(cache_rev_state_value.to_string(), state.values().next().unwrap().to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_real_with_newer_cache() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let current_timestamp = time::get_time().sec as u64;
        let cached_timestamp = current_timestamp + 100;
        let cached_rev_state = "{\"witness\":{\"omega\":\"2 0BB3DE371F14384496D1F4FEB47B86A935C858BC21033B16251442FCBC5370A1 2 026F2848F2972B74079BEE16CDA9D48AD2FF7C7E39087515CB9B6E9B38D73BCB 2 10C48056D8C226141A8D7030E9FA17B7F02A39B414B9B64B6AECDDA5AFD1E538 2 11DCECD73A8FA6CFCD0468C659C2F845A9215842B69BA10355C1F4BF2D9A9557 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000\"},\"rev_reg\":{\"accum\":\"2 033C0E6FAC660DF3582EF46021FAFDD93E111D1DC9DA59C4EA9B92BB21F8E0A4 2 02E0F749312228A93CF67BB5F86CA263FAE535A0F1CA449237D736939518EFF0 2 19BB82474D0BD0A1DDE72D377C8A965D6393071118B79D4220D4C9B93D090314 2 1895AAFD8050A8FAE4A93770C6C82881AB13134EE082C64CF6A7A379B3F6B217 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000\"},\"timestamp\":100}".to_string();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (schema_id, _, cred_def_id, _, _, _, _, cred_id, rev_reg_id, cred_rev_id) =
            crate::utils::libindy::anoncreds::tests::create_and_store_credential(attrs, true);
        let cred2 = ExtendedCredentialInfo {
            requested_attr: "height".to_string(),
            referent: cred_id,
            schema_id,
            cred_def_id,
            rev_reg_id: rev_reg_id.clone(),
            cred_rev_id,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: Some(NonRevokedInterval { from: None, to: Some(cached_timestamp - 1) }),
            timestamp: None,
        };
        let rev_reg_id = rev_reg_id.unwrap();

        let cached_data = RevRegCache {
            rev_state: Some(RevState {
                timestamp: cached_timestamp,
                value: cached_rev_state.clone(),
            })
        };
        set_rev_reg_cache(&rev_reg_id, &cached_data);

        // assert data is successfully cached.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        let states = IndyHolder::build_rev_states_json(vec![cred2].as_mut()).unwrap();
        assert!(states.contains(&rev_reg_id));

        // assert cached data is unchanged.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        // check if this value is not in cache.
        let states: Value = serde_json::from_str(&states).unwrap();
        let state: HashMap<String, Value> = serde_json::from_value(states[&rev_reg_id].clone()).unwrap();

        let cache_rev_state = cache.rev_state.unwrap();
        let cache_rev_state_value: Value = serde_json::from_str(&cache_rev_state.value).unwrap();
        assert_ne!(cache_rev_state.timestamp, state.keys().next().unwrap().parse::<u64>().unwrap());
        assert_ne!(cache_rev_state_value.to_string(), state.values().next().unwrap().to_string());
    }

    #[test]
    fn test_get_credential_intervals_from_proof_req() {
        let _setup = SetupDefaults::init();

        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "non_revoked":  {"from": 123, "to": 456}
                },
                "zip_2": { "name": "zip" }
            },
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": 123}
        });
        let proof_req: ProofRequest = serde_json::from_value(proof_req).unwrap();

        // Attribute not found in proof req
        assert_eq!(proof_req.get_revocation_interval("not here").unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // attribute interval overrides proof request interval
        let interval = Some(NonRevokedInterval { from: Some(123), to: Some(456) });
        assert_eq!(proof_req.get_revocation_interval("address1_1").unwrap(), interval);

        // when attribute interval is None, defaults to proof req interval
        let interval = Some(NonRevokedInterval { from: Some(098), to: Some(123) });
        assert_eq!(proof_req.get_revocation_interval("zip_2").unwrap(), interval);

        // No interval provided for attribute or proof req
        assert_eq!(proof_req_no_interval().get_revocation_interval("address1_1").unwrap(), None);
    }
}
