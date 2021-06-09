use crate::api::VcxStateType;

use crate::v3::handlers::issuance::states::{HolderState, OfferReceivedState, RequestSentState, FinishedHolderState};
use crate::v3::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::v3::messages::issuance::credential::Credential;
use crate::v3::messages::issuance::credential_offer::CredentialOffer;
use crate::v3::messages::issuance::credential_request::CredentialRequest;
use crate::v3::messages::issuance::credential_ack::CredentialAck;
use crate::v3::messages::error::{ProblemReport, ProblemReportCodes, Reason};
use crate::v3::messages::a2a::A2AMessage;
use crate::v3::messages::status::Status;
use crate::v3::handlers::connection::types::CompletedConnection;

use crate::utils::libindy::anoncreds::{self, libindy_prover_store_credential, libindy_prover_delete_credential};
use crate::error::prelude::*;
use std::collections::HashMap;

use crate::object_cache::Handle;
use crate::connection::Connections;
use crate::{credential, settings};
use crate::v3::handlers::connection::agent::AgentInfo;
use crate::messages::thread::Thread;
use crate::v3::handlers::connection::connection::Connection;
use crate::utils::libindy::signus::create_and_store_my_did;

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

    pub fn get_source_id(&self) -> String {
        self.source_id.clone()
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
        trace!("Holder::find_message_to_handle >>> messages: {:?}", secret!(messages));
        debug!("Holder: Finding message to update state");

        for (uid, message) in messages {
            match self.state {
                HolderState::OfferReceived(_) => {
                    // do not process messages
                }
                HolderState::RequestSent(ref state) => {
                    match message {
                        A2AMessage::Credential(credential) => {
                            if credential.from_thread(&state.thread.thid.clone().unwrap_or_default()) {
                                debug!("Holder: Credential message received");
                                return Some((uid, A2AMessage::Credential(credential)));
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) |
                        A2AMessage::CredentialReject(problem_report) => {
                            if problem_report.from_thread(&state.thread.thid.clone().unwrap_or_default()) {
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
                    // do not process messages
                }
            };
        }
        debug!("Holder: no message to update state");
        None
    }

    pub fn step(state: HolderState, source_id: String) -> Self {
        HolderSM { state, source_id }
    }

    pub fn handle_message(self, cim: CredentialIssuanceMessage) -> VcxResult<HolderSM> {
        trace!("Holder::handle_message >>> cim: {:?}", secret!(cim));
        debug!("Holder: Updating state");

        let HolderSM { state, source_id } = self;
        let state = match state {
            HolderState::OfferReceived(state_data) => match cim {
                CredentialIssuanceMessage::CredentialRequestSend(connection_handle) => {
                    if connection_handle == 0 && state_data.offer.service.is_some() {
                        state_data.handle_ephemeral_credential_offer()?
                    } else {
                        state_data.handle_credential_offer(connection_handle)?
                    }
                }
                CredentialIssuanceMessage::CredentialRejectSend((connection_handle, comment)) => {
                    let connection = connection_handle.get_completed_connection()?;
                    let thread = state_data.thread.clone()
                        .update_received_order(&connection.data.did_doc.id);

                    let problem_report = _reject_credential(&connection, &thread, comment)?;

                    HolderState::Finished((state_data, problem_report, thread, Reason::Reject).into())
                }
                _ => {
                    warn!("Credential Issuance can only start on holder side with Credential Offer");
                    HolderState::OfferReceived(state_data)
                }
            },
            HolderState::RequestSent(state_data) => match cim {
                CredentialIssuanceMessage::Credential(credential) => {
                    let thread = state_data.thread.clone()
                        .increment_sender_order()
                        .update_received_order(&state_data.connection.data.did_doc.id);

                    match state_data.store_credential(&credential) {
                        Ok(cred_id) => {
                            if credential.please_ack.is_some() {
                                let ack = CredentialAck::create().set_thread(thread.clone());
                                state_data.connection.data.send_message(&A2AMessage::CredentialAck(ack), &state_data.connection.agent)?;
                            }

                            HolderState::Finished((state_data, cred_id, credential, thread).into())
                        }
                        Err(err) => {
                            let problem_report = ProblemReport::create()
                                .set_description(ProblemReportCodes::InvalidCredential)
                                .set_comment(format!("error occurred: {:?}", err))
                                .set_thread(thread.clone());

                            state_data.connection.data.send_message(&A2AMessage::CredentialReject(problem_report.clone()), &state_data.connection.agent)?;
                            return Err(err);
                        }
                    }
                }
                CredentialIssuanceMessage::ProblemReport(problem_report) => {
                    let thread = state_data.thread.clone()
                        .update_received_order(&state_data.connection.data.did_doc.id);

                    HolderState::Finished((state_data, problem_report, thread, Reason::Fail).into())
                }
                CredentialIssuanceMessage::CredentialRejectSend((connection_handle, comment)) => {
                    let connection = (connection_handle.get_completed_connection())?;

                    let thread = state_data.thread.clone()
                        .increment_sender_order()
                        .update_received_order(&connection.data.did_doc.id);

                    let problem_report = _reject_credential(&connection, &thread, comment)?;

                    HolderState::Finished((state_data, problem_report, thread, Reason::Reject).into())
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Credential and Problem Report");
                    HolderState::RequestSent(state_data)
                }
            },
            HolderState::Finished(state_data) => {
                warn!("Exchange is finished, no messages can be sent or received");
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
                let cred_id = state.cred_id.clone()
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
}

fn _parse_cred_def_from_cred_offer(cred_offer: &str) -> VcxResult<String> {
    trace!("Holder::_parse_cred_def_from_cred_offer >>> cred_offer: {:?}", secret!(cred_offer));

    let parsed_offer: serde_json::Value = serde_json::from_str(cred_offer)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer, format!("Cannot parse Credential Offer from JSON string. Err: {:?}", err)))?;

    let cred_def_id = parsed_offer["cred_def_id"].as_str()
        .ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer, "Invalid Credential object state: `cred_def_id` not found"))?;

    Ok(cred_def_id.to_string())
}

fn _parse_rev_reg_id_from_credential(credential: &str) -> VcxResult<Option<String>> {
    trace!("Holder::_parse_rev_reg_id_from_credential >>>");

    let parsed_credential: serde_json::Value = serde_json::from_str(credential)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredential, format!("Cannot parse Credential from JSON string. Err: {:?}", err)))?;

    let rev_reg_id = parsed_credential["rev_reg_id"].as_str().map(String::from);

    Ok(rev_reg_id)
}

