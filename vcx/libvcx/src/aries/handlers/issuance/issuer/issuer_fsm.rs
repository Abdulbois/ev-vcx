use std::collections::HashMap;

use crate::api::VcxStateType;
use crate::aries::handlers::{
    issuance::{
        issuer::messages::IssuerMessages,
        issuer::states::*,
    },
    connection::agent::AgentInfo,
};
use crate::aries::messages::{
    a2a::A2AMessage,
    issuance::{
        credential_offer::CredentialOffer,
        credential::Credential,
        v10::credential_offer::CredentialOffer as CredentialOfferV1,
        v10::credential::Credential as CredentialV1,
        v20::credential::Credential as CredentialV2,
    },
    error::{ProblemReport, ProblemReportCodes},
    mime_type::MimeType,
    status::Status,
};
use crate::aries::messages::thread::Thread;
use crate::issuer_credential::encode_attributes;
use crate::utils::libindy::anoncreds::{
    self,
    libindy_issuer_create_credential_offer,
};
use crate::error::{VcxResult, VcxError, VcxErrorKind};
use crate::connection::Connections;
use crate::utils::object_cache::Handle;
use crate::aries::messages::issuance::credential_request::CredentialRequest;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuerSM {
    state: IssuerState,
    source_id: String,
}

impl IssuerSM {
    pub fn new(cred_def_id: &str, credential_data: &str, rev_reg_id: Option<String>,
               tails_file: Option<String>, source_id: &str, credential_name: &str) -> Self {
        IssuerSM {
            state: IssuerState::Initial(InitialState::new(cred_def_id,
                                                          credential_data,
                                                          rev_reg_id,
                                                          tails_file,
                                                          Some(credential_name.to_string()))),
            source_id: source_id.to_string(),
        }
    }

    pub fn get_source_id(&self) -> &String {
        &self.source_id
    }

    pub fn step(state: IssuerState, source_id: String) -> Self {
        IssuerSM {
            state,
            source_id,
        }
    }

