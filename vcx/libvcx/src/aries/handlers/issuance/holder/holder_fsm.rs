use std::collections::HashMap;

use crate::api::VcxStateType;
use crate::aries::handlers::{
    connection::{
        Connection,
        types::CompletedConnection,
        agent::AgentInfo,
    },
    issuance::holder::{
        states::*,
        messages::HolderMessages,
    },
};
use crate::aries::messages::{
    a2a::A2AMessage,
    error::{ProblemReport, ProblemReportCodes, Reason},
    status::Status,
    issuance::{
        credential::Credential,
        credential_offer::CredentialOffer,
        credential_request::CredentialRequest,
        credential_ack::CredentialAck,
        v10::credential_request::CredentialRequest as CredentialRequestV1,
        v20::credential_request::CredentialRequest as CredentialRequestV2,
    },
};
use crate::error::prelude::*;
use crate::utils::object_cache::Handle;
use crate::connection::Connections;
use crate::{credential, settings};
use crate::utils::libindy::{
    anoncreds::{libindy_prover_store_credential, libindy_prover_delete_credential, prover_get_credential},
    types::CredentialInfo,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HolderSM {
    state: HolderState,
    source_id: String,
}

impl HolderSM {
    pub fn new(offer: CredentialOffer, source_id: String) -> Self {
        HolderSM {
            state: HolderState::OfferReceived(OfferReceivedState::new(offer)),
            source_id,
        }
    }

    pub fn get_source_id(&self) -> &String {
        &self.source_id
    }

    pub fn state(&self) -> u32 {
        match self.state {
            HolderState::OfferReceived(_) => VcxStateType::VcxStateRequestReceived as u32,
            HolderState::RequestSent(_) => VcxStateType::VcxStateOfferSent as u32,
            HolderState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    Status::Rejected(_) => VcxStateType::VcxStateRejected as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
        }
    }

    pub fn update_state(self) -> VcxResult<Self> {
        trace!("Holder::update_state >>> ");

        if self.is_terminal_state() { return Ok(self); }

        let agent = match self.get_agent_info() {
            Some(agent_info) => agent_info.clone(),
            None => {
                warn!("Could not update Holder state: no information about Connection.");
                return Ok(self);
            }
        };

        let messages = agent.get_messages()?;

        match self.find_message_to_handle(messages) {
            Some((uid, msg)) => {
                let state = self.handle_message(msg.into())?;
                agent.update_message_status(uid, None)?;
                Ok(state)
            }
            None => Ok(self)
        }
    }

    fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("Holder::find_message_to_handle >>> agent: {:?}", secret!(messages));
        debug!("Holder: Finding message to update state");

        for (uid, message) in messages {
            match self.state {
                HolderState::OfferReceived(_) => {
                    // do not process agent
                }
                HolderState::RequestSent(ref state) => {
                    match message {
                        A2AMessage::Credential(credential) => {
                            if credential.from_thread(state.thread.thid.as_deref().unwrap_or_default()) {
                                debug!("Holder: Credential message received");
                                return Some((uid, A2AMessage::Credential(credential)));
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) |
                        A2AMessage::CredentialReject(problem_report) => {
                            if problem_report.from_thread(&state.thread.thid.as_deref().unwrap_or_default()) {
                                debug!("Holder: CredentialReject message received");
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        message => {
                            warn!("Holder: Unexpected message received in RequestSent state: {:?}", message);
                        }
                    }
                }
                HolderState::Finished(_) => {
                    // do not process agent
                }
            };
        }
        debug!("Holder: no message to update state");
        None
    }

    pub fn step(state: HolderState, source_id: String) -> Self {
        HolderSM { state, source_id }
    }

    pub fn handle_message(self, cim: HolderMessages) -> VcxResult<HolderSM> {
        trace!("Holder::handle_message >>> cim: {:?}", secret!(cim));
        debug!("Holder: Updating state");

        let HolderSM { state, source_id } = self;
        let state = match state {
            HolderState::OfferReceived(state_data) => match cim {
                HolderMessages::CredentialRequestSend(connection_handle) => {
                    state_data.send_credential_request(connection_handle)?
                }
                HolderMessages::CredentialRejectSend((connection_handle, comment)) => {
                    state_data.send_credential_reject(connection_handle, comment)?
                }
                _ => {
                    warn!("Credential Issuance can only start on holder side with Credential Offer");
                    HolderState::OfferReceived(state_data)
                }
            },
            HolderState::RequestSent(state_data) => match cim {
                HolderMessages::Credential(credential) => {
                    state_data.handle_received_credential(credential)?
                }
                HolderMessages::ProblemReport(problem_report) => {
                    let thread = problem_report.thread.clone()
                        .update_received_order(&state_data.connection.data.did_doc.id);
                    HolderState::Finished((state_data, problem_report, thread, Reason::Fail).into())
                }
                HolderMessages::CredentialRejectSend((connection_handle, comment)) => {
                    state_data.send_credential_reject(connection_handle, comment)?
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Credential and Problem Report");
                    HolderState::RequestSent(state_data)
                }
            },
            HolderState::Finished(state_data) => {
                warn!("Exchange is finished, no agent can be sent or received");
                HolderState::Finished(state_data)
            }
        };

        trace!("Holder::handle_message <<< state: {:?}", secret!(state));
        Ok(HolderSM::step(state, source_id))
    }

    pub fn is_terminal_state(&self) -> bool {
        match self.state {
            HolderState::Finished(_) => true,
            _ => false
        }
    }

    pub fn get_credential_offer(&self) -> VcxResult<CredentialOffer> {
        match self.state {
            HolderState::OfferReceived(ref state) => Ok(state.offer.clone()),
            HolderState::RequestSent(ref state) => state.offer.clone().ok_or(
                VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Holder object state: `offer` not found", self.source_id))),
            HolderState::Finished(ref state) => state.offer.clone().ok_or(
                VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Holder object state: `offer` not found", self.source_id))),
        }
    }

    pub fn get_credential(&self) -> VcxResult<(String, Credential)> {
        match self.state {
            HolderState::Finished(ref state) => {
                let cred_id = state.cred_id.clone()
                    .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Holder object state: `cred_id` not found", self.source_id)))?;
                let credential = state.credential.clone()
                    .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Holder object state: `credential` not found", self.source_id)))?;
                Ok((cred_id, credential))
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                        format!("Holder object {} in state {} not ready to get Credential message", self.source_id, self.state())))
        }
    }

    pub fn delete_credential(&self) -> VcxResult<()> {
        trace!("Holder::delete_credential >>>");

        match self.state {
            HolderState::Finished(ref state) => {
                let cred_id = state.cred_id.as_deref()
                    .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Holder object state: `cred_id` not found", self.source_id)))?;
                state.delete_credential(&cred_id)
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                        format!("Holder object {} in state {} not ready to delete Credential", self.source_id, self.state())))
        }
    }

    pub fn get_agent_info(&self) -> Option<&AgentInfo> {
        match self.state {
            HolderState::RequestSent(ref state) => Some(&state.connection.agent),
            HolderState::OfferReceived(_) => None,
            HolderState::Finished(_) => None,
        }
    }

    pub fn problem_report(&self) -> Option<&ProblemReport> {
        match self.state {
            HolderState::OfferReceived(_) |
            HolderState::RequestSent(_) => None,
            HolderState::Finished(ref status) => {
                match &status.status {
                    Status::Success | Status::Undefined => None,
                    Status::Rejected(ref problem_report) => problem_report.as_ref(),
                    Status::Failed(problem_report) => Some(problem_report),
                }
            }
        }
    }

    pub fn get_info(&self) -> VcxResult<CredentialInfo> {
        match self.state {
            HolderState::OfferReceived(_) |
            HolderState::RequestSent(_) => {
                Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                       format!("Holder object {} in state {} not ready to get information about stored credential", self.source_id, self.state())))
            }
            HolderState::Finished(ref state) => {
                match state.cred_id.as_ref() {
                    Some(cred_id) => prover_get_credential(&cred_id),
                    None => {
                        Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                               format!("Holder object {} in state {} not ready to get information about stored credential", self.source_id, self.state())))
                    }
                }
            }
        }
    }
}

fn _store_credential(credential_offer: &CredentialOffer,
                     credential: &Credential,
                     req_meta: &str,
                     cred_def_json: &str) -> VcxResult<String> {
    credential.ensure_match_offer(credential_offer)?;
    let (_, credential_json) = credential.credentials_attach().content()?;
    let cred_id = libindy_prover_store_credential(None,
                                                  req_meta,
                                                  &credential_json,
                                                  cred_def_json)?;
    trace!("Holder::_store_credential <<<");
    Ok(cred_id)
}

impl RequestSentState {
    fn handle_received_credential(self, credential: Credential) -> VcxResult<HolderState> {
        let thread = credential.thread().clone()
            .increment_sender_order()
            .update_received_order(&self.connection.data.did_doc.id);

        match self.store_credential(&credential) {
            Ok(cred_id) => {
                if credential.please_ack().is_some() {
                    let ack =
                        CredentialAck::create()
                            .set_message_type(credential.type_())
                            .set_thread(thread.clone());
                    self.connection.data.send_message(&ack, &self.connection.agent)?;
                }
                Ok(HolderState::Finished((self, cred_id, credential, thread).into()))
            }
            Err(err) => {
                let problem_report = ProblemReport::create()
                    .set_message_type(credential.type_())
                    .set_description(ProblemReportCodes::InvalidCredential)
                    .set_comment(format!("error occurred: {:?}", err))
                    .set_thread(thread.clone());

                self.connection.data.send_message(&problem_report, &self.connection.agent)?;
                return Err(err);
            }
        }
    }

    fn send_credential_reject(self, connection_handle: Handle<Connections>, comment: Option<String>) -> VcxResult<HolderState> {
        let connection: CompletedConnection = connection_handle.get_completed_connection()?;

        let thread = self.thread.clone()
            .increment_sender_order()
            .update_received_order(&connection.data.did_doc.id);

        let offer = self.offer.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState,
                                      format!("Invalid Holder object state: `offer` not found")))?;

        let problem_report = ProblemReport::create()
            .set_message_type(offer.type_())
            .set_description(ProblemReportCodes::CredentialRejected)
            .set_comment(comment.unwrap_or(String::from("credential-offer was rejected")))
            .set_thread(thread.clone());

        connection.agent.send_message(&problem_report, &connection.data.did_doc)?;

        Ok(HolderState::Finished((self, problem_report, thread, Reason::Reject).into()))
    }

    fn store_credential(&self, credential: &Credential) -> VcxResult<String> {
        trace!("Holder::_store_credential >>>");
        debug!("holder storing received credential");

        self.thread.check_message_order(&self.connection.data.did_doc.id, &credential.thread())?;

        let credential_offer = self.offer.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState,
                                      format!("Invalid Holder object state: `offer` not found")))?;

        _store_credential(&credential_offer,
                          &credential,
                          &self.req_meta,
                          &self.cred_def_json)
    }
}

impl OfferReceivedState {
    fn send_credential_request(self, connection_handle: Handle<Connections>) -> VcxResult<HolderState> {
        if connection_handle == 0 && self.offer.service().is_some() {
            self.handle_ephemeral_credential_offer()
        } else {
            self.handle_credential_offer(connection_handle)
        }
    }

    fn send_credential_reject(self, connection_handle: Handle<Connections>, comment: Option<String>) -> VcxResult<HolderState> {
        let connection: CompletedConnection = connection_handle.get_completed_connection()?;

        let thread = self.thread.clone()
            .update_received_order(&connection.data.did_doc.id);

        let problem_report = ProblemReport::create()
            .set_message_type(self.offer.type_())
            .set_description(ProblemReportCodes::CredentialRejected)
            .set_comment(comment.unwrap_or(String::from("credential-offer was rejected")))
            .set_thread(thread.clone());

        connection.agent.send_message(&problem_report, &connection.data.did_doc)?;

        Ok(HolderState::Finished((self, problem_report, thread, Reason::Reject).into()))
    }

    fn make_credential_request(&self) -> VcxResult<(CredentialRequest, String, String)> {
        trace!("Holder::OfferReceivedState::make_credential_request >>> offer: {:?}", secret!(self.offer));
        debug!("holder preparing credential request");

        let did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        let (_, cred_offer) = self.offer.offer_attach().content()?;
        let cred_def_id = self._parse_cred_def_from_cred_offer(&cred_offer)?;

        let (req, req_meta, _cred_def_id, cred_def_json) =
            credential::Credential::create_credential_request(&cred_def_id, &did, &cred_offer)?;

        self.offer.ensure_match_credential_definition(&cred_def_json)?;

        let cred_req = match self.offer {
            CredentialOffer::V1(_) => {
                CredentialRequest::V1(
                    CredentialRequestV1::create()
                        .set_requests_attach(req)?
                )
            }
            CredentialOffer::V2(_) => {
                CredentialRequest::V2(
                    CredentialRequestV2::create()
                        .set_indy_requests_attach(&req)?
                )
            }
        };

        trace!("Holder::make_credential_request <<<");
        Ok((cred_req, req_meta, cred_def_json))
    }

