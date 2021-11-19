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
                    let thread = state_data.thread.clone()
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
        let thread = self.thread.clone()
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