    pub fn update_state(self) -> VcxResult<Self> {
        trace!("Issuer::update_state >>> ", );

        if self.is_terminal_state() { return Ok(self); }

        let agent = match self.get_agent_info() {
            Some(agent_info) => agent_info.clone(),
            None => {
                warn!("Could not update Issuer state: no information about Connection.");
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
        trace!("Issuer::find_message_to_handle >>> agent: {:?}", secret!(messages));
        debug!("Issuer: Finding message to update state");

        for (uid, message) in messages {
            match self.state {
                IssuerState::Initial(_) => {
                    // do not process agent
                }
                IssuerState::OfferSent(ref state) => {
                    match message {
                        A2AMessage::CredentialRequest(credential) => {
                            if credential.from_thread(state.thread.thid.as_deref().unwrap_or_default()) {
                                debug!("Issuer: CredentialRequest message received");
                                return Some((uid, A2AMessage::CredentialRequest(credential)));
                            }
                        }
                        A2AMessage::CredentialProposal(credential_proposal) => {
                            if let Some(ref thread) = credential_proposal.thread() {
                                debug!("Issuer: CredentialProposal message received");
                                if thread.is_reply(state.thread.thid.as_deref().unwrap_or_default()) {
                                    return Some((uid, A2AMessage::CredentialProposal(credential_proposal)));
                                }
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) |
                        A2AMessage::CredentialReject(problem_report) => {
                            if problem_report.from_thread(state.thread.thid.as_deref().unwrap_or_default()) {
                                debug!("Issuer: CredentialReject message received");
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        message => {
                            warn!("Issuer: Unexpected message received in OfferSent state: {:?}", message);
                        }
                    }
                }
                IssuerState::RequestReceived(_) => {
                    // do not process agent
                }
                IssuerState::CredentialSent(ref state) => {
                    match message {
                        A2AMessage::Ack(ack) | A2AMessage::CredentialAck(ack) => {
                            if ack.from_thread(state.thread.thid.as_deref().unwrap_or_default()) {
                                return Some((uid, A2AMessage::CredentialAck(ack)));
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) |
                        A2AMessage::CredentialReject(problem_report) => {
                            if problem_report.from_thread(state.thread.thid.as_deref().unwrap_or_default()) {
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        message => {
                            warn!("Issuer: Unexpected message received in CredentialSent state: {:?}", message);
                        }
                    }
                }
                IssuerState::Finished(_) => {
                    // do not process agent
                }
            };
        }
        debug!("Issuer: no message to update state");
        None
    }

    pub fn state(&self) -> u32 {
        match self.state {
            IssuerState::Initial(_) => VcxStateType::VcxStateInitialized as u32,
            IssuerState::OfferSent(_) => VcxStateType::VcxStateOfferSent as u32,
            IssuerState::RequestReceived(_) => VcxStateType::VcxStateRequestReceived as u32,
            IssuerState::CredentialSent(_) => VcxStateType::VcxStateAccepted as u32,
            IssuerState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    Status::Rejected(_) => VcxStateType::VcxStateRejected as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
        }
    }

    pub fn handle_message(self, cim: IssuerMessages) -> VcxResult<IssuerSM> {
        trace!("Issuer::handle_message >>> cim: {:?}", secret!(cim));
        debug!("Issuer: Updating state");

        let IssuerSM { state, source_id } = self;
        let state = match state {
            IssuerState::Initial(state_data) => match cim {
                IssuerMessages::CredentialInit(connection_handle) => {
                    state_data.init_credential(connection_handle)?
                }
                _ => {
                    warn!("Credential Issuance can only start on issuer side with init");
                    IssuerState::Initial(state_data)
                }
            }
            IssuerState::OfferSent(state_data) => match cim {
                IssuerMessages::CredentialRequest(request) => {
                    let thread = state_data.thread.clone();
                    IssuerState::RequestReceived((state_data, request, thread).into())
                }
                IssuerMessages::CredentialProposal(_) => {
                    state_data.handle_received_credential_proposal()?
                }
                IssuerMessages::ProblemReport(problem_report) => {
                    let thread = state_data.thread.clone()
                        .update_received_order(&state_data.connection.data.did_doc.id);
                    IssuerState::Finished((state_data, Status::Rejected(Some(problem_report)), thread).into())
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Request, Proposal and Problem Report");
                    IssuerState::OfferSent(state_data)
                }
            },
            IssuerState::RequestReceived(state_data) => match cim {
                IssuerMessages::CredentialSend(connection_handle) => {
                    state_data.send_credential(connection_handle)?
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only CredentialSend");
                    IssuerState::RequestReceived(state_data)
                }
            }
            IssuerState::CredentialSent(state_data) => match cim {
                IssuerMessages::ProblemReport(problem_report) => {
                    info!("Interaction closed with failure");
                    let thread = state_data.thread.clone()
                        .update_received_order(&state_data.connection.data.did_doc.id);
                    IssuerState::Finished((state_data, Status::Rejected(Some(problem_report)), thread).into())
                }
                IssuerMessages::CredentialAck(_ack) => {
                    info!("Interaction closed with success");
                    let thread = state_data.thread.clone()
                        .update_received_order(&state_data.connection.data.did_doc.id);
                    IssuerState::Finished((state_data, thread).into())
                }
                _ => {
                    warn!("In this state Credential Issuance can accept only Ack and Problem Report");
                    IssuerState::CredentialSent(state_data)
                }
            }
            IssuerState::Finished(state_data) => {
                warn!("Exchange is finished, no agent can be sent or received");
                IssuerState::Finished(state_data)
            }
        };

        trace!("Issuer::handle_message <<< state: {:?}", secret!(state));
        Ok(IssuerSM::step(state, source_id))
    }

    pub fn is_terminal_state(&self) -> bool {
        match self.state {
            IssuerState::Finished(_) => true,
            _ => false
        }
    }

    pub fn get_agent_info(&self) -> Option<&AgentInfo> {
        match self.state {
            IssuerState::OfferSent(ref state) => Some(&state.connection.agent),
            IssuerState::RequestReceived(ref state) => Some(&state.connection.agent),
            IssuerState::CredentialSent(ref state) => Some(&state.connection.agent),
            IssuerState::Initial(_) => None,
            IssuerState::Finished(_) => None,
        }
    }

    pub fn get_credential_offer(&self) -> Option<&CredentialOffer> {
        match self.state {
            IssuerState::Initial(_) => None,
            IssuerState::OfferSent(ref state) => Some(&state.offer),
            IssuerState::RequestReceived(ref state) => Some(&state.offer),
            IssuerState::CredentialSent(ref state) => Some(&state.offer),
            IssuerState::Finished(ref state) => state.offer.as_ref(),
        }
    }

    pub fn problem_report(&self) -> Option<&ProblemReport> {
        match self.state {
            IssuerState::Initial(_) |
            IssuerState::OfferSent(_) |
            IssuerState::RequestReceived(_) |
            IssuerState::CredentialSent(_) => None,
            IssuerState::Finished(ref status) => {
                match &status.status {
                    Status::Success | Status::Undefined => None,
                    Status::Rejected(ref problem_report) => problem_report.as_ref(),
                    Status::Failed(problem_report) => Some(problem_report),
                }
            }
        }
    }
}

impl InitialState {
    fn init_credential(self, connection_handle: Handle<Connections>) -> VcxResult<IssuerState> {
        let cred_offer = libindy_issuer_create_credential_offer(&self.cred_def_id)?;
        let cred_offer_msg = CredentialOffer::V1(
            CredentialOfferV1::create()
                .set_comment(self.credential_name.clone())
                .set_offers_attach(&cred_offer)?
        );
        let cred_offer_msg = self.append_credential_preview(cred_offer_msg)?;

        let connection = connection_handle.get_completed_connection()?;

        let thread = Thread::new()
            .set_thid(cred_offer_msg.id())
            .set_opt_pthid(connection.data.thread.pthid.clone());

        connection.data.send_message(&cred_offer_msg, &connection.agent)?;
        Ok(IssuerState::OfferSent((self, cred_offer_msg, connection, thread).into()))
    }

    fn append_credential_preview(&self, cred_offer_msg: CredentialOffer) -> VcxResult<CredentialOffer> {
        trace!("Issuer::InitialState::append_credential_preview >>> cred_offer_msg: {:?}", secret!(cred_offer_msg));

        let cred_values: serde_json::Value = serde_json::from_str(&self.credential_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure,
                                              format!("Cannot parse Credential Preview from JSON string. Err: {:?}", err)))?;

        let values_map = cred_values.as_object()
            .ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure,
                                              "Invalid Credential Preview Json".to_string()))?;

        let mut new_offer = cred_offer_msg;
        for item in values_map.iter() {
            let (key, value) = item;
            new_offer = new_offer.add_credential_preview_data(key, value, MimeType::Plain)?;
        }

        trace!("Issuer::InitialState::append_credential_preview <<<");
        Ok(new_offer)
    }
}

impl OfferSentState {
    fn handle_received_credential_proposal(self) -> VcxResult<IssuerState> {
        let thread = self.thread.clone()
            .increment_sender_order()
            .update_received_order(&self.connection.data.did_doc.id);

        let problem_report = ProblemReport::create()
            .set_message_type(self.offer.type_())
            .set_description(ProblemReportCodes::Unimplemented)
            .set_comment(String::from("credential-proposal message is not supported"))
            .set_thread(thread.clone());

        self.connection.data.send_message(&problem_report, &self.connection.agent)?;
        Ok(IssuerState::Finished((self, Status::Failed(problem_report), thread).into()))
    }
}

impl RequestReceivedState {
    fn send_credential(self, connection_handle: Handle<Connections>) -> VcxResult<IssuerState> {
        let connection = connection_handle.get_completed_connection()?;

        let thread = self.request.thread().clone()
            .increment_sender_order()
            .update_received_order(&self.connection.data.did_doc.id);

        match self.create_credential(&thread) {
            Ok(credential_msg) => {
                connection.data.send_message(&credential_msg, &connection.agent)?;
                Ok(IssuerState::Finished((self, thread).into()))
            }
            Err(err) => {
                let problem_report = ProblemReport::create()
                    .set_message_type(self.offer.type_())
                    .set_description(ProblemReportCodes::InvalidCredentialRequest)
                    .set_comment(format!("error occurred: {:?}", err))
                    .set_thread(thread.clone());

                self.connection.data.send_message(&problem_report, &connection.agent)?;
                return Err(err);
            }
        }
    }

    fn create_credential(&self, thread: &Thread) -> VcxResult<Credential> {
        trace!("Issuer::RequestReceivedState::create_credential >>>");

        self.thread.check_message_order(&self.connection.data.did_doc.id, self.request.thread())?;

        let (_, request) = &self.request.requests_attach().content()?;

        let cred_data = encode_attributes(&self.cred_data)?;
        let (_, cred_offer_attachment) = self.offer.offer_attach().content()?;

        let (credential, _, _) = anoncreds::libindy_issuer_create_credential(&cred_offer_attachment,
                                                                             &request,
                                                                             &cred_data,
                                                                             self.rev_reg_id.as_deref(),
                                                                             self.tails_file.as_deref())?;

        let credential = match self.request {
            CredentialRequest::V1(_) =>
                Credential::V1(
                    CredentialV1::create()
                        .set_credential(credential)?
                        .set_thread(thread.clone())
                ),
            CredentialRequest::V2(_) =>
                Credential::V2(
                    CredentialV2::create()
                        .set_indy_credential_attach(&credential)?
                        .set_thread(thread.clone())
                )
        };

        trace!("Issuer::RequestReceivedState::create_credential <<<");
        Ok(credential)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::utils::devsetup::SetupAriesMocks;
    use crate::aries::handlers::connection::tests::mock_connection;
    use crate::aries::test::source_id;
    use crate::aries::messages::issuance::credential::tests::_credential;
    use crate::aries::messages::issuance::credential_request::tests::_credential_request;
    use crate::aries::messages::issuance::credential_offer::tests::_credential_offer;
    use crate::aries::messages::issuance::credential_proposal::tests::_credential_proposal;
    use crate::aries::messages::issuance::test::{_ack, _problem_report};
    use crate::aries::messages::issuance::v10::credential::tests::_credential as _credential_v1;
    use crate::aries::messages::issuance::v10::credential_offer::tests::_credential_offer as _credential_offer_v1;
    use crate::aries::messages::issuance::v10::credential_proposal::tests::_credential_proposal as _credential_proposal_v1;
    use crate::aries::messages::issuance::v10::credential_request::tests::_credential_request as _credential_request_v1;

    fn _issuer_sm() -> IssuerSM {
        IssuerSM::new("test", &json!({"name": "alice"}).to_string(), None, None, &source_id(), "test")
    }

    impl IssuerSM {
        fn to_offer_sent_state(mut self) -> IssuerSM {
            self = self.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            self
        }

        fn to_request_received_state(mut self) -> IssuerSM {
            self = self.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            self = self.handle_message(IssuerMessages::CredentialRequest(_credential_request())).unwrap();
            self
        }

        fn to_finished_state(mut self) -> IssuerSM {
            self = self.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            self = self.handle_message(IssuerMessages::CredentialRequest(_credential_request())).unwrap();
            self = self.handle_message(IssuerMessages::CredentialSend(mock_connection())).unwrap();
            self
        }
    }

    mod new {
        use super::*;

        #[test]
        fn test_issuer_new() {
            let _setup = SetupAriesMocks::init();

            let issuer_sm = _issuer_sm();

            assert_match!(IssuerState::Initial(_), issuer_sm.state);
            assert_eq!(source_id(), issuer_sm.get_source_id().to_string());
        }
    }

    mod handle_message {
        use super::*;
        use crate::aries::messages::issuance::credential_request::CredentialRequest;
        use crate::aries::messages::issuance::v10::credential_request::CredentialRequest as CredentialRequestV1;

        #[test]
        fn test_issuer_init() {
            let _setup = SetupAriesMocks::init();

            let issuer_sm = _issuer_sm();

            assert_match!(IssuerState::Initial(_), issuer_sm.state);
        }

        #[test]
        fn test_issuer_handle_credential_init_message_from_initial_state() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();

            assert_match!(IssuerState::OfferSent(_), issuer_sm.state);
        }

        #[test]
        fn test_issuer_handle_other_messages_from_initial_state() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();

            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialRequest(_credential_request())).unwrap();
            assert_match!(IssuerState::Initial(_), issuer_sm.state);
        }

        #[test]
        fn test_issuer_handle_credential_request_message_from_offer_sent_state() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();

            let credential_request = _credential_request();

            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialRequest(credential_request.clone())).unwrap();

            match issuer_sm.state {
                IssuerState::RequestReceived(state) => {
//                    assert_eq!(credential_request.thread.thid, state.thread.thid);
                    assert_eq!(0, state.thread.sender_order);
                    Ok(())
                }
                other => Err(format!("State expected to be RequestReceived, but: {:?}", other))
            }
        }

        #[test]
        fn test_issuer_handle_credential_proposal_message_from_offer_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialProposal(_credential_proposal())).unwrap();

            assert_match!(IssuerState::Finished(_), issuer_sm.state);
            assert_eq!(VcxStateType::VcxStateNone as u32, issuer_sm.state());
        }