    fn handle_ephemeral_credential_offer(self) -> VcxResult<HolderState> {
        trace!("Holder::OfferReceivedState::handle_ephemeral_credential_offer >>> offer: {:?}", secret!(self.offer));

        let (cred_request, req_meta, cred_def_json) = self.make_credential_request()?;

        let thread = self.thread.clone();

        let cred_request = cred_request
            .request_return_route()
            .set_thread(thread.clone());

        let did_doc = self.offer.service().cloned().unwrap().into();
        let message = Connection::send_message_and_wait_result(&cred_request, &did_doc)?;

        match message {
            A2AMessage::Credential(credential) => {
                let cred_id = _store_credential(&self.offer, &credential, &req_meta, &cred_def_json)?;
                Ok(HolderState::Finished((self, cred_id, credential, thread).into()))
            }
            A2AMessage::CredentialReject(problem_report) => {
                Ok(HolderState::Finished((self, problem_report, thread, Reason::Reject).into()))
            }
            _ => {
                let problem_report = ProblemReport::create()
                    .set_message_type(self.offer.type_())
                    .set_comment("Unexpected message received in response on Credential Request".to_string());
                Ok(HolderState::Finished((self, problem_report, thread, Reason::Reject).into()))
            }
        }
    }

    fn handle_credential_offer(self, connection_handle: Handle<Connections>) -> VcxResult<HolderState> {
        trace!("Holder::OfferReceivedState::handle_credential_offer >>> offer: {:?}", secret!(self.offer));

        let connection = connection_handle.get_completed_connection()?;
        let thread = self.thread.clone()
            .update_received_order(&connection.data.did_doc.id)
            .set_opt_pthid(connection.data.thread.pthid.clone());

        match self.make_credential_request() {
            Ok((cred_request, req_meta, cred_def_json)) => {
                let cred_request = cred_request.set_thread(thread.clone());
                connection.data.send_message(&cred_request, &connection.agent)?;
                Ok(HolderState::RequestSent((self, req_meta, cred_def_json, connection, thread).into()))
            }
            Err(err) => {
                let problem_report = ProblemReport::create()
                    .set_message_type(self.offer.type_())
                    .set_description(ProblemReportCodes::InvalidCredentialOffer)
                    .set_comment(format!("error occurred: {:?}", err))
                    .set_thread(thread.clone());

                connection.data.send_message(&problem_report, &connection.agent)?;
                return Err(err);
            }
        }
    }

    fn _parse_cred_def_from_cred_offer(&self, cred_offer: &str) -> VcxResult<String> {
        trace!("Holder::_parse_cred_def_from_cred_offer >>> cred_offer: {:?}", secret!(cred_offer));

        let parsed_offer: serde_json::Value = serde_json::from_str(cred_offer)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer, format!("Cannot parse Credential Offer from JSON string. Err: {:?}", err)))?;

        let cred_def_id = parsed_offer["cred_def_id"].as_str()
            .ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer, "Invalid Credential object state: `cred_def_id` not found"))?;

        Ok(cred_def_id.to_string())
    }
}