fn _store_credential(credential_offer: &CredentialOffer,
                     credential: &Credential,
                     req_meta: &str,
                     cred_def_json: &str) -> VcxResult<String> {
    credential.ensure_match_offer(credential_offer)?;

    let credential_json = credential.credentials_attach.content()?;
    let rev_reg_id = _parse_rev_reg_id_from_credential(&credential_json)?;
    let rev_reg_def_json = if let Some(rev_reg_id) = rev_reg_id {
        let (_, json) = anoncreds::get_rev_reg_def_json(&rev_reg_id)?;
        Some(json)
    } else {
        None
    };

    let cred_id = libindy_prover_store_credential(None,
                                                  req_meta,
                                                  &credential_json,
                                                  cred_def_json,
                                                  rev_reg_def_json.as_ref().map(String::as_str))?;

    trace!("Holder::_store_credential <<<");
    Ok(cred_id)
}

impl RequestSentState {
    fn store_credential(&self, credential: &Credential) -> VcxResult<String> {
        trace!("Holder::_store_credential >>>");
        debug!("holder storing received credential");

        self.thread.check_message_order(&self.connection.data.did_doc.id, &credential.thread)?;

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
    fn make_credential_request(&self) -> VcxResult<(CredentialRequest, String, String)> {
        trace!("Holder::OfferReceivedState::make_credential_request >>> offer: {:?}", secret!(self.offer));
        debug!("holder preparing credential request");

        let did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        let cred_offer = self.offer.offers_attach.content()?;
        let cred_def_id = _parse_cred_def_from_cred_offer(&cred_offer)?;
        let (req, req_meta, _cred_def_id, cred_def_json) =
            credential::Credential::create_credential_request(&cred_def_id, &did, &cred_offer)?;
        self.offer.ensure_match_credential_definition(&cred_def_json)?;
        let cred_req = CredentialRequest::create().set_requests_attach(req)?;

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

        let did_doc = self.offer.service.clone().unwrap().into();
        let (_, pw_vk) = create_and_store_my_did(None, None)?;
        let message = Connection::send_message_and_wait_result(&cred_request.to_a2a_message(), &did_doc, &pw_vk)?;

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
                connection.data.send_message(&cred_request.to_a2a_message(), &connection.agent)?;
                Ok(HolderState::RequestSent((self, req_meta, cred_def_json, connection, thread).into()))
            }
            Err(err) => {
                let problem_report = ProblemReport::create()
                    .set_description(ProblemReportCodes::InvalidCredentialOffer)
                    .set_comment(format!("error occurred: {:?}", err))
                    .set_thread(thread.clone());

                connection.data.send_message(&A2AMessage::CredentialReject(problem_report.clone()), &connection.agent)?;
                return Err(err)
            }
        }
    }
}

impl FinishedHolderState {
    fn delete_credential(&self, cred_id: &str) -> VcxResult<()> {
        trace!("Holder::_delete_credential >>> cred_id: {}", cred_id);
        libindy_prover_delete_credential(cred_id)
    }
}