        #[test]
        fn test_issuer_handle_problem_report_message_from_offer_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::ProblemReport(_problem_report())).unwrap();

            assert_match!(IssuerState::Finished(_), issuer_sm.state);
            assert_eq!(VcxStateType::VcxStateRejected as u32, issuer_sm.state());
        }

        #[test]
        fn test_issuer_handle_other_messages_from_offer_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialAck(_ack())).unwrap();

            assert_match!(IssuerState::OfferSent(_), issuer_sm.state);
        }

        #[test]
        fn test_issuer_handle_credential_send_message_from_request_received_state() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialRequest(_credential_request())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialSend(mock_connection())).unwrap();

            assert_match!(IssuerState::Finished(_), issuer_sm.state);
            assert_eq!(VcxStateType::VcxStateAccepted as u32, issuer_sm.state());
        }

        #[test]
        fn test_issuer_handle_credential_send_message_from_request_received_state_with_invalid_request() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialRequest(
                CredentialRequest::V1(CredentialRequestV1::create())
            )).unwrap();

            issuer_sm.handle_message(IssuerMessages::CredentialSend(mock_connection())).unwrap_err();
        }

        #[test]
        fn test_issuer_handle_other_messages_from_request_received_state() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialRequest(_credential_request())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialSend(mock_connection())).unwrap();

            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialSend(mock_connection())).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialAck(_ack())).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);
        }

        // TRANSITIONS TO/FROM CREDENTIAL SENT STATE AREN'T POSSIBLE NOW

        #[test]
        fn test_issuer_handle_messages_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let mut issuer_sm = _issuer_sm();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialRequest(_credential_request())).unwrap();
            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialSend(mock_connection())).unwrap();

            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialInit(mock_connection())).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);

            issuer_sm = issuer_sm.handle_message(IssuerMessages::CredentialRequest(_credential_request())).unwrap();
            assert_match!(IssuerState::Finished(_), issuer_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;
        use crate::aries::messages::issuance::credential_proposal::CredentialProposal;

        #[test]
        fn test_issuer_find_message_to_handle_from_initial_state() {
            let _setup = SetupAriesMocks::init();

            let issuer = _issuer_sm();

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

                assert!(issuer.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_issuer_find_message_to_handle_from_offer_sent_state() {
            let _setup = SetupAriesMocks::init();

            let issuer = _issuer_sm().to_offer_sent_state();

            // CredentialRequest
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::Credential(_credential()),
                    "key_3".to_string() => A2AMessage::CredentialRequest(_credential_request())
                );

                let (uid, message) = issuer.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CredentialRequest(_), message);
            }

            // CredentialProposal
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialAck(_ack()),
                    "key_3".to_string() => A2AMessage::Credential(_credential()),
                    "key_4".to_string() => A2AMessage::CredentialProposal(_credential_proposal())
                );

                let (uid, message) = issuer.find_message_to_handle(messages).unwrap();
                assert_eq!("key_4", uid);
                assert_match!(A2AMessage::CredentialProposal(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialAck(_ack()),
                    "key_3".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = issuer.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

            // Credential Reject
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialAck(_ack()),
                    "key_3".to_string() => A2AMessage::CredentialReject(_problem_report())
                );

                let (uid, message) = issuer.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
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

                assert!(issuer.find_message_to_handle(messages).is_none());
            }

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::CredentialOffer(_credential_offer()),
                    "key_2".to_string() => A2AMessage::CredentialAck(_ack())
                );

                assert!(issuer.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_issuer_find_message_to_handle_from_request_state() {
            let _setup = SetupAriesMocks::init();

            let issuer = _issuer_sm().to_finished_state();

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

                assert!(issuer.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_issuer_find_message_to_handle_from_credential_sent_state() {
            let _setup = SetupAriesMocks::init();

            let issuer = _issuer_sm().to_finished_state();

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

                assert!(issuer.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[test]
        fn test_get_state() {
            let _setup = SetupAriesMocks::init();

            assert_eq!(VcxStateType::VcxStateInitialized as u32, _issuer_sm().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _issuer_sm().to_offer_sent_state().state());
            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _issuer_sm().to_request_received_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _issuer_sm().to_finished_state().state());
        }
    }
}