impl FinishedHolderState {
    fn delete_credential(&self, cred_id: &str) -> VcxResult<()> {
        trace!("Holder::_delete_credential >>> cred_id: {}", cred_id);
        libindy_prover_delete_credential(cred_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::utils::devsetup::SetupAriesMocks;
    use crate::aries::handlers::connection::tests::mock_connection;
    use crate::aries::test::source_id;
    use crate::aries::messages::issuance::v10::credential_offer::CredentialOffer as CredentialOfferV1;
    use crate::aries::messages::issuance::v10::credential::Credential as CredentialV1;
    use crate::aries::messages::issuance::credential::tests::_credential;
    use crate::aries::messages::issuance::v10::credential::tests::_credential as _credential_v1;
    use crate::aries::messages::issuance::v10::credential_offer::tests::_credential_offer as _credential_offer_v1;
    use crate::aries::messages::issuance::v10::credential_proposal::tests::_credential_proposal as _credential_proposal_v1;
    use crate::aries::messages::issuance::v10::credential_request::tests::_credential_request as _credential_request_v1;
    use crate::aries::messages::issuance::credential_request::tests::_credential_request;
    use crate::aries::messages::issuance::credential_offer::tests::_credential_offer;
    use crate::aries::messages::issuance::credential_proposal::tests::_credential_proposal;
    use crate::aries::messages::issuance::test::{_ack, _problem_report};


    fn _holder_sm() -> HolderSM {
        HolderSM::new(_credential_offer(), source_id())
    }

    impl HolderSM {
        fn to_request_sent_state(mut self) -> HolderSM {
            self = self.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();
            self
        }

        fn to_finished_state(mut self) -> HolderSM {
            self = self.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();
            self = self.handle_message(HolderMessages::Credential(_credential())).unwrap();
            self
        }
    }

    mod new {
        use super::*;
        use crate::aries::messages::thread::Thread;

        #[test]
        fn test_holder_new() {
            let _setup = SetupAriesMocks::init();

            let holder_sm = _holder_sm();

            assert_match!(HolderState::OfferReceived(_), holder_sm.state);
            assert_eq!(source_id(), holder_sm.get_source_id().to_string());
        }

        #[test]
        fn test_holder_new_for_offer_with_empty_thread() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let offer = r#"{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential","credential_preview":{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/credential-preview","attributes":[{"name":"family_name","value":"Ferreira"},{"name":"given_name","value":"A"},{"name":"birth_date","value":"01/01/2019"},{"name":"issue_date","value":"10/05/2021"},{"name":"expiry_date","value":"10/05/2023"},{"name":"issuing_country","value":"US"},{"name":"issuing_authority","value":"Department of Motor Vehicles"},{"name":"document_number","value":"F2741"},{"name":"administrative_number","value":"F2741"},{"name":"driving_privileges","value":"UNRESTR"},{"name":"un_distinguishing_sign","value":""},{"name":"gender","value":""},{"name":"height","value":"180"},{"name":"weight","value":"75"},{"name":"eye_color","value":"brown"},{"name":"hair_color","value":"brown"},{"name":"birth_place","value":"New York, New York, USA"},{"name":"resident_address","value":"Nowhere St."},{"name":"portrait","value":""},{"name":"portrait_capture_date","value":""},{"name":"age_in_years","value":"45"},{"name":"age_birth_year","value":"1976"},{"name":"age_over_18","value":"yes"},{"name":"age_over_21","value":"yes"},{"name":"age_over_25","value":"yes"},{"name":"issuing_jurisdiction","value":"USNY"},{"name":"nationality","value":"US"},{"name":"resident_city","value":"New York"},{"name":"resident_state","value":"New York"},{"name":"resident_postal_code","value":"10101"},{"name":"name_national_character","value":""},{"name":"signature_usual_mark","value":""}]},"~thread":{},"@id":"97faf3cb-cf17-4965-bccb-4d5af0e8d8da","offers~attach":[{"mime-type":"application/json","data":{"base64":"eyJzY2hlbWFfaWQiOiAiVGg3TXBUYVJaVlJZblBpYWJkczgxWToyOmNvbV9nY29tc29mdF9kbGQ6MC4xIiwgImNyZWRfZGVmX2lkIjogIlRoN01wVGFSWlZSWW5QaWFiZHM4MVk6MzpDTDo4MDgyOmRlZmF1bHQiLCAia2V5X2NvcnJlY3RuZXNzX3Byb29mIjogeyJjIjogIjE0NTgxMzEzNzEyNjgxMzQyNDg5ODY3NDAyNDkwMTg2MDM2OTY5OTE0NDQ4ODg1ODQzMTY1Njg2NDQ4MTczNzQ3NDc2NjA1MTY5MDgiLCAieHpfY2FwIjogIjMwNjY4Mjc0NDkzMzA2OTQyNjYzNjI3OTI1NzQ2MDM5NTYzNzk5ODgxNDc1OTUxMzE5ODQxNTMwOTQ4MDQzOTE1NjI3OTM4Njc1MTMzOTUyNjYyMzAxMzMzMDUzNjU1MzQxNjgxNDMzNDQ0ODc0Mjg5MTcxNTYyNjc0Mjg5ODUxNDIzNDIyMjk2NzI1NDI3MzU3MTE2NTc2NDYwNjc2NTA2MzA3Njc4MjA5NDUwNzQ2OTI5ODM4NjA4NzQ1Njc2MTE4MTc5Mjg0NDUyMTYzNDI4Njc5NDc0OTc2NTQ3NDU1OTkwNTY4MDg5MjI0Mzk5Nzk4NDQ4MzcwNjg4NzU3NzY3MzAzNTI3NTI5MjQzMTc4MzYxNTA4NDU2ODEzNTUwNTY1OTQ2ODQwOTM4MDA4NzY3NDczOTA3MjM0MDM5MDI2MDkxNTY2NDU4MjQzNzUwMjgwNDc5NTc5ODEzMjg2MzY2NjAzODI2ODU1NTg2ODYyMTE5NDY1NzM5NjIwNzQ2NjYwMjcxMTMzNzIzNjI2NzIwMjM0NTg0NDQ4NjY1MjE1NzIzMDYzOTYyNDk0NzA0MDQ0MDUyNzI4NDYyMTA0Nzk5MzAyMjg0OTY2NjIzNTQzNTgzNzc1ODM2Njc5MDExNTQxODYwNjg4MjUyNzU2NzQ1ODgxNjIzMDc4NTUzMTU4MDg2OTgyOTAwMzMyMjI4NDQ1OTE0MTk1Nzk3OTE4MTIzODMzODk4NjA2NjEyNjkzOTU5NTk5NTA5MjE1MDY5MTQ0MjA1Mjk1NDYyMjE4ODc0MzU4OTcxMzI3MDc1MjYzMDU1ODM2MTc0MTA1Nzk1NDQ3NjQ4OTAzMjAzMDg1NTU4NTQxNjU2MDEyOTYyOTExNDUzMzU5NDQwNzU4MzI5MTA0MDQ5MjAwMjE3OTIwOTI5NjM0IiwgInhyX2NhcCI6IFtbInBvcnRyYWl0IiwgIjE4MDM2MjEzMDYyMDAyMjk2ODYyNTM4NDgxODkwMDA4OTc3ODIyMDM0Mzg2MDM1NjkzMTc3MTgwNjU5NzQyMzEwMTA5MjA5Mjc1NzQ2MzYxNDcxNDU1NDM5ODkwNDQ4NTQ5MDc5NjI3ODczMjEzMzcxNDA3MTc0MDU1NDA2MTM0Nzg2NDg0NjExODU2NjQ3MzIzNzQzNjA3MTY1NTcyNTkzNTI5MjQwNTQzMzE4NTA2ODgxNDIwMjA4NzMxNDk4MjE5MjgzNjc5ODUxODIxNzU2ODgxOTYyNzAwNTEyODc1OTIwNDU4NTc1MTc4ODcyMTQwMTk5MTc5Nzk2Mjk5NzQ5NTAwMDc4MjMxNDUxMjY0OTg5MDg3ODk4NzU5OTQ0NTk1MzQ5OTc0Njk5OTYwNDE4ODE4NjA0OTA1NjY3NDc2NTQwNDAyNDgyNzQ4NDU4NzcxOTQyNzgxOTg4MzUxMjg0MDEzNDE1ODY0MDE2MDkwODg2MjE5MjQxMzkzMTc4OTA3MDY5MjgzNzg4MjgxNDk5NjY0ODA0NjY2MTU3NjY5NTc3MDY2MTAyMTgxMzkwNDA4ODk5ODg1MDM0MjIxODE5NzE1MjQ3Mjg0ODg2MTY4NjEyNDgxMzEwNjQ3Njk2OTc1NzM2MjA4MzU4MzIzNzU5MzIwOTc4MDAyNzQ5OTcwMDY4NDgwMzczMzIxNTQyODkxNDA0OTE2MTgxODM1MjI3NTQ0MTQ2MDk1NDI0MDAxNTA5NTY5MDc3MzE3MTI2NDQzNjM4NjUxNDM5MjA3MjQzNDc2ODc1MDMzNTA0MDg3OTk5ODQyMTcwNDkxMjM2NzQyODg3MTkwNTM5MjIwMzU4MDE5Mzg5MzgwMzEwMzI1NTEzMDAwNTkxODIzNDg4MjI0NjA2MzYzNTA4ODA1ODA5NDU0Il0sIFsiYWdlX292ZXJfMjEiLCAiNTU1NTA3MDc3OTk1NDQ2MzAxMjUxNDMwMDQwMjMwNzkwMDAyMzM5MDQxMjIyMjMxMzMwMDI0NDEzMTA3OTk1MjQ2MTYxNjI5MDgyNjIyNjM4NDc1OTgwNTM0ODc1MzE2MTIzNDk5MzE3OTgxODU0MzI3NzI2NDE0NzA1MTc3Nzc5MDQ5OTQ3MjkwODY3NjcxMDU0NTQ5NTYwODU5MTc3NDg3NDQ4MDU2Mzc3MTgyOTYwMjkxMDkyNjM5NjQ2MjAyNTMyOTE3NjkxNTgxNTQ3NTg2NTg5NzgzMDk1OTAxODIxODgzODc1MTE1NDAwODYzNjQxMjk1MTgxNzQ0MzIwNTgzMTQyMTM2NTkxNTQ1MDIzNzU0ODI5ODYyNjI2OTM4Mzc0ODA2NTQwMDMxMDEwNTk4NjA1ODU2ODQzNzM4MjM0NTU3NDc5OTQ2NTMyNTI0NTYxMjg3MzE0OTcyOTM5NDMzNDcyMDgyNjkwNDc0MTQwMzA3NDIyOTA5ODkxNzk4NDY5MzY3NDkzMzcyNDY5ODY4NjEzOTg0Mzc0Mjg2MTY3MTA4ODUwNzk4OTc4MDIzNTkyMDQzNTU5MTY0NjU2NDc2OTA4NTc5MzA4MDUyMzgxNzM1MzA4NDQ2MzI3MTU1NzU4NTU3NTgwMDA0NzU1ODUyODg3NDg2MTg5MzkyMjMxMDI5NDQxMDA4MTI5ODg1MDk3MTcyNDM4Mjg0NzAzNTE0NDcxMzEzMDcxMTM5NDAwNzc5ODU0ODE0OTMwNTgyMTY2MTk0NjU2MjMzMDczMzExNjA4ODg3NTQ1ODgzMTQ3MjU1NzAwNTE3NTg4NDU2MzUxMzU2OTgzMjU3ODk0Mjg4Nzc0Njg3MDk5MDM5MDMzNTQwMTQ4NDkwMTk3NzkyMjkxODc5NjgyNzczMTM4MjUzNyJdLCBbImlzc3VpbmdfanVyaXNkaWN0aW9uIiwgIjE0NjY3NTEyNDcyNjcxMDg2Njc3MDkwNTk5MzM1MzY2NDQxNjM3ODYwMjkzNzU3MzcyNzAwODAwMzgyOTAyMDA3OTE2NTI3MDI4Nzg3NTQ5ODQwNTI5MzE5MTc0NDg4MjQ3NzE5MjQ0MzI2NzY4NDYxMDU5NzQwNjI1NjAyMzQ3MzEwNzQ2ODUwOTI2NDAyMTQwNTAxNDg1NTg3MDI5NDkwMDQ2ODczMjkxOTU5NzgwNjg1MTkwMDQ5NjQzMzY3MjE0MTQ2MTU1NjEzMDY4MTkyODg1NTM3MjE4NDgxNzMzMTEwMDA4OTg4NDU5MjIzNTIyODU2NTA2NDE4NzMwNzk0OTQ3MjkzNDQyMDc5NTk2MTA0MTk4MjE1NjA5NDg3ODQ2Nzk4NDg4Nzg4MjQwNzI3MDY2NzU5MDEyMTI1MjQxNTY1ODEzMzYzMDA0NDk5MzIyNDE5ODQxNTg1OTAwMzAzMzYyNTIwOTA5Nzk4Njc3NDA4MDI3NzUwNjUxMzQ5MDE1MzExMTk3MjkxMTk5NTczMDg4NTM0MTY5MzI0MDQzMDEyMTk3NDQyODUzMzcwMjYwNDEyMDE4OTY3NjUyNTM4MDM5MTUwNDgzMzcwMzM0NTgzMjQyOTQ2NTc0NDkyOTQ3MjMyMDIzNjUyNjE3NDI0ODI5NDY1NDgwNzAyOTk1MTQ5NTE3NzEzNTk5MzM2MzQ3ODg4MTA0ODU4NDkzNDE1Mjc2NDMwODM3MzUxODE4OTE4NzcyMDY3OTUwMTE5MTA0MDM5MDU1MTIxMjkxNTk2MjA0OTg1NzE5NzQ3MTg0NDgyMDMzMDc3NzkyNDkwNjAzNDIyMTIzNjAyMDU0OTg1ODg0OTk4MDkyOTU2NDgxMzE2MzgyMDU5NjA1NDU5NjE1MjIyNTIwOTg4NzM4ODUzMzUwIl0sIFsiaGFpcl9jb2xvciIsICIyOTc3MzM4MjEzODg1NjE2Njc3MDA2NzkyNTc5MTQ1MjI5MTg5NzM5NzAwMDcyMzExMzQ3MzU4NTQ1NTY5MzcxMDQ3NzQyODA5MDY2NjE5MjE2MjQyMTM4NDM5Mzc0NjQ5NDExNzcxNDE3NjE1Mjc2OTg3Nzk1MjkzNzAyOTU1MDUwMzUyNjg0MzEyMDM2MDA3OTIzOTg4Nzg5OTQzODgwMDg1NTE3NjU4MDk2Mzg0MjM3Njg1MzQ3OTQ2NjkyNjEyMTQ4MDgyMDA4MzczMTgwNTA2Mjk3MjcxODQwMjgxMzI2MzEzMjEzNjcxMzE1MTk3NTEyMzE1NzMwMjM3NDYxODQzMDI0NTIzOTUxNDg4OTU4NTczMTYxODYzMjgyMDMzNTQ5OTEwOTI4ODg0MjQ5ODIzMzY3MzM1MDk5ODY5NDAzODA1MzQwNDc4MzU4MzU1NzQwMDkyNzgyMjk3MTA0MDA2ODUxMzE2OTc1OTAxODcyNjQyMjAxODEwNzY1OTk1ODE1OTgxOTIyODE2NTA1NjY5ODgwODI2MTQyNDc0MTU3MzM4MjEzNzQyNzIzOTk0MDAyNjc4MjYyNTg3MDE5OTU3ODI3NTI4MjgxODIzNzE5NTc5NjIzOTk0MDkyMTg0MjgzMDA5OTIwODMzNzI5NTE3NTQ3ODM5MjgxNzEyOTUzNTcyODc3ODk1NDE1OTY1MDI1MzE1MTc3OTE5MjY2Mzg1NTk4MDczMjE2NzEzMjM2MTQ2MDgzNTYxNTAwNDM0NzQ3MjAyMTY2NzYyMTQ3MjUzMTEwNTczMzIxMTA4MDgzNTYyNzM0NzA1MzM0MTk3MDAyODA0MDE4MjQ1OTEwODExMTgxNjY5Mzc5NTY2NDAwMDk0NTQ2OTUwMjU5MDQ5NDY2MTY3ODY1OTI3Mzk0MzEyNSJdLCBbIndlaWdodCIsICIyMTI1ODU1OTgyNDY4MTAzMjA3MTc2MTQ2NjkzMTkzODY2MDkyMjYwNzk2MTAzNTE4NjI0NDcxNDE1MjE3MTkwMTU2NTE1MjQyMDM5NjU3MTcyMDE2MTc3NTI3NTAzNzY2NTYwOTUyNTc2NjQ3NzI3ODczNTUxMTEyNDEzMDA2ODY5NTg4MjgzOTE0ODYzMzUxMTE2NTE3MjU0OTk1NzIxNDg1NDk2MDcxNjA5NTUwMjQ3Njk5MDk1MTI5MTEzNTkyOTU3MDc1NzcwMzk3Njk1MDY2OTk5ODAyNTI1ODYyNDA0MTc4ODA4MDMyNDIyNDg4NzY3MjM1MTA0MTQ1NjkyNzYyMjM3NzI4Nzg5MTk2OTg3MjMwNjI1ODUyNjkyMzIzNzMyMzM5MjMyOTAwNTk2NDcxMzQ4NDIzMTQxOTgyODY3NDAyMzA0ODk5OTI4NzE2Mzg3Mzc2MzgxMDUwMTUyMDM1MDAzNTI5NDc2NTA0NzY1OTQwMzgwNDAyNjEzNDY5MjY4OTc2Nzc4OTkxODQ1MjMxMjE5ODMyODAxMzg2NzUwNTE5NTg4ODk3MDc3NTY1MTgwOTEzMjAzNjc0MjkzNDE5OTY2Mzk0MTczNzM0OTA3MzMwOTc1NzYyNDIxNjQzNjQwOTU5NTA3MTIzMTAwNjk5MTk3ODgzMTgwMDYxOTc1OTM1NTE2MDA4NDEyNzMxNjU4NDQzNjU4ODI1Nzg4NDczODIxMzc1NjU5NzMzNDI5MTIxNzc1MDI2NDI2ODEzNTUzMzIyMjkyNjgxOTM5NzgxODg1NDQzNDUwOTg1NDk0MDE4OTA2MTk2NDU3MTc2MzQ1Njc5MTUxNjE5OTI1ODYzOTI0MjEyMDA1Mjg5MjI0NTEwNjE1MDQ0NTkwMTE2NzgyODEzMTU2MjEwMzU2Mzg0MSJdLCBbImhlaWdodCIsICIzMDQ0MzgyOTM4MTg3Mjg2MTQxMjMwMzc0Mzc5MDMxODE0NjI5MTM5MTU5NDQ5NDIxNDgzNjA0NjI0NzQ1OTM3ODkzNDIxMjA0NDg5MTk2NjY1MDM2MTExOTkzOTExMzk2MDUyMzc5NDQ1MjA0OTUzMjYzNDYzNTg1NjI1NTU0OTc1NTg2MjYxODc0OTE4NDEzNzgxNTUzMDEzMzMzNTQwNDczMDA4MjU3NTYwNzA4MjYxODU0MTE0MjA4MDY1NTQ5MjcxODU2NzUwNDI4MDM0NTk1NjkyMzY3MTI5MTE4MDc4NDA2MDA2NDcxNzEyNjU4NjE4MDUxNTY1NzEyOTg0MTE5OTEwNzkyNDE2OTA5Mzc5MzM1MzE3OTM2MTY0NDIxOTc2NTU1NDgxNTk2NTI4MDY4MzAwODU4MDk5Nzg4NzUyMDY5MzgyMTI4NDIzODU4MzI4ODUyMDcxMzUxMTI3NDAzNjY3MjEyNDI4MzE0MDY2NjAxMzQ0MDcxODU1NjI5OTU0MTEyMDUzOTAyNjAyODU2MDg4Mjg3MTA0ODI3NzUxMzgxODk4NDA5OTY3MzcyNjEwMjQzNDk4Mzc0OTE0MjkyMzYwNTMyMTM0OTcwMjM1NTE4ODAyNjY2NjY0MTE2NjE3MzA0MTc2MDMxMDYzMjQ5OTg3MTExMDk0MDk3NzMzMTE3NjMyNjIwNjk1NzU3MDc0NjgyMTQ5MTMyMjUxNDk0OTcwNTIyOTMzMDgzMjM1NjMyNTkxNDA2Mzk5NjE4MzAzNzkwODk1NDYzMDczNzMyMjQwNDcyNTEyNzQ1OTc1MjM4MTI4Nzc2NjU2Mjg1NDY0MzQwNTI1MzkxNzE0Njg0NjI3MjY2MTUyODcxODA3MDMzNDcwNjU2MTQ0OTY5NzcwMTQ4MDQ1Njg0NzQ2ODA0MiJdLCBbImFnZV9vdmVyXzE4IiwgIjIyMTEwODc3NDcyMzk2MzMxMjM0MzkwOTU4NTQ2NTY3MzIzNTA5MzIzMzQ5NzE3MDEzOTA4MTgxNDI5NDA5MzEzNTY4MDcyMDI0NDc5NzY4MTUxMTgxNTgyODEzMTYwNzAxMDgxOTQwNTU4NDQxOTIwNDEyNDgwNTc4NDMzNDM5OTY5NTEzNjE0Mzg0NTU1MjE1NTA4MDIwNTcwNjA0NzY1NDkwMTU5MTgyMzg0NDIyOTc5MTA3NjMwNjM0NDQ2NTA0MTAwMDE3NTc2OTI1MzkxNDM4MjMzMzY4Nzk1MDE5MzIyNTA3MzM2Mzc2NjY5Njc3MjM4MDMxNzM4MDMxMzU5OTU0MTc3NTczMTYxNDQ0MzQ5MzYyNzc0MTk5MzM3NzYzMDE0MTI3NTkxMjgyNjI5NTM3NzIzOTY1NTQyNTQ3MDY5NTA1NTI0MjE5OTczOTk0OTg1OTg5ODgzMzg1MzcxNzM0MDg1MTk3NjcyMzA1ODk5ODUzOTUwNTg3NDM5OTEzNjk1MzUyMjMxNzI4MzM1MjA1ODA2ODY5MTg5ODU0ODQwNTQzNzIxMTcyMDU1MzIwOTUxOTExODQxOTYwODAyOTgwMjkyNzMzNjY1MTI0MDg5NjkzNDI5MDI5MTc1MDY4NjI5MDg3OTIyMzkxNDczNTEwMDUwNDY4Mzg5NjE0NzQ3OTk3NzM1ODE0MzU1OTYxOTI2MDgzODU4MDExNzk2NDAyMzA2NzE5OTA4NjA5MDk5Mjk0ODIyNjUyODg4MDA3MzcxMjUzMTUyMzUxOTI0ODIyNDE3ODE4ODEyNjMwMDQ2MTMwNjY4ODYwMTIzMzY0ODAwNzE5NDg1MTY5Mjg5Mzc1ODMyOTk4NTU0Mzc3ODI0MzY3MjEwMjU5MzcyNTc4MzU5NTIzODI1MzEyOTgzNzc5Il0sIFsiaXNzdWluZ19jb3VudHJ5IiwgIjExMDMxMDE1MzY4MTY4ODY0NTQ2Mjg4MDkzMTcxOTE3MDc0MzE5NTg5MDM0NTI3MjY4NTU3ODU0ODM5ODQ2MzgxNzQ3OTU4MzE1ODcwNzEwNjQwOTgxNjgxNzAxODk0OTc5MjMwMzI2MDIzMTI0NjA3MDI5NTAwOTU1MzA5ODA3MDUyMTczMDk3OTkwNjY3MDU2NjgxNTE5MjU0NDc2NDQ0NjUyNjExNjIzODA2MjE5OTQ4MzI3MjczNzQyNjg1MjQ3NzQ3NjA2OTMyNzQ0NTk5MDU2ODc0NjUzNDA2NzY5ODg2MzkxMzQ5MDI1NjI2OTk0NDkzMzUzMTcyODE1NDMyMTg1OTYxNTMzMzk2MDE0Mjc0NDY4ODcwMDk2Mzc4MjIwNzEyODc3MTEwNzU2MjUxNTgxNjgyMDc5MzM4NTA3ODg0NzQ1OTI2MTM2MzU5MDYxMTAzMTQ2OTU4NzU4NDc4MTU2MzM3ODg2NzU2MjgwNDczMDAyNDQ1ODc4MTk2NDE2MTA2MTA0MDI1MTYwMjAxODY3MzM0MDQzMTUwMTYxNTAxMDEyNDMyNjEzMzUwMTE3NDUzODg1MTE1MzEwNzUxMzg4NzA5Njc4Mjg0MjQ3ODkzNTE4ODY5ODAzODM1NTU4NjQ1MDk1MTIyMjgyMDg2MTQ2ODI0NjE0ODMxNzYzMTg0ODY5NzY4ODI5NjYzMDAyNDAyMjIxMzY1MjExNjY3Nzg0NDYzMTAwNTc5OTI5NzY3MDc3MTcxMTg5Mzk5OTEzNjcwNDg2MTU1NzE3MzEzMjk4NzczNTc5NTA0MTYxOTM3NDYzMTI3NjI3MTAwMjgyMjc1NzEwOTUzODEzMzU3NzAyOTI4MTg5NTU3NDQ3MzU1MzUxOTg4NTMwMTc4MzQ4NDg0NDc3NDc3OTM2MzUwMTgiXSwgWyJkcml2aW5nX3ByaXZpbGVnZXMiLCAiMTU0NzcxODAzNDMwNTQzMjIwNDA1NjMyNDgyNDI3OTg5NjQ5NjczMTk4NjM2MTIxMTg0NTczMzAzMTA0NzE4NDIzNzc4MDUwMTY2ODYxMTExMzM5MDg4NjQ1MTM5NTU0ODIyOTE5NzQ1MjgwMDA3MTI0MjU5ODk5MDYxMzY3NDY1ODMxMjYxMzgwNjIzODI0NDYyNjA3OTczMTM0NTIxOTIyNzcyOTkxOTMwMDgyNzQ3MDg1NzcyNzU0ODkwNzI2MzU2NTk3Nzc1ODYwNzIzNjk1OTg2MzU3MjYyNzQ3NjI4MTc0NTIxNjMwNTY2NDc5ODQxODM3NDUyNTk1MTI2MjQ1NDc0NDQ5MzQxMzMyMjA4MjIyNjU3Mjg2MDcxNzg1NDA0MDk2MTQ3MTIzOTExMDYxMzkyNjc5NDQ1MTc1OTA1OTk5ODkzODY5NDA4MzYwNDY3NDQ5Mzg4MjM0OTA3NDI2NzUwNjQ1OTQzOTUwODQ4MDEwMTM5ODA4NDcxMjEyNjg0OTY2Nzg0NTE0MDg1MjgwMTU0MTc5NzE0NTA4NzY3ODc3NDg0NTQ3ODU0MjQ1MDkwMjQ4MzY4NTA1MTI1ODI5MjM0MjA5NjcwNDM4OTAwMzYzNDA4Nzk3MDM0Nzc1MDU3NjAzMDM1OTAwNDQ2OTQwMTg5OTEzMjQ1MDQ2NjE0OTAwNjQ0NTQ3MDg1OTA4NDY0MjY1ODczMTY2OTUwNTQ5MTYyMTI5OTYwMTc5NDk0MDE2Njk0OTk0Mzg2MjY5OTI0MTAzNjQ2NjcxNjcyNzk5OTQxNjM0MzAyOTY3OTA3OTQzMzc0MDU0NzYwNTc3MTI5OTg3ODE5NjE0Mjk4NTA1MzExMTg0MjU3OTIzOTY4NjkzODY1OTgzNTgxNDQ5NDQxNTE4ODY2NDE0MDc0ODAyNjIiXSwgWyJzaWduYXR1cmVfdXN1YWxfbWFyayIsICIyNzU4ODQyNjc0NDQ0OTQxNjY1MTU0ODE5ODY2NjI1Nzc3ODEyNjM5NjUwMjg2MTI3MjAwMjk2NTc5MTU3MzE4MDU5OTg4MjExNTA2NTQ1NDQxMDM2NTI4MTQxNjkyNzY0NzIwNDQ2MjkxODAyNTk3NTY4ODAwMjg1NDg3NzkzNzUzMzk1OTQwMDc5Nzk3MDc5Mjc2MjAyNjQyNjY1ODM2MzM5MTg5MzM0MTUwMDg1NDYyNzMxNzk4MzYyNjEzNjY5NDg1MzU1MzYyOTA4OTIxMjg1MDAwOTIwMjQxMDI2MDA2NDcxNzAyMjQ2ODI3NjIzNjQ5MjMxMDkxMzc1NzU3NzM4ODA1MDQzNzcxNTAxMDA1OTUxOTcwODE4NTE0NzY0MjQyMTIzOTQwNjE3NzI0NjEzNzU0ODY0NzE0NzkxMTAxNjEwODkzNTE2MjU2NDI5MjQzMjgwMDc1OTg4MzIzMjg0OTMwNDAyNTc0NjU0OTMxMDEzNTU4MjQyNTI0NzMxNjIxNjIzNzk3NjUwMDIwMTQ0ODEwMTU0MDcxMTQyMTgxMzkyMjQwNTk4NTA5NDgzMjEwMTU5NzYzNjk5MjI4MTcxODg3MzQ3NTIzNjIyMjk5MjAwMzg4MDA0ODQ0ODQzOTQxMTM2MjIwMTI5NDkxODI1MTgxMzY1NDA4NTg5ODc4MjkzODQ3NDcyMjg3Nzk5NDk2NTY1Mzk2NjUwMTkzMjkyNjc4OTI4MTIxMzg2NTc1OTM0NTU1MDc1MTI2NDk2NzA4MDU4MTYyNjIyMDE1MTc5NjM3MzU2MTQ1NzYyNjc4OTgzNTg3MDg5NTgzNjM1MjQyMjY5ODYwNjQ1Njk0OTQ0NTgyNzg1OTYxMzkyODgyNDkyMjIxMDkyNDIwNzQ2NTU1NjIwMDI2MTExNTcwOSJdLCBbImFnZV9iaXJ0aF95ZWFyIiwgIjIxNDE1MzUzMjgwODk3OTAwMjI0OTk5NTY0NDQzODYwMTQyNDEzNTcxNjQxNzIwNTc2MDQ2NTk5OTI1NjU1MTQzMTkyODgwNzEwOTM5NzYzMTYwOTU2NjAwNTg0MzcwNDI2MzU4MTcyNTgwOTUyOTc4NzkwMjU1NzcwODM2MDEwNjI1NjA2ODc4OTg4NTI4NjQzMjU3NzkxNjY3NDQwNTU2MTkwNTkzMzE2MjkyMzU5NjA1MTcwNDg3MzQzMTY2MTU3NDgxMTM2NjMwMjE5MTU0MjI3NzU2Nzk2ODA3NDcxODEyNDI1NDg0MDcyNTAzODA5NjE4NjY0MjQ5OTk0OTE4ODE1MTE0MjU1NzUzMDIyMjM0MTgwMTE1NjA5MTQyOTgwMTI4MjE2MjY1ODAxMzkwNzgwOTcwMDk0MDMxNDA0ODc0Njc0Njk0NzA0ODM2MzUzNTA5NTcyNTg5OTM3NDE3MDA3OTU1MjExNzM2NzY5ODAyMzc4NzUwODkzMjk3MTc1MDE4MzQ5NzA4NzMwMjM5MzA1MTM1OTIzODkzNzc3NzQwMjYwMzUyMTA3NjIyMDgyMDg1NDA4MjgxNDIzNjYyNDU2NTY4OTQ2MzYxMTc0OTE2NDc1MTYwNzAzMTM1MDE5NTE5NjAxNzU4MDQwNzI0NTA1MzkzNTMxNTAyMjcxNTgwODE1Njg0ODE2NDM2ODk3MDU5MTY0NDUzNTEzMDE2ODEzNjU0MDg3MTMxMDgzMDMzODA4MTkwNzY5MTg2Nzc1Njk0MTQ0OTI3ODEzNDExNTczOTgzNjQ4ODg4MDI4NTQ1MzAxMjI0NjM1OTkzMjg2NzIwMDk0MTMzNTMwMzkwNzQ1MDI0MDE4OTkyODE3Mjk5MzYwMjY0MjAzNjUwMTQzMTA1MTA1NTg1ODY4NDc2MDUxIl0sIFsidW5fZGlzdGluZ3Vpc2hpbmdfc2lnbiIsICI4NjU0MjgxMDkyOTI0NzQ1ODg2OTI3MTkwNTY4NTQxMjI1OTcwOTExNDUzMDE5Njg4NDE2NjE2MDU4MTc3NzEzMDc5ODcwMzcwMzQ5NTU0NzA4NjA3MjUzNzc0OTMwMzYyOTIxMjUyNDgyMjEyMjQ3MDkyNjQyNzQxMzc1ODYwNDI2MDIzNzY0NzU2ODQzMjM2MDM1NjQ2ODAyMzcyMTU4ODQzMTc5MzY1NTkxODY4MTk5MjQzNTI5NTkxMTYzODY0MzM5NjM1NDg4MzM5ODg3NDA4Mjk5Mzg1MzkwMjg1ODc3Mjk3Nzg0ODU1MzYyNzU5MDk0MjIxNzYyNzcwNzUwNTg5NjM1NzUzNjExMTY0MTAzNzQzMDY5NjM4NzY4OTU2NzIwNzM0NzIxNDU3Nzk3MDIzODE4OTI3Nzk0ODI3MjI1MzU0MTEyMTA4OTQ2Njk2NjE4NTUyNjk2OTEyMjM0Mzk2NTUxNDUyODgyODg3OTM2NTIwOTQwMzM1MjQxODkwOTE5NzI1Mzc2MTQwNjgzMDg4NDEwMTU2MDAxNzY3Mzk2NzY5NjQ1MTUwNTM0MDYwODA5MzQwOTY1MjMxOTI3MDI5NjA3OTkzMDc0NjUxMzc3NTkyOTYyNDk3ODYxNzM4MzI2MDk4OTA0NTkxMTcwNjkwMTU0MzE3ODU0NTQ0MzQ2MDA5MzcwNjY1ODU0NzQxMTYxNjMyNzg4MTk0MDA5MTU4MzU4NDc3NjAxMDYxMzMzMjg0NzUyMDUwNjY0Nzk5MzkwNjg3NDQ0MzgwMjQwNzY4ODE5Mzc5MDUzODc5NTkwNDk4MTAwNDI4Njk1MjEyNTUyMzYwOTc2NzgyNTIzNzY2MTEzMzUxNTExNjMyODgyMTYwNjY5OTI3MjM3ODgyODc4NTM5Mzc2Mzc0ODYxNzE2Il0sIFsicG9ydHJhaXRfY2FwdHVyZV9kYXRlIiwgIjE0MzU4NTA3OTc4NDQ1MTU2NzIwNTY0NDc3ODA0NjU4MTM1MjQ0MzU3Mjk3NTAwNDM3OTI5NzU2ODQ4NjY4NzE2OTI0ODM0NDc2MTY4MjQ2NTc3Njg3NzU2MDg4OTU2OTc4ODAyOTMxNjQzNzc1MDg3MjIyNjQyOTIxNzkxMDQxMTg0NjA1NjQ5MjM4Mjc0MzY4NTU2NDQxNzA0MjE3MDg1MzQxNjQyODgwODk5MjA3MTg1MTY4NzMzMTUyNDMxMjE3ODAxNDMzOTMzODE3OTk2MDE3NDgxNTY3ODE2MjA4MjIyNDU3ODMzODA4MTIwMDgyOTgwNTk5NDg0NTk3MDc5ODI0Nzc2NzQwMzE5MzU0MjkxMjEwMjUzMTA2Mzk5MjI4MzU5NTM4OTY5Nzg0MjQyMjA2MzgyNDk0NzI0Njc2MTc4NjM0NjIxODYyNDAzOTY5NTgzMDI0Mjg5ODQzMzgyODUyMDYyODM2NDU4ODcwMDY3Mzc2MjQ3Mzg0MzU3Njk5Mjg3ODYzODU2OTA0NDI5NTQxMDgzODM0MTExOTQ3ODA0OTM0NDI5NTQ1ODc3ODAyNzk2MjEyNTQxOTUxNDQ5NDg0NDc0Mzc3MTE0ODE1NjkwOTU5MDcwMjM5NzY3MzM5NzY5MzcxNjA3MTMxOTk4MjE0ODMzOTI5OTMwMTMxODY3MTYxMzU4MTc0NTI5OTk4MTY0MTkxMDQ4NTU2NDkxMzQ5MTYxMDMzMjczODM5NTA4NjUwOTYxMzM3NzA4MTIzNjcxNjkwMDgyNzE3ODE1MzkzMjIyOTc1MTU5NDkyNjMyMjgwNjIwODU1NTA5NDIyOTQ1MDIzODA5ODI4MzM3NzM5NDA1MDY1NDIxOTgxMjkwOTA3Nzg1OTI0NzYxNzIzMzI3MzE1NDA4OTE2NDI0ODY4Il0sIFsiYWdlX2luX3llYXJzIiwgIjI1NTI0MTkwMjQ2MjQ4OTM4MDA4MDQzMzYzMTk3MzQxOTgxNTM3MzYyNjM2MjIzMjExNTIwMDg3MjIwMDE1MTgzMDUwMDgwNzQ2MDQ5Nzc2OTkwODUzNDIxNTM3MDA0MTk4NDE1ODgyMjkxMDYxMDI3NDA0ODA4NjQ5ODQ0Mzk4MDY2NTUxNjA5NzMwNzQxNDI1MzM5NDU4ODAzMjk2NDM4NDU3NzE2ODEyODY5NDY5MDcyODcyNjI3NTIyMDkxMDc5NDkwMjc1OTkyMjAzMTA3NTIzNzM5MTE0NjE2NjU2ODgwODQ1MzE2MjQ4ODA5NDE0OTMyNDU3OTI3MTU2Nzc0MjEzMjY0ODY2ODkzNjQwMDM5NjMyNDQ4NzY5ODA5ODE0MTE3NTc2MDU4OTc0MTAzMjE3NjAzMzQ0MDkyMjQzODgyNjE4ODEwNjY1OTk3NTQ0MzAxNTQ5NTY5NTE5MDcxMTkyMDc1MzE2MzY1NTQ4NzEyODMwMTI5Nzg1Nzg1OTU2NDg5MjAxMzYzOTczOTM2MDg0NjU0ODU5OTM3NDE3NjI5NTkyODE1Mzk3MTkyMzgyMzAwMTQwODAwNDM5ODg1NTY4NzQyMDUyODkyMDA1NTEyNTg5OTAxNzQ2OTcxNjg0OTA5MDA5NjUzNDMxMTA0NDM2NzU0NjU0NDcxOTI1MDM2MDU5NDUzODcwMzIxOTcxMzk4MzA4ODU4MDM3NDgzMjg0OTE2Njc4ODU1NjczNDQ4ODUyNjIyMTg1NDQzMjY0MDk4MzQ5ODY0ODAyMTUxNDQyMTI5OTAyMzMwMTY1NTE5Mzc5ODIyNzY0MTUzMTQyNDA5MTE4MzYzMDAwNDEwOTMzOTMwNzQ3OTAyNDE1NzExNTA5NzQ5MzExNDA4MTA5MTEzNDIzNTcyNTU1OTEyNDkyIl0sIFsibmFtZV9uYXRpb25hbF9jaGFyYWN0ZXIiLCAiOTI3Mjc1NzQ0MjE1MjIxODI1ODEwNjI4Njk0MjQ1MzU2MDk2MzU1Mjc3NzEyNjIxNTI1OTQ0MjE1MjcyMTc3ODc0MDU2NjY4ODI0MjgwMjg1MjEzNzY0NDc1ODM2MTU1MTcwNDg3NTkyMDIwMDcyNTc0MTQwMjAwNDQ4NTkxODcwNTk1MzAyNzIxMzg0NDEzODczNzY2MTkyNTAwMTY0ODk4MjY0MzQ5NTEwMDgzMjM0NjIwNTE3OTUxNDA2OTI5NjkzMjU3OTQ0MTU3NzE2OTQyMjA2NTExMzAxNzU4NTQwODM2MTI0NTg2MzQ5NDI1NDg2Mzg4Mzg3MDczMTk3MjE2NTU5ODgwMzM0MTYxNTMwNjE0MTIzNDQxMzc3NDQ3OTEzMTcxNjg4MTAzNDM4Nzg3NjY5MDYyMTQ3NTM3NzU1NzY1MDIyMzE4NTgyNDM1NDAyMTMyODcxMDU4NjczNzQyOTkyMTY3OTI3MTI2OTk5MjMwMzMyNzgxNjc4MTY1MjIwNTg3OTA4NjgyMTgwNjg3MzEyODI2NTY0MjYwOTIzOTQzOTY0MDg4MjAwNzk1ODAwNDI2NzcyNjUwNDI0NjQyMTk0NjY5MTgzOTY2NDE3NjE3Mzg2NTIzNjI0NDc5MzQ5Mjg3NDEyMTgzMzIyNDgzMTE1NzgyNTIxNjU5MzczOTY2NjQ4NjI0MTMwMDM2ODYyNDAxNjQwNTU2NDYxMDc3NjUxNDUwMzU3NTM4NzM3MzgyODgxMzEwMjA3MzI2NzY0MDkzOTI2NjAyNTQyODUxNDU5OTY3NDAzNzg0ODIzMjAwODc3MDg5ODk1NDA5MDQ5NTY0NTUyMjYzMjc1MDI1NDYzNDYyMTMzMjc1OTUxMjM0NTQwMjM2OTI1NzEyMDcwMTU2ODE2NDczMDc5ODg4MiJdLCBbInJlc2lkZW50X3N0YXRlIiwgIjE5NTc0NDkzOTkwODIxMTI5MTMyODAyNzk5MzE4MTk0NDI2NTQ5MzY0MDYwOTI4MTUwNDk2MDgwNjc1OTMyNjI5ODYwMDg0MzY3MDgwOTcyNjIzNDQ4NDI4NDI4MjQ4NzI3MDIzMTg2NDI1ODk0NDA2NTEzNDg2MjAxNjQ0NTM2NDMyMDY0Mzg1MTIxOTA1MzgwOTkxODY1MDAxNzA5MjM3MTM0NTgxNjgwNDA4ODI0NTc0NTQ1Njg1ODUwMjc4NDkxNjk2MDQzMTIyNDA2ODk4OTI2MTEyNDc2ODE4MTIzNjI0MTEwMzA4MTE3OTM1MDE0NjI1NDEzMDExMjMyMTExNjYyNDM3NTE2NjUzMjU0MjAxNzQ1NDc1NjEwMTA2NjMzMzM4NDc5Nzk1OTQyMDQ2NjYyNTYzNzIzMjk4OTcyNDUwNjYwOTAwODA3ODE3NTI0NDkwMTA3OTQyODEwNzgxMTcwNzEwMzY5MDYwNTEyOTU4NjE1OTEwOTg3MjYzNzQxOTc4MzU4MjY1MjkwODEwNDc0ODgzMDY5Mjc5MDM4NTA4MDY1NDQzODI4ODQzODY5MjYwOTU4MTgzODgwNzU4NDc0MDI0NzI3MTk1NjQyMDI0NzM5NTI1Njk1MTQ3MzYyMDI0MzQ1MDk2MDIxODczNTkzMjUxMTQzMzUxOTk2OTM2NTUzMDcwNjgxMDYxNzk5NDQyMTkyMzIxNDU2ODkwNzk2NDA5NTc2ODkzMTk0NjE0MTg1MjcwODQ1MjUzODcxNDUzODAyODk4MTk0NDI2MTY3NzgwMDUwOTEzMjgxOTM1NTQwOTgzMTA2NDIyMTE1NzM4MTAzODg0NDQ2NDE4NzE1MTg4MTA5NDY4NTkyMDkyODAyNzUyNDY1ODUxOTA3NTMzOTg5NjgyODM3MzY3NzgiXSwgWyJkb2N1bWVudF9udW1iZXIiLCAiMTAwODc2ODIxMzgzOTcyOTQ0MjEyODI5MTMyMTAzNTMxMzU3MzE5MzU2NjY3NzEzMzUwODg4NDc1MDg1NTM2NTA3NDQxNzYwMTI2MzU1NjE5MDgxNzY3NjEwOTkxNDAyOTUxMjQxMjY3NTgxNjk0MzkwNzIyNjAzMTE3OTQ2NTQ4OTMxNTgwNDE4NTQ3Njg0MTg5MjI5NTIwNzE3MTQwNDI4ODE0ODc2MzA1MjU3ODg5OTU1OTM5OTk1MTg2NTk3Mjk5MjkxNjIwNjk0Mzc5NDU2NTA4OTA4MDE4MDIxMTQxMTg3NzU4NzY5NDc1NjI3MzU2OTYzOTE4MTY0MzI4NDAxNzg0MTgyNTU5NTU4NTcxMDAwNzIzOTExNDA1MTk0NTgxNzgwOTc5NjA5NDc5NDM0NDc5ODkzMzU3NDAzODQ3NTUyODQxMjk5ODA3OTg1MTQyMzgzODg3Mjk5MzU1Njg4ODY3Mzk0ODA0ODQwMDI2MzI5NDI5MzU1MzM4NjIzNzIzNTA5Mzg3NTgxMjMwNjAzMzczMDg0ODU3MTIyNzY2NDc4NzQwOTA1NzE2NDk1MzgxMDgwNDMwMTA1MTI2NjY5Njc4NTk4MTkzODg1ODYxODIxODQ4Nzc4NjQ4NjYyNjMyMzAwNTgwODk3OTQ2NDczMTA3MDEyNDAyMDI2MjA5MDI2OTI5MjUyMDk1Mjc3OTI4NzcxMzA5OTY2ODgzMTM5NzgwMDc4ODY4MDgyNDU4NDU0MTU1MTM3NDcyMDY5Mzg3MjExNzkwNDQ3MjcxODQ4NTQ1MDU4MDM2MjcxNDE0MDc4MjQ5NDMyOTkzNDA2MjU5NzY0OTQ1MjU1Njc0MDg4MzA0MTYyNzU4MTczMTI5MzY1ODUzNTA2NzYzMzIzMzI4MzQ2Njk3MDA1NDI3MjU2NjYiXSwgWyJyZXNpZGVudF9hZGRyZXNzIiwgIjMwNzc3NzM2NDE0OTgyNTIxMTgwMTQwMDQwMTMzNDU4MTY3NDM3NDUwOTYwMzM1ODUwNTgyNDgwMzIzODc5NzczMjY5MjQ0ODAzMjU0MTA2ODUxMTg4ODk2NjEwMjM2ODA5MDk2NzM1NjczMzc1NTcxNzgyOTc3NDU2NzM0NzI4NjY2NDc3OTE2MTgxNzQ0NDU4MjE1MTM3NTk1ODIwNzI2NTUyNTM1NDQyODk5ODc5MjUzODgzOTk3OTMzNDkyMzg5Mzc4NTExOTgzNDkwNjAyMzM2MjA5NTA1NjQzNjY1MDI5NTI3MzczMjc1NTAxNDA4MzQxMDU1MDcyMzc3MzkyNDQyNzc1NDYwNDQ3MzMwNDIwMTgwMzA1MzE0Mzg4NjQ2MTI1MzQ4MDM1NDQ4MzM2MzM2ODI5MDMzNjM2NzA1MzIwMjExOTYxNzU3MTg0MjMzMDc1NjY5OTg4MzUyMTM4NDEyNzM4NjgzMTg5NTM3MzQ2MzY1NTIzNDc0ODY1ODM2OTA1MTg2ODEyNTI0NTAwNTA2NjMwMjI2NzE0OTM2MTc3MzI3NzY2Nzg5ODk4NzQwMjAyNjg4NjkzMjk2MzM0NTIzMjQ1NjYzMjU5NDk5NzY2MDk3OTM3NTI0NjY2NDExMTAyODEwODI0NjgxMDQ1NDE1MzA2MDcxMzAwNTM3NjAzMzM5NjU2MjEwMTk2MTQwNzEzMjI4NDAyNjQxODM3OTk1OTUwODY4Mjc4MDMwOTM3NTQxMDUzNDQ5NTI1ODkzMjA4MzQ0ODQyNjU0ODU4Nzg5MTI3NTc0OTUzNjU3NzQ2MjMyMjgyMjI2MjExOTk4NDU3OTY5MjE1MzY5MzM2Nzk5MjI4MDIxODA3NDk3NTkxMjE1NDYxMTMyNTUxNzk3MTczNzg2NTA2OTU1NDIyMjQyIl0sIFsiaXNzdWluZ19hdXRob3JpdHkiLCAiMTUxNzMxOTMzOTQ0NjExNDQ5OTM3MDQ4NDY4Njk5NDUwMTczMDg4ODg1NDk2NTkzNzgwMDc5NDU5MDg4ODI4MDk3NTAzMDgzNjczOTY2MzEyNTg4OTg1MjkwMTYxMDYzMTkwODI0Njc3MjY0MTA3MzU2MTE5NzI5ODU0NTUwOTkzNjgwMTIxNTQ0MTQ2OTU5ODYzNjUzMTAwNzkwNTM4OTI5NjYyMzQ0ODA0NDYyMjc2MjQ4OTcxNDYyNDA2MjYzNzY5MzMyNTU1NzA1MTU1MzI3NzMxNDA3Mzk1NzgxOTU2MDE2MTUzOTE2NzEyNDk3MDY5ODkzNjI0MDQ2NDgzNzAyNzk4MTU0NTQxMTcwOTY2MzEwODY5Mjk2MDQ2MTM4NjI0NDI0ODk4MDczOTE4OTMzNDk2NDQzOTQxNDE0NTY4NzUzMjAwNDQ5MTQ0MjQ2Mzc3NzQ1NTAxMDM1OTY3NDQ4NzE1NDAwNzYxOTkyNjc1NTkyMjM3NDQxMDA1ODAwMjQ1MDMyMDYyNTAxNTEwNDExMjg4NzA3ODIxMzE5Mzg4MzkzNTgzODkyNjg5NDU2NTg4Njc5MDAzMzA4ODY4MzYyMDU1ODg5MjQyNjExNDkxODQ3MzY4NDU2NjUyMDQ5MDUzNTY4NzA5NTg0NzQxMDMzNTUyMDQ1Njg2Njc0NzU5MjY5OTE5NjE2NTgyOTM4MjQ5NjYyNzQ1MjM4NjM2Mzc3NDQ2NDAxMzk2MjU5NTEyOTgwMTg4NTg4NTYzMzAxMjk2NjQ3MzY1NzEyODg3ODU1NDE5OTM0MjgzNzk4OTUzNDU0OTUzMDk2NjUxNjk0MDU4MDExMTg2MDU1NTY0NzQ1NDI1Mjk1MzAzODE5OTE1NDI2ODk2NjM4NzM4MjYzNDAwNTY0NzA2NDY5OTI5OTk2MjQiXSwgWyJleWVfY29sb3IiLCAiMTM3Nzg1NDA1NTkyMTY5ODA5NDQzMTM3OTk4NjQ0ODI2OTQyMDc1Nzc1MzI3MDQwNTIzMjA1MzczMDk4MjI4MzMxNDM1NDEyMTY2NTc1Mjg0NTcxOTI2NDE0MTQzNzM5MTYwMzg3MTQwNjUwMTQ4MDg0NzY2NDE1OTUyNjU3OTgwODI4MjQxMTMxMjE3NDMzMDMxMjE5MTg2OTI2OTUxMzc3NTE2ODMzNTMzNzgxMTE3ODk2ODU3MzY0OTIwNzc5NjQ5MzgwNTgzOTQ4MTg1MzcxODQyOTA4ODQxNTIxNTAxMDUzMzE5NTk0MTI1Nzg4OTM0MDc2NDI4OTU1NzEzMDMyMzE5OTQ1NjgwOTM4NjE4NDA3MTI1NjYwOTkwNTQ2OTcxNzM4NTUzMDM1MjQwNzM3NzQ5NzEwODI0OTcyOTkwMjM2MDU3ODE1NDM3MjIxMDUxNjg2Mzk2NDk1OTQxOTExMzg3MTg3NDMyMzkwODI0MjUzNjQwNDE0MjAzNjQ1MDM4Mzk0NDk3MTA1MTI5OTY1MTg2MTQyNTg3ODc4OTU0NTM4NjI2NTkwODQ2MDk0NDgxNDUxNDA2MTE4NjA1MzU5NjM3MTkzNDM3NjAwNTk1NjEzNjYyNjUzNzgzNjcwMjk2MzA2ODE3NjM0MjkxMDgyMTM2Mjg3NjAwMDcwNjg5NDEyOTE1MDYzNzA3Mzc0NTkwNjYwNjU1NjQ2MTQ1OTI1ODY5NzE5OTE2MzY0Nzc2MjAxMjczODQ3NzM4MjE2NjQ3NDkwNDgyMDkyMzUzNDkyNjAyMDI1NDI0NDMyMTg2MzI5OTYxMjM2MjIxMzIxMzc3NTU4ODc4ODI5NzY3NTcxOTc4NDg1NTY5MzU2NDM4NzY2NzAwMzIxMTcyNDk3MDY0NjY4MzUzMjU0MzI1OTU4NDQiXSwgWyJuYXRpb25hbGl0eSIsICIxMjQ3NjMyNzEwMjk3MDY4NzA3MzQ2NTI4OTA2MDE5ODEzMzY2MDk1ODI0MTkzODA3MjQ2NTMwMjM3NjU4ODc4MTYyNDc4MTIzNDEyODAxMTE3MTAwNzk3NTY0Mjk0NzI4NjAyNjQwMjcyMDEzODA0NTk5NTQzMzE4NDU3MzQ1MDI2NTE3NzI0NzY1OTk0NDM3ODIxMDkxMDI4NTEyMTgwODI5NjI2OTUzODg1MjYwODQ2MTc0MTEzNDU5MTM4NjU0ODk0NDYxMDUyMDQ1NjQzMDY3NzAxMTUwMDc2MDAyOTAxMjU2MTQ5ODg0NDY4NzkwNjE3MTc3NjQ2NDk5NTk4NzkyMjgzMzIyNTUyODUxNDk0NzM5MjcyOTI5MzQxMzg5MjkwNDY0MDUxNjQ4NjAxNzE1MTg2NjQ4ODAyNDAwNDA0MjcwODU1NTgzODA2MzQyNjU1MzYyOTczMTk4MzA4NjA4MzM5OTI5NDc5OTk1NTc4MDczMzUyMzcyMzM0ODIxNzM3Mjc4MjQ0OTc2NTQwOTIxNjUyMDExNzE4OTIzNzQ4NDkwODI2ODExMTk0NTAyMzA5MzI2NDEzNjIwMjg5NTE4MTI5MjE5Njc4OTc2Nzk3NzYwNzc4NTIwMjQxNTc5Mjk1MzI2NDI1MTA4MjcxMTE0MTEzOTYwNzY3NjQwMzAwNTEzOTY4MjA2NzA0NzgxMzc3NDg0MjA5NDc5Njg4NzQxOTU2MDQ5NjExMjI0Nzc5NDM1ODY1NDg4Njg4OTc5NTQ2MjA4NDI1OTM0MTg4NjYzNDE1NjIyMDc1MjI0Njk4MzcxNjU0NTI4MDcxMDEzNDczNTk0NjQ2OTUyMTQwNTM2ODk3MTYzNzU0Njg5MjIwMjUyNjIzMDc0ODM4NTQ0NTQ1MjY1Mzc4NTQ2OTMzMTM3NyJdLCBbImdpdmVuX25hbWUiLCAiNDM1OTA0ODc5MzM0NzExOTYwNTI2NjU0NjUwOTQxMTE5MDQ2NjI4Mjc2ODIzNTI1Njc4NjMxNDA2MTIwMjI2OTUxNDMxNTA3NTY4NzMxNTA3MjY5Mjc5NTg2MzQwNTU2NDkzNzA4MTIwNjgzODAzMjcyNjg5MTU2MTUyNTM0MjM5MDc2MjMyMTU3MzU4NjYzNjc2MTU3MTUwMDUzNDY5NTU3NjcwMzIzMDE2MjA1MzQ4NTgyNzg2NzYwMjM2MTIzMzg2NTg3ODc3NDM4MzM3MzIxMzMxMjcxNTQyNjkxMjI2NTEwODU2Mjg2NjQzMjc2OTI1MDUwMzUwMTA2ODg4NDcxMTEzNDU5NTA3MzQ2MjkzNTk4MjkwMDU1Mzk3NDM0ODYzMzU2MzE1NTgwOTIwMDI5MzQwNzg5NzYwNzA3NTY0MDQ3ODk1NTQzNzc4Mzc0MDM5NzQ2NTIyOTgzODczNjM2MDQ1NzMxOTg2MzQ0MDE5NzM5MjYyNzEwMTQ5MTA2Mzc0Mjk3ODY5Mjk3NzYzNDYzODU3MTkxMzk2MTQxNjc4MzQ4MTgzNjUyMzc3NzQ5MDIzOTk4NTM4NTE1ODU3MzAzNzYxODM4NjYzNjAwNzQ1OTM3MTIxNDEyODU3MzcyODcwNzU3NTc1OTAxMjczODc0MjY5NjkwMDI2MzM3OTk0MTk2MDgxMzc3MTI2OTQ1NjcwOTg5OTQ0ODE5MzA5Nzg3NjY0NzUyNjkyMTAwMzAwNzczODExMzUwNDUwNTAyMTE3MTE4MjgwNjQ0ODY5OTk2MzUyMTQ3MjQ5OTMwNzgxNzAyMDg5MDQ3ODAzNzkxMjI3ODM1NDQxMzkxMTIwNTE0ODYwNjA5NjI4ODk5NjI1NTU2MTEyMDIwMDkzNzA5NjA5NjAzOTI4ODM1MDMzNjE1MCJdLCBbIm1hc3Rlcl9zZWNyZXQiLCAiMjMxMzY3MTczMDcyMzkwNTQ0MTY0MjI0NTU0NTE5MjUxODUwMjgzMTQ2MzY3MzQzNjYyMDQzMTUyOTI0NDM0Nzg0NzMxOTg2NjY5MTMwNzY3NzIzMDAxODY0ODM3NDg0ODUzNTg3OTQ0ODUxNzA4NjAwNTMzODYwMjQ4Mzk2OTkzMjk3NzMxNDkzNDgxNTg4MjkxMTk4Mjg5NDQ3ODk1MjUxMDU2MzQwNzE5NTUzODE0MzgwMjYzOTMwNjQ0ODY0NzE3MTI2Nzg3NTg1MzEzMzQ3ODQxMzQ0ODY3NjMzMjkyOTc4NTI2NTA2ODQ5MTc5NTQ2NzY1Mjc2NTg2NjE3MzM4MjczNDAwNDI0MjE3ODI2OTMzNzkzMzU5MzEyODUyNTc0NzQ3MTkxODk5Mzk1NDUzMzM2MjYzMTkzMDEzNzM1NzQ2NzMwMTc1Mjg1MzQyMTY3Mjk5NTYzOTg0NzA1OTY3NjIxNzMwMzQ2MTY4NDI5NTg1ODE0NjM5NjUyMDMwNzY3NDAxMDgyNzYzMjE5ODg1MTczODc1NzU1OTEwMzk3NDc4ODE4NjMzNzU1OTgxODMyODg2NzE2ODA2NjI3NzExODIyNDU3OTA1MjgzMjI3Mzc5MDA3NjQwNjgyMzI4MjI4NDUyNTQyNTQyNDMwOTE3MzYxMTM5NjI5MDY5NjkzMDI5MDY4MDQ1NzI5Nzc3MTUxNjcxMTMzNTMxMTQyNzQ2MzI2MzQzNTIxOTM3MzUzMzIwNDIzMTgyMDAwMDk0MzcxMjE4Mjk5MjMwNDExNzc5MDQ3MTA0MDQ3MTk5Njk4NjUyMzkzNzIxNzEyNTM2MTI1MTM2NTM4NTM1NzYwNjg5NjYyMTAwNTUzNTc3Nzc1NzgwNDE4NzA2MzExOTgwMjgxMjQxNTE1OTU0MjExMTYxMjkiXSwgWyJiaXJ0aF9wbGFjZSIsICIxNDI5MTY0NzA3MjMzODE5OTM1NDAxNzM1NTI4ODM1MDM3MTIwNzk5OTQ4ODg1MzkzODUxNzgzMzMxNzI4NzAyODE3NjQ3MzAxODEyNzQ1NjQ2ODQ4MjY4Nzg4MDI3Mzg5Mjk0NTYxNTI0NzA3OTE1MTE4MDQzNTg5NTc3MzkyNTY2NDUwNjE2MTU2NTUyNzgxNzUxNTg3NTA5ODY1NzExMTM4MTY1Mzg2MTMxMjU4MjUwMzc5MDYwNzMwMTY4OTc2NzEzNTkzNTMwMjE2NDU5NTUzMzk3MDgzMTIwMDgzMTI0OTMwMjU2NTM0OTEyMDYxNTg1NjQ2NzkyMzk0NTAxMDQ2Njc2ODMxNjA1MDM4NTkzNTQ2OTE1MzM2MTA1MjQ0NzQxODM4MjExNzA2NDUyNDM1MDU1MzExMTc3MDg2ODM4OTA2NDQ5Mjg4NjQyNDc1Mjg5MTM3NTMyMTU3NjE5MTcxOTczNDUxMzU0MjM3MzAyMTU4NTk4NzIxMzYyMTU2NTU5Njc3MzEyODA3MzAxNjEzMTU2NTg4OTMwMjg1NDQ0NDg2NjQzNDU2NDUyNjI1ODcyNjU1MDAzODUzNzQ4NjY4NDk1MTMzMDY3OTg3NDE2OTAyMDYyODE3MjEyNTc2MTcxNTE2NzM4ODI4MTIyMjAzMDkwMzMzODk2MzE4NDk2MjU5NDcxMTM5MTMxMjc1MDU5MTA2NzY0MjQ3NDUzMzQ5OTE5NjczMzE4MDA5MjUyNDkwNTcxNTMyNTEyNTU1ODc4NTk4NTU3Njc1NTcxNjEyNDU4MDM2MzU4Mzc1MzUzNzE0NjEzOTkwNjg0NjMwNTQyNjQ0NzI1OTg2NDQ2MjQwNjQwOTI5MjY2OTc5NTU5NDU0NTY4NDE3MjAyMzI3MzE3NTU3MzUxNjg2OTc0NjU0NSJdLCBbImFnZV9vdmVyXzI1IiwgIjE2ODk0MTAwNjU4NzM3NTY1OTk3Mzk5MTYyMDg3MTU5Mjc3NTM2NDk0NzIxNjQxMjQ1NjQ5MzI0MzU1NTQxMDEzMDA4MzUyMzExNTc5Nzg3NTY3Njg1ODU0MzU1OTY1NDE4NTk0MjEwMzA2NzQ5NDI1NTE0NTI4MTkwNDM1MTIzNDY2OTY4MDE0MDcxNjU5MzQzMTU0Nzg2OTk3MTc1MjYzODM3MjgzNzkzODE0NDkzMDk5NzczNDY4OTgyMzUxMjAzODY1ODgxMDYwOTk5MjYzNTMyNjI5NjE4OTc5NzU4MzA0NTEzNTI3NTI5NTQ2NTYzMTk1OTk2MzI4MDIyMjc4OTE2MzUxNDQ3NDM0NjE0NjM0NzI3MjQ2NzY1MDk0NTAwMjM2OTkwMTg2Mjk0NDU4MjAwMTM4MzgzODE2NTkzNzEzNTkwMDk4Njk1MTg1MzcyMzExOTAxNzE5NTY2NjI3MTAyMzEyMjI2MTk1NjcyMDU2MTgzMTgwNjkwOTg2NzY4NzEyMTY5MTA2NDk2NjU3ODY4MTgyMjk2NDUwOTUwNTQ0MTU0MjkwNzIxOTAwMTk0MzEzNDgwOTY5MDU3NzM0Njg0MTE3NzgyNTU1MDg5OTI0ODQ3NDA3MzI0OTYzNjY0ODQ3NjkxODc3MjUwMjM0NzgyODk4ODA1NTg5NTAyMTkxNzQ0NTMzNzc1Mzg0ODgwMDEzODA5OTA1NjE2NTk4MjgwNjIwOTA1Nzc5ODA1MzQ3MDg1NDAxOTUzMDY1OTgzNTY2MDAwMjY2ODAyNzU5MTU2ODA1MjUwMzUxOTM1NjAwNjE5NDU1Njg1OTQ5OTM2MzE3NTQ5ODk0NTIwNjAyMDgxNjM5MzY4Mjk3MzMyMjY5ODkxMTQ3MzM4Njc3MDY3NjEzMjQxNzQwMjM3MTIwNjIxIl0sIFsiZmFtaWx5X25hbWUiLCAiMjc1Nzk5MjcxMDg0NTM4NzE1NDUyNzc1MjMzMjU1Nzc4NTcxMzU5OTA2NDk2ODM3NTM3ODI2ODYwNDY0OTI3NTgzODM3MDMxMDIwNjQ3NDczMTUzMzUzMTQwODc5MTA2NjM1ODE1NzUzMjY1NzExNjM0OTk0OTc4NjYyMTg5MjgzNDYyOTE2NTU2ODIyNzQyOTk4MDI0NTc1NzgyNzE2MDAxNjMyNDcyNTgzNDc1ODE2MzMxMzA1NjM2MTM1NTcwOTAyMDI3MjUzNzYyNDQ4Mjk5MDA5MTgzNTg2NzE2MzU3MDE4OTYyMTYzMTA3NjM0MzQ2Mzk1NzE3MjY5Njk1ODIyNzI4MzA2OTQzNTcxNTkyNTA4NTc3MDg2MTAwMTQ5OTg5MTM1NzI1ODk0NjU2MTg3Nzk1MzcyMzYwODIxNDkzMzE3Njg2OTc4MDMyMTIyMTkzMDA3NjY4NTExMzQyNDk5NDYzMzI2MzM3OTY0MTEzODI3NDc2NDQzMDQ0ODc3OTM1ODk5NzQ1NTkwMzQwNTAxNjIzNTMzNjc0ODIzMTc4MTE5ODQ1NTAyMTY4NTQ5ODEzMjU1Mjk5NjYzNzkzMDc0OTM3NjMwMTgxNDE5Mzg0MTQ3NjU2MDgyNzgyODk4Mzk5OTMzNzExMTE5NjEwODE4MjI4NjY1OTY2NjY0NjIxMjg1MzgwMzUwODcxNTkzNjA4MDc1MjEzMzQyODE5MDU1NDg3MTQ1NjkxMTY3MjA2MDE3MTcxMTcxNDc3NjI0NzkyOTU2MjYwNjEyNjk4NTkwMTIyOTc2NzQ0NTI3Njc5MzE0MDUwNzg5OTEzMzM3ODI5MzUwNzk5MzcyMDE5Njg5NTYyMjcwMTE2OTMzOTQ5MzQ5NzcwNTA2ODI3MDc3NTc5ODA0MzQ1MjMwNDc4ODgwNDMiXSwgWyJyZXNpZGVudF9jaXR5IiwgIjIxOTc5NzczNjEwNDk0MzU1MDE5NzM4MzQzOTY5ODMyODIxODk3OTE0NjE4MDY2NzM0NzcwMzczNzMxNTcyNzI5MjQ4NjA2MjM0OTI5NDM2NzgzOTg4MjQ4NjYzMjc2NTcwNTc2NTAzNTE4NTU4NTY1NzEzMjE5Mzk3MDQ5MTYwMTIwNzU4MDk1NDY4MTMyMTIzNDUzMjIwNDU5NjM3MzQxODgyNzQ2NTA1MDAzMzYxMDU0Nzk0MDM5OTc1NDc3MjU0MDExNDE3Mzk1NTk5NDUxNTgyNzQxNDEzNzE2NTY2NzUwMDIxOTEyMTc1NjU1NzY3OTc0MjUxMDU1NDQwOTUyNTQ3NTEzMjAwMjQ0MTY1OTMwNjE3ODc0NjE2NzkwNjQ3Mjg2MDg1MjgyOTkwNTg0NTc4ODUwMDAyNTExMjk4MDUzOTM2OTgwMjU2NzU4NzU5ODY2ODA1MzQ2MzcwNTA3OTM2NTE5Mzk5MzY0MTYwOTU3NDgyNjYyNTYxMzE1MjM2ODQ4MjQyNDE0MTg0NjcwNjEyNjQ2MjAyODI3NzcxODE1MTg5NTEwMTk1NDU1MDIzMzgzMzI3MDE0NzkwNDMyMDk3ODc1ODc1NjY5ODA2NTM0ODUyNTgyOTc4ODI0NDMxMDY5OTk2ODczOTk3Nzk4NjIyMTg4MjIzMzkxMDUzODEzMDQzMTgxOTk0OTIyMTU0NjgxNzU2MzAzNTk5ODY2NzcxMDE2MjQzMjQ1NDg2MDQwMjcwODUwMTUzNjc1MzE2MTIxNzcwMTQ2MTk1NTk2OTM0OTUxMTc3NDQ4ODcwMzYwMjA0ODY1NzQ3OTU5MjE3Njk3MTYzMjY0NzQ2NDc0OTE4ODE5NDcwODgzMTI1NzU0OTE1MzE4MDI3NzIwNzMyMDU1MTExMTc5MzcwNjI5MDU2Il0sIFsiYWRtaW5pc3RyYXRpdmVfbnVtYmVyIiwgIjc3NTc3NTY0MjY2NDg5NzEzMzYyMjI4Njg3MDMyOTg3ODQxODExMzM5MTM2MTk1Mjg1MzEyNjE5MDAyMjY2NDY3NTU1NDMxMjU1MDc0NzEwNzk3MjMxNTQwMzQyODAzODU0MTEyODUyMjA3Mjg0OTIwOTU1Njc4MzUxODE4ODg5ODQxMjQ4OTExOTk4OTkzNjUxOTYyMjE2NTcwMTIzNTYzMjI1MDEwNjI0MTQ2NTk0MzIyNTM1MDM3NTYwNDYxMTQ4OTM1MDY4OTQyNzU4OTIwMzE2NDQ5NDc2ODI4Njg1NTk2MzQ0NjQyMTgwNzY3NTY5NTU4MTA2MzU1MTQyNTkzNDIxMzgwMTQxNDMyMjczMjk1NzE3NDU5Mjg3ODM3Mjk5NDg5NTYyNDc0MjkxNjkyMDAzMzAwNDkyODcyNTk3Nzk4NTg0MTEzOTU3MDc2MDM2OTU5NjcwMTA5ODM0NTAxNzkxNzUyMjg0OTc0MjU0NjIxMTI3MDI0MzU3NjU5MTA2MzgyNTUwNTU2NzM3NzE5MTI5MTA0ODQzOTk2NDE1MzIyMDg4MDgxMDAyNjAxMTk0MDY4MzI4MTI1MDIxNDMxMTYzODcwOTM5MTU3NDIyNDIyMTcyMDEwNTU4ODA0NDgwNzQ0MDcyNDM4Mzk3NDQxOTQzODIwOTUxMDE4OTc4MjExODkzODA5NDIwNDY5NTAwNDI1ODY2MzM2OTI0MDA5OTgxNTA5NDAwNjg2NDc5MjU0MjY5MjU5NTY1NTI5OTgwMDU5ODcyOTQ4MzM5NjQ3Mzg5MDg4MDg1NjYyMjQ4ODA4ODEyNjM5ODU4NDQ5MDIxNzgyMjM0NzIxMDQxMTAzNjU1Mzc1NTA3NzkxMjIzOTYwNzY4Mzc3NTk0NjM1MzA0ODE0MDUxMDU1MjU3OTg1NTUiXSwgWyJiaXJ0aF9kYXRlIiwgIjk0NDk1MTc2OTA4NzE2ODg0ODgzMTE0ODUyNzc1NTEwMjUxMDk0MDg5NzMwMjkzNTA0NDkzMzIzMTg1NTc5NDc1OTAxODYyMzYyNTI5MjYzNjk4NjkyNjc1NDU3NDgxMTk0MDc2NTcwNzA1NTUyNzg5MzM3MzI4MTgwOTY1MzgzNzA2NTAwMDI2MzIzOTA5NDAxMzgwNDQwMzQ4MDUzNzIxMDEyNDk1MjE2MDk4OTMzNTYzNDkwODM2MTM1MTUyODUxOTMwMDgzNTQwNTM4Nzg4MDMzNTg1NDIyNTAxOTgxODM2OTA4NjgyNzI2MjIwODkwNDUyNTYwMzk4MDc5NzIzMDIxODQ0MDg1MTg2NDk1MTY0NDExOTg1MDgwNDY2MjEwMDMxNjA5ODc4MDEyNDcyOTExNDk2NTM5ODkyOTA5MzY2NzQyNTM4NDU0MzIzOTgxMzc1NDk3NTI3MjQ5MTEwNDgzOTg5MzMyNTE3NjQzNTEwMzkyODc1ODI1NDkxODI1MDQwNTMwNDk4NjMyOTk0MjQ2MjQ3OTUwNzMwNTI4MzQ2MDcwMzYwNzQyOTgzNjQ2NTY1Njc2MzAxNTIyMjA4OTU2MzUzNTkwNjc0NzI4NjQ5MjEwOTU4NTA3NzcwMjMwNzc0MjMyODE5MjA1NTA4NTA2MDQ3NzI0NDk5NTI2ODkwMjY2NTI4MjQ4Nzk4MTQyMjY4Njk0MjU2NjkyNDY1ODg0MDMxNzIwOTQyMjM1NDk3MDA2MjM5MjI1MDQyMjk3MTg5MTQxOTM0MzkwNDI0NDEzMzEyMDU2MDkxNTM5NzkxMDczMTI0NTg4Njc0MDQ0NTY5NjI3ODg0MzMxMTIxMzgwNDAxMDEzOTQ2OTY0Nzk0NjQzNzQ4Mjc1OTU3NDkyMzU1OTc3MzA5NTU4MDkwNjYiXSwgWyJnZW5kZXIiLCAiMTk3OTUwNDU1MzYxOTk1MTgyMTc3Njc0MTc5Njc3NDY1ODU1OTY2NDA0NzM0NzU4NzQzMjAzNTA2MzI1NzUzNTQ0OTk4MzkwMDM3Njg3MTY3NzQ3MzUxODgzODY2MjM0MzU2NTAyNzgxNzMxODI5Nzk4NTMxNDY4ODcwMTI2NjU5NTU0NTcxNDAzNzg5ODY4MzEwMzgzNDg0MTY3Nzc3NzQ4Njk3MjMwNjYwNzc5NzY5NTc0MjEyNzU1OTE2ODU4MjE5NDM5MzUyNjIwNjM4Njc4MDgwOTA2MDQ3MDU0MTk5NjE3OTczMDUwNzUwMjgyNTI0NTQ0ODY5MTEzNDUxNDUzNjY5MjUzOTM1MzI5MjY1MTA2MDcyMTkzNzgxNjIzMTUxNzUyMDgwODE2OTI1NDQ5Njc0MzcwMDI2NTM2OTQ4MTQ2MTc4NDU1NjE3NTg5ODc1NDQwNDkzMzA4MTYxNTgzMDg1MDYwNjQ5ODUxNjEyNTc3ODc3ODk0NDg3NjQzMjQ0ODY2NTE4NTA1ODY4Nzg0NTU4NDM1MzU3OTA1NDAxNDI4MzU4MzYyMDIwNTExNTE0NTczMzk0MDYxOTg2NDAyMjk4NDIyMDc1NDkyMDkwNjY5NjM5ODg4MzkzMTc0MjM3NzA0MTM2MjIwMjg0NDA3ODY5NDQ4ODU5NTc2NzMxNzE0OTEzNTY5OTkyNjM5MDIxMjI3MzU3MDI3ODI5NjUzNzUxNTAxMDcxNTAwNjAxNzgwOTU3Mjc4NDk4MDExMDM2MjAxOTE4NTUyMjc4NDExODUxMjI1MDMzMzE2NjA1ODYyOTcwNzA4NDg4ODQyMzc1NjU0NDYzMzgyODA0NzgyNDA2Njc0NDAwNjI4NDU5Nzc5MTE0MTAwODg0MzQ3ODM3MjQ2MjA0ODAxMDIxOTUwNDgiXSwgWyJleHBpcnlfZGF0ZSIsICIyMTMzMTA0OTAzNTQwMjQ2Njg1NjU2OTE2MTQ0MjI2MzE2NjUzNTU2MjczMjA2NDA2MzUwNjE0OTA0MzcyNDE5OTcyMjQzMjQyNzkyMTgwMzkxOTk5Mjc1MDcwNzA4NDYxNzU1MDg0MDg3OTgxMzAwNzk3ODY5OTUxMzIyODg0MDYyMjAzOTQ4MTI4MDYxNjg4NzM2NDk4NzkzMzIxNjgwMjI4NDI5MTg2ODY5NzAyMTAzOTcwMzk2MjcyMjM2ODUwNTk1ODI2Njc5Mzg5OTA1ODM4MzA2Nzg5NjEwNTM1NTUzMzQ3MDkxNDExODIxMjY0MDE4MDkzNTMxMzMxMjIyMTQ0MDQ2MDM0NTMzNjM1MzkzNjYzODI1NDc0MDc3MDIwMzY1NjUxMTE0NTk0MDM4Njg0MjgyNDI4MTY1OTgxNDQ4NTkxMTkzNzk2Njg3Nzk1MTEyNjM0MjY5MDYyMzgyMDc5NzE1OTk2NDY1MTI2NDI3MTM0MTYxNDkxOTE2Mjk1OTI1MDQ5NjE1NjQyMTA4MDM4NzAzMjY0Njc5MjIyMjY5OTA5NDIwMDAzNzU1MzE0MDA2NTcyNjc5NzQ3MzY0NjI2OTUxMjEwNDkyMDY4NDMxNzA5NzQwODgxMTUzNzY3MDcwNDkxMzQ0NDEzMTYwMTI1MjIwMjMwNjMzOTY4MTExMTU0NzQ5ODk0ODU1MjU4NzY3ODE5NTIwMzIxMTAyMjE3MjkwNTM0NjkxNDU1Mzk1ODk2NjkwMDY3NDQ3NDIzMTE1MDA3MjUwNjUyMTU2MzUyNTM5OTA3MzI3MDUxNTc5NjI1MzY5NjY3NjM5NDE2MTg5Mzc2MTYyODA1MjgwMTAxMDE5NzExNDA0MTkzNTY4MTMyNzk5NjUxMDA3OTMyMDMzMjQ4OTEyMDA2Mjk4NjQ5MiJdLCBbImlzc3VlX2RhdGUiLCAiMjIwMjE3NjI1NDMzNTcyNjUzNjU5MTgxNDk1Njk5NjUxNzg1MDEzMTAwNzA0NjI3NTgyMTMwNzU2Mjg3OTI3MTMzNjYyMjE4MjM3MzM4MzQ3NTAyMTM0NjUxMjY2ODI0Njc5MDQ3OTMzODYwMDA0MDgwODA0NTQxMDkyNTg4NjU5ODcyODkwMDAzNzMwODY0MjgyMjY1MTE5MjYzMDU2Mzc3ODkwOTk2MzU3MTkyMzc1MTMzOTI1NDk1NzYwMjk2NDk0MjUwNjM0MDI4MTc3MjAwMjA2Mjg3MzUwOTcwODE4NDk1MTg4MzEzNTY2MDU0NTcyMDc0NDY0MjkyNzkyMzk3NDE4MjQ3NDE1NzA5OTIzMzkyMTUxMTI2OTQxMzkyOTUxNzA3OTEzMjYxMjEyMDYwMjMxOTkwODQ2MzM5NDY4NTAyOTMyNjUxMDUyNTM1NDUzMjA5Njg0MjkzNzM2MDUxMjk0Nzg4Njk0OTYwNjQxOTAzOTA2NDExNTczNzMyNTQ5NjI0ODEwMTkzNjM0MDMzMzQ3NzI5MzM1MTg2MTg1NjA3NjI1OTU5MTcxOTYxMjgwMjQ1NDExNzE0OTM1MjE2MzM2NjY4NTYxNzY4NzU4Njc3NjU5ODY0MDgzMzM3NDk5MjIzNjkzNDgxMjY5ODM2OTg5MzA4MzI0MDg0MzczOTY4OTA4NjYyMjA1OTIwMTY0MzgzMjk5NzQwODAxOTEwNDgwODI5NTM3NjgwMTQ1Mzk3MTM2NzA3NjU1NjQ1MDY0MzY0MjYyOTYzNzMyMjAyMTQyMTY3ODIxMDk5MTQ0MzI1MDE2NDk2MjA1NzcyMTYxODk5Njk1MjgyMDI0ODcwMzU5NzQ2Nzc3NTY5Nzc2MzQ1MzcwNTQ3MDg0NTEyOTA2NDQ0MjcxMjM1NjEwMTM5NDciXSwgWyJyZXNpZGVudF9wb3N0YWxfY29kZSIsICI5NDEyOTI1NzA1Nzc0MzUzMzU4NDc2MjM3NTk0ODk1Mzc4MDc2NTEwMzk1NDg1NDk4Njk5Mzc3OTk5MTY3NjEzNjgzNDIyNzUwMTcyNTc1ODIyMDEyNDY4NTA2MzQwMTI2NjQxMDQ1MjYxMzg4NjgzNzEwODE0NTk2NDIyNzU0MTg2MTk0MTU3NDMzODIzMDk0NjMwNDYwNzM4NzY0NTg5MDQ3NTE3ODIzNjMyMTkyMjgwOTQ2MzY3MTc2MjExMDA2ODc5MTMzMzUzMzIxODQ2OTAzNjc2MzYxMTI0MzA0NjQxMzYzNzg1NTk1MDIyOTUzMTgzMjg2NzQ5MzkwNzg0NTEwMDgzOTYzMjg1MzQyMjAwMDQ5Mjk4MzEzMTA3MDk1NTM1Mzk2OTIyNjU2MTQ1Njk0MDA4MDYyMzI3NTUzOTA5NzQ3NDYxMDU0NDc4NDIxNDA3OTMzODM2NTE3MzI4ODc0NjI5NDgxNjU0NTk3OTczODE1MjU4NDg2NTk3MzQyNTk1MzY4MjU5MzA3ODk1OTEzNjkxMjY4OTY5MDc2Nzk0ODUzNjkzNzQ4MjQxODIwMjU3MDQ0NjAwMjI1MjU2MzMxMTE3NDcxMTg0MjEyNzE4NjQ5MDExOTM0NDQwNDIwNzEzNDcxMjcyMDMyMzI0MDM1NTcyNDk0MzI4NTI3NDMyNzUxNzI3MjE4NzU2NDYwOTU5NTA2MTY4OTY1NDcyODQ0NDUzNTM4MTYyNTgwMjYxNTM2Nzk1NTU1NTE2ODIwMzU2ODgzMzM1MTgwMTIwNDQzODM1OTMyNTMwMTMxNDI3Nzk1ODg1MDc1NTM1MTEyMjQxNjIwMzEyNTk3MzgyMzk3NzI4Mjg3OTI2Nzg5MDQ5MTMyODU2MzE3Mzc0MDQ3NDA4NDY1MzgyNDQ0NjQ2ODM1Il1dfSwgIm5vbmNlIjogIjI2NTY5ODg1MzUyODY2ODY0MzAyNzM2NCJ9"},"@id":"libindy-cred-offer-0"}]}"#;
            let offer: CredentialOffer = serde_json::from_str(offer).unwrap();

            let holder_sm = HolderSM::new(offer, source_id());

            match holder_sm.state {
                HolderState::OfferReceived(state) => {
                    let expected_thread = Thread {
                        thid: Some("97faf3cb-cf17-4965-bccb-4d5af0e8d8da".to_string()),
                        pthid: None,
                        sender_order: 0,
                        received_orders: HashMap::new()
                    };
                    assert_eq!(expected_thread, state.thread);
                    Ok(())
                }
                other => Err(format!("State expected to be RequestSent, but: {:?}", other))
            }
        }
    }

    mod step {
        use super::*;

        #[test]
        fn test_holder_init() {
            let _setup = SetupAriesMocks::init();

            let holder_sm = _holder_sm();
            assert_match!(HolderState::OfferReceived(_), holder_sm.state);
        }

        #[test]
        fn test_issuer_handle_credential_request_sent_message_from_offer_received_state() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();

            match holder_sm.state {
                HolderState::RequestSent(state) => {
                    assert_eq!(0, state.thread.sender_order);
                    Ok(())
                }
                other => Err(format!("State expected to be RequestSent, but: {:?}", other))
            }
        }

        #[test]
        fn test_issuer_handle_credential_request_sent_message_from_offer_with_thread_received_state() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let thread_id = "71fa23b0-427e-4064-bf24-b375b1a2c64b";
            let credential_offer = _credential_offer().set_thread_id(thread_id);

            let mut holder_sm = HolderSM::new(credential_offer, source_id());
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();

            match holder_sm.state {
                HolderState::RequestSent(state) => {
                    assert_eq!(thread_id, state.thread.thid.unwrap());
                    assert_eq!(0, state.thread.sender_order);
                    Ok(())
                }
                other => Err(format!("State expected to be RequestSent, but: {:?}", other))
            }
        }