fn _reject_credential(connection: &CompletedConnection, thread: &Thread, comment: Option<String>) -> VcxResult<ProblemReport> {
    trace!("Holder::_reject_credential >>> comment: {:?}", secret!(comment));
    debug!("holder preparing credential reject");

    let problem_report = ProblemReport::create()
        .set_description(ProblemReportCodes::CredentialRejected)
        .set_comment(comment.unwrap_or(String::from("credential-offer was rejected")))
        .set_thread(thread.clone());

    connection.agent.send_message(&A2AMessage::CredentialReject(problem_report.clone()), &connection.data.did_doc)?;

    trace!("Holder::_reject_credential <<<");
    Ok(problem_report)
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::utils::devsetup::SetupAriesMocks;
    use crate::v3::handlers::connection::tests::mock_connection;
    use crate::v3::test::source_id;
    use crate::v3::messages::issuance::credential::tests::_credential;
    use crate::v3::messages::issuance::credential_offer::tests::_credential_offer;
    use crate::v3::messages::issuance::credential_request::tests::_credential_request;
    use crate::v3::messages::issuance::credential_proposal::tests::_credential_proposal;
    use crate::v3::messages::issuance::test::{_ack, _problem_report};

    fn _holder_sm() -> HolderSM {
        HolderSM::new(_credential_offer(), source_id())
    }

    impl HolderSM {
        fn to_request_sent_state(mut self) -> HolderSM {
            self = self.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();
            self
        }

        fn to_finished_state(mut self) -> HolderSM {
            self = self.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();
            self = self.handle_message(CredentialIssuanceMessage::Credential(_credential())).unwrap();
            self
        }
    }

    mod new {
        use super::*;

        #[test]
        fn test_holder_new() {
            let _setup = SetupAriesMocks::init();

            let holder_sm = _holder_sm();

            assert_match!(HolderState::OfferReceived(_), holder_sm.state);
            assert_eq!(source_id(), holder_sm.get_source_id());
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
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();

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
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();

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

            let credential_offer = CredentialOffer::create().set_offers_attach(r#"{"credential offer": {}}"#).unwrap();

            let holder_sm = HolderSM::new(credential_offer, "test source".to_string());
            holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap_err();
        }

        #[test]
        fn test_issuer_handle_reject_creedntial_message_from_offer_received_state() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRejectSend((mock_connection(), None))).unwrap();

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

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialSend(mock_connection())).unwrap();
            assert_match!(HolderState::OfferReceived(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::ProblemReport(_problem_report())).unwrap();
            assert_match!(HolderState::OfferReceived(_), holder_sm.state);
        }

        #[test]
        fn test_issuer_handle_credential_message_from_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::Credential(_credential())).unwrap();

            assert_match!(HolderState::Finished(_), holder_sm.state);
            assert_eq!(VcxStateType::VcxStateAccepted as u32, holder_sm.state());
        }

        #[test]
        fn test_issuer_handle_invalid_credential_message_from_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();

            holder_sm.handle_message(CredentialIssuanceMessage::Credential(Credential::create())).unwrap_err();
        }

        #[test]
        fn test_issuer_handle_problem_report_from_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::ProblemReport(_problem_report())).unwrap();

            assert_match!(HolderState::Finished(_), holder_sm.state);
            assert_eq!(VcxStateType::VcxStateNone as u32, holder_sm.state());
        }

        #[test]
        fn test_issuer_handle_reject_creedntial_message_from_request_sent_state() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRejectSend((mock_connection(), None))).unwrap();

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
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialOffer(_credential_offer())).unwrap();
            assert_match!(HolderState::RequestSent(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialAck(_ack())).unwrap();
            assert_match!(HolderState::RequestSent(_), holder_sm.state);
        }

        #[test]
        fn test_issuer_handle_message_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let mut holder_sm = _holder_sm();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialRequestSend(mock_connection())).unwrap();
            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::Credential(_credential())).unwrap();

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialOffer(_credential_offer())).unwrap();
            assert_match!(HolderState::Finished(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::Credential(_credential())).unwrap();
            assert_match!(HolderState::Finished(_), holder_sm.state);

            holder_sm = holder_sm.handle_message(CredentialIssuanceMessage::CredentialAck(_ack())).unwrap();
            assert_match!(HolderState::Finished(_), holder_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        fn test_holder_find_message_to_handle_from_offer_received_state() {
            let _setup = SetupAriesMocks::init();

            let holder = _holder_sm();

            // No messages

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

            // No messages for different Thread ID
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer().set_thread_id("")),
                    "key_2".to_string() => A2AMessage::CredentialRequest(_credential_request().set_thread_id("")),
                    "key_3".to_string() => A2AMessage::CredentialProposal(_credential_proposal().set_thread_id("")),
                    "key_4".to_string() => A2AMessage::Credential(_credential().set_thread_id("")),
                    "key_5".to_string() => A2AMessage::CredentialAck(_ack().set_thread_id("")),
                    "key_6".to_string() => A2AMessage::CommonProblemReport(_problem_report().set_thread_id(""))
                );

                assert!(holder.find_message_to_handle(messages).is_none());
            }

            // No messages
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

            // No messages
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