        #[test]
        fn test_issuer_handle_credential_request_sent_message_from_offer_received_state_for_invalid_offer() {
            let _setup = SetupAriesMocks::init();

            let credential_offer = CredentialOffer::V1(CredentialOfferV1::create().set_offers_attach(r#"{"credential offer": {}}"#).unwrap());

            let holder_sm = HolderSM::new(credential_offer, "test source".to_string());
            holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap_err();
        }

        #[test]
        fn test_issuer_handle_reject_creedntial_message_from_offer_received_state() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRejectSend((mock_connection(), None))).unwrap();

            match holder_sm.state {
                HolderState::Finished(state) => {
                    assert_eq!(3, state.status.code());
                    Ok(())
                }
                other => Err(format!("State expected to be Finished, but: {:?}", other))
            }
        }

        #[test]
        fn test_issuer_handle_other_messages_from_offer_received_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();

            holder_sm = holder_sm.handle_message(HolderMessages::ProblemReport(_problem_report())).unwrap();
            assert_match!(HolderState::OfferReceived(_), holder_sm.state);
        }

        #[test]
        fn test_issuer_handle_credential_message_from_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();
            holder_sm = holder_sm.handle_message(HolderMessages::Credential(_credential())).unwrap();

            assert_match!(HolderState::Finished(_), holder_sm.state);
            assert_eq!(VcxStateType::VcxStateAccepted as u32, holder_sm.state());
        }

        #[test]
        fn test_issuer_handle_invalid_credential_message_from_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();

            holder_sm.handle_message(HolderMessages::Credential(Credential::V1(CredentialV1::create()))).unwrap_err();
        }

        #[test]
        fn test_issuer_handle_problem_report_from_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();
            holder_sm = holder_sm.handle_message(HolderMessages::ProblemReport(_problem_report())).unwrap();

            assert_match!(HolderState::Finished(_), holder_sm.state);
            assert_eq!(VcxStateType::VcxStateNone as u32, holder_sm.state());
        }

        #[test]
        fn test_issuer_handle_reject_creedntial_message_from_request_sent_state() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRejectSend((mock_connection(), None))).unwrap();

            match holder_sm.state {
                HolderState::Finished(state) => {
                    assert_eq!(3, state.status.code());
                    Ok(())
                }
                other => Err(format!("State expected to be Finished, but: {:?}", other))
            }
        }

        #[test]
        fn test_issuer_handle_other_messages_from_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();

            holder_sm = holder_sm.handle_message(HolderMessages::CredentialOffer(_credential_offer())).unwrap();
            assert_match!(HolderState::RequestSent(_), holder_sm.state);
        }

        #[test]
        fn test_issuer_handle_message_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(HolderMessages::CredentialRequestSend(mock_connection())).unwrap();
            holder_sm = holder_sm.handle_message(HolderMessages::Credential(_credential())).unwrap();

            holder_sm = holder_sm.handle_message(HolderMessages::CredentialOffer(_credential_offer())).unwrap();
            assert_match!(HolderState::Finished(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(HolderMessages::Credential(_credential())).unwrap();
            assert_match!(HolderState::Finished(_), holder_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;
        use crate::aries::messages::issuance::credential_proposal::CredentialProposal;

        #[test]
        fn test_holder_find_message_to_handle_from_offer_received_state() {
            let _setup = SetupAriesMocks::init();

            let holder = _holder_sm();

            // No agent

            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::Credential(_credential()),
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack()),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(holder.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_holder_find_message_to_handle_from_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let holder = _holder_sm().to_request_sent_state();

            // CredentialAck
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::Credential(_credential())
                );

                let (uid, message) = holder.find_message_to_handle(messages).unwrap();
                assert_eq!("key_4", uid);
                assert_match!(A2AMessage::Credential(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::CredentialAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = holder.find_message_to_handle(messages).unwrap();
                assert_eq!("key_5", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

            // Credential Reject
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::CredentialAck(_ack()),
                    "key_5".to_string() => A2AMessage::CredentialReject(_problem_report())
                );

                let (uid, message) = holder.find_message_to_handle(messages).unwrap();
                assert_eq!("key_5", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

            // No agent for different Thread ID
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(CredentialOffer::V1(_credential_offer_v1()).set_thread_id("")),
                    "key_2".to_string() => A2AMessage::CredentialRequest(CredentialRequest::V1(_credential_request_v1()).set_thread_id("")),
                    "key_3".to_string() => A2AMessage::CredentialProposal(CredentialProposal::V1(_credential_proposal_v1()).set_thread_id("")),
                    "key_4".to_string() => A2AMessage::Credential(Credential::V1(_credential_v1().set_thread_id(""))),
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack().set_thread_id("")),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report().set_thread_id(""))
                );

                assert!(holder.find_message_to_handle(messages).is_none());
            }

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal())
                );

                assert!(holder.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_holder_find_message_to_handle_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let holder = _holder_sm().to_finished_state();

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request()),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal()),
                    "key_4".to_string() => A2AMessage::Credential(_credential()),
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack()),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(holder.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[test]
        fn test_get_state() {
            let _setup = SetupAriesMocks::init();

            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _holder_sm().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _holder_sm().to_request_sent_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _holder_sm().to_finished_state().state());
        }
    }
}
