use std::collections::HashMap;

use crate::api::VcxStateType;
use crate::aries::handlers::{
    proof_presentation::verifier::{
        messages::VerifierMessages,
        states::*,
    },
    connection::agent::AgentInfo,
};
use crate::aries::messages::{
    a2a::A2AMessage,
    error::{ProblemReport, ProblemReportCodes},
    proof_presentation::{
        presentation::Presentation,
        presentation_ack::PresentationAck,
        presentation_proposal::PresentationProposal,
        presentation_request::PresentationRequest,
        v10::presentation_request::{
            PresentationRequest as PresentationRequestV1,
            PresentationRequestData,
        },
        v20::presentation_request::PresentationRequest as PresentationRequestV2,
    },
    status::Status,
};
use crate::proof::Proof;
use crate::error::prelude::*;
use crate::aries::messages::thread::Thread;
use crate::utils::object_cache::Handle;
use crate::connection::Connections;
use crate::aries::handlers::connection::types::CompletedConnection;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifierSM {
    source_id: String,
    state: VerifierState,
}

impl VerifierSM {
    pub fn new(presentation_request: PresentationRequestData, source_id: String) -> VerifierSM {
        VerifierSM {
            source_id,
            state: VerifierState::Initiated(
                InitialState { presentation_request_data: presentation_request }
            ),
        }
    }

    pub fn new_from_proposal(presentation_proposal: PresentationProposal, source_id: String) -> VerifierSM {
        // ensure thid is set.
        let thread = match presentation_proposal.thread() {
            Some(thread) =>
                if thread.thid.is_some() {
                    thread.clone()
                } else {
                    thread.clone().set_thid(presentation_proposal.id())
                },
            None => Thread::new().set_thid(presentation_proposal.id())
        };

        VerifierSM {
            source_id,
            state: VerifierState::PresentationProposalReceived(
                PresentationProposalReceivedState {
                    presentation_proposal,
                    connection: None,
                    thread,
                }
            ),
        }
    }
}

impl VerifierSM {
    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("VerifierSM::find_message_to_handle >>> agent: {:?}", secret!(messages));
        debug!("Verifier: Finding message to update state");

        for (uid, message) in messages {
            match self.state {
                VerifierState::Initiated(_) => {
                    // do not process message
                }
                VerifierState::PresentationRequestPrepared(ref state) => {
                    match message {
                        A2AMessage::Presentation(presentation) => {
                            if presentation.from_thread(&state.presentation_request.id()) {
                                debug!("Verifier: Presentation message received");
                                return Some((uid, A2AMessage::Presentation(presentation)));
                            }
                        }
                        A2AMessage::PresentationProposal(proposal) => {
                            match proposal.thread().as_ref() {
                                Some(thread) if thread.is_reply(&state.presentation_request.id()) => {
                                    debug!("Verifier: PresentationProposal message received");
                                    return Some((uid, A2AMessage::PresentationProposal(proposal)));
                                }
                                _ => return None
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) |
                        A2AMessage::PresentationReject(problem_report) => {
                            if problem_report.from_thread(&state.presentation_request.id()) {
                                debug!("Verifier: PresentationReject message received");
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        message => {
                            warn!("Verifier: Unexpected message received in OfferSent state: {:?}", message);
                        }
                    }
                }
                VerifierState::PresentationRequestSent(ref state) => {
                    match message {
                        A2AMessage::Presentation(presentation) => {
                            if presentation.from_thread(&state.thread.thid.clone().unwrap_or_default()) {
                                debug!("Verifier: Presentation message received");
                                return Some((uid, A2AMessage::Presentation(presentation)));
                            }
                        }
                        A2AMessage::PresentationProposal(proposal) => {
                            match proposal.thread().as_ref() {
                                Some(thread) if thread.is_reply(&state.thread.thid.clone().unwrap_or_default()) => {
                                    debug!("Verifier: PresentationProposal message received");
                                    return Some((uid, A2AMessage::PresentationProposal(proposal)));
                                }
                                _ => return None
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) |
                        A2AMessage::PresentationReject(problem_report) => {
                            if problem_report.from_thread(&state.thread.thid.clone().unwrap_or_default()) {
                                debug!("Verifier: PresentationReject message received");
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        _ => {}
                    }
                }
                VerifierState::PresentationProposalReceived(_) => {
                    // do not process message
                }
                VerifierState::Finished(_) => {
                    // do not process message
                }
            };
        }
        debug!("verifier: no message to update state");
        None
    }

    pub fn step(self, message: VerifierMessages) -> VcxResult<VerifierSM> {
        trace!("VerifierSM::step >>> message: {:?}", secret!(message));
        debug!("verifier updating state");

        let VerifierSM { source_id, state } = self;

        let state = match state {
            VerifierState::Initiated(state) => {
                match message {
                    VerifierMessages::SendPresentationRequest(connection_handle) => {
                        state.send_presentation_request(connection_handle)?
                    }
                    VerifierMessages::PreparePresentationRequest() => {
                        state.prepare_presentation_request()?
                    }
                    _ => {
                        VerifierState::Initiated(state)
                    }
                }
            }
            VerifierState::PresentationRequestPrepared(state) => {
                match message {
                    VerifierMessages::SetConnection(connection_handle) => {
                        let connection = connection_handle.get_completed_connection()?;
                        VerifierState::PresentationRequestPrepared((state, connection).into())
                    }
                    VerifierMessages::SendPresentationRequest(connection_handle) => {
                        state.send_presentation_request(connection_handle)?
                    }
                    VerifierMessages::PresentationReceived(presentation) => {
                        state.handle_received_presentation(presentation)?
                    }
                    VerifierMessages::PresentationRejectReceived(problem_report) => {
                        state.handle_received_presentation_reject(problem_report)?
                    }
                    VerifierMessages::PresentationProposalReceived(proposal) => {
                        state.handle_received_presentation_proposal(proposal)?
                    }
                    _ => {
                        VerifierState::PresentationRequestPrepared(state)
                    }
                }
            }
            VerifierState::PresentationRequestSent(state) => {
                match message {
                    VerifierMessages::PresentationReceived(presentation) => {
                        state.handle_received_presentation(presentation)?
                    }
                    VerifierMessages::PresentationRejectReceived(problem_report) => {
                        let thread = state.thread.clone()
                            .update_received_order(&state.connection.data.did_doc.id);
                        VerifierState::Finished((state, Status::Rejected(Some(problem_report)), thread).into())
                    }
                    VerifierMessages::PresentationProposalReceived(presentation_proposal) => { // TODO: handle Presentation Proposal
                        let thread = state.thread.clone()
                            .update_received_order(&state.connection.data.did_doc.id);
                        VerifierState::PresentationProposalReceived((state, presentation_proposal, thread).into())
                    }
                    _ => {
                        VerifierState::PresentationRequestSent(state)
                    }
                }
            }
            VerifierState::PresentationProposalReceived(state) => {
                match message {
                    VerifierMessages::RequestPresentation(connection_handle, presentation_request_data) => {
                        state.send_presentation_request(connection_handle, presentation_request_data)?
                    }
                    VerifierMessages::SendPresentationRequest(connection_handle) => {
                        state.send_presentation_request_from_proposal(connection_handle)?
                    }
                    _ => {
                        VerifierState::PresentationProposalReceived(state)
                    }
                }
            }
            VerifierState::Finished(state) => VerifierState::Finished(state),
        };

        Ok(VerifierSM { source_id, state })
    }

    pub fn source_id(&self) -> String { self.source_id.clone() }

    pub fn state(&self) -> u32 {
        match self.state {
            VerifierState::Initiated(_) => VcxStateType::VcxStateInitialized as u32,
            VerifierState::PresentationRequestPrepared(_) => VcxStateType::VcxStateInitialized as u32,
            VerifierState::PresentationRequestSent(_) => VcxStateType::VcxStateOfferSent as u32,
            VerifierState::PresentationProposalReceived(_) => VcxStateType::VcxStateRequestReceived as u32,
            VerifierState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    Status::Rejected(_) => VcxStateType::VcxStateRejected as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
        }
    }

    pub fn presentation_status(&self) -> u32 {
        match self.state {
            VerifierState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code()
        }
    }

    pub fn has_transitions(&self) -> bool {
        match self.state {
            VerifierState::Initiated(_) => false,
            VerifierState::PresentationRequestPrepared(_) => true,
            VerifierState::PresentationRequestSent(_) => true,
            VerifierState::PresentationProposalReceived(_) => false,
            VerifierState::Finished(_) => false,
        }
    }

    pub fn get_agent_info(&self) -> Option<&AgentInfo> {
        match self.state {
            VerifierState::Initiated(_) => None,
            VerifierState::PresentationRequestPrepared(ref state) => state.connection.as_ref().map(|connection| &connection.agent),
            VerifierState::PresentationRequestSent(ref state) => Some(&state.connection.agent),
            VerifierState::PresentationProposalReceived(ref state) =>
                match state.connection {
                    Some(ref connection) => Some(&connection.agent),
                    None => None
                }
            VerifierState::Finished(_) => None,
        }
    }

    pub fn presentation_request_data(&self) -> VcxResult<&PresentationRequestData> {
        match self.state {
            VerifierState::Initiated(ref state) => Ok(&state.presentation_request_data),
            VerifierState::PresentationRequestPrepared(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                    format!("Verifier object {} in state {} not ready to get Presentation Request Data message", self.source_id, self.state()))),
            VerifierState::PresentationRequestSent(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                format!("Verifier object {} in state {} not ready to get Presentation Request Data message", self.source_id, self.state()))),
            VerifierState::PresentationProposalReceived(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                     format!("Verifier object {} in state {} not ready to get Presentation Request Data message", self.source_id, self.state()))),
            VerifierState::Finished(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                 format!("Verifier object {} in state {} not ready to get Presentation Request Data message", self.source_id, self.state()))),
        }
    }

    pub fn presentation_request(&self) -> VcxResult<PresentationRequest> {
        match self.state {
            VerifierState::Initiated(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Could not get Presentation Request message. VerifierSM is not in appropriate state.")),
            VerifierState::PresentationRequestPrepared(ref state) => Ok(state.presentation_request.clone()),
            VerifierState::PresentationProposalReceived(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                     format!("Verifier object {} in state {} not ready to get Presentation Request Data message", self.source_id, self.state()))),
            VerifierState::PresentationRequestSent(ref state) => Ok(state.presentation_request.clone()),
            VerifierState::Finished(ref state) => Ok(state.presentation_request.clone()),
        }
    }

    pub fn presentation(&self) -> VcxResult<Presentation> {
        match self.state {
            VerifierState::Initiated(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                  format!("Verifier object {} in state {} not ready to get Presentation message", self.source_id, self.state()))),
            VerifierState::PresentationRequestPrepared(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                    format!("Verifier object {} in state {} not ready to get Presentation message", self.source_id, self.state()))),
            VerifierState::PresentationRequestSent(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                format!("Verifier object {} in state {} not ready to get Presentation message", self.source_id, self.state()))),
            VerifierState::PresentationProposalReceived(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                     format!("Verifier object {} in state {} not ready to get Presentation message", self.source_id, self.state()))),
            VerifierState::Finished(ref state) => {
                state.presentation.clone()
                    .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Verifier object state: `presentation` not found", self.source_id)))
            }
        }
    }

    pub fn presentation_proposal(&self) -> VcxResult<&PresentationProposal> {
        match self.state {
            VerifierState::Initiated(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                  format!("Verifier object {} in state {} not ready to get Presentation proposal message", self.source_id, self.state()))),
            VerifierState::PresentationRequestPrepared(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                    format!("Verifier object {} in state {} not ready to get Presentation proposal message", self.source_id, self.state()))),
            VerifierState::PresentationRequestSent(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                                format!("Verifier object {} in state {} not ready to get Presentation proposal message", self.source_id, self.state()))),
            VerifierState::PresentationProposalReceived(ref state) => Ok(&state.presentation_proposal),
            VerifierState::Finished(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                 format!("Verifier object {} in state {} not ready to get Presentation proposal message", self.source_id, self.state()))),
        }
    }

    pub fn problem_report(&self) -> Option<&ProblemReport> {
        match self.state {
            VerifierState::Initiated(_) |
            VerifierState::PresentationRequestPrepared(_) |
            VerifierState::PresentationRequestSent(_) |
            VerifierState::PresentationProposalReceived(_) => None,
            VerifierState::Finished(ref status) => {
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
    fn prepare_presentation_request(self) -> VcxResult<VerifierState> {
        let presentation_request: PresentationRequestData = self.presentation_request_data.clone();

        let presentation_request =
            PresentationRequest::V1(
                PresentationRequestV1::create()
                    .set_comment(presentation_request.name.clone())
                    .set_request_presentations_attach(&presentation_request)?
            );

        Ok(VerifierState::PresentationRequestPrepared((self, presentation_request).into()))
    }

    fn send_presentation_request(self, connection_handle: Handle<Connections>) -> VcxResult<VerifierState> {
        let connection: CompletedConnection = connection_handle.get_completed_connection()?;

        let presentation_request: PresentationRequestData =
            self.presentation_request_data.clone()
                .set_format_version_for_did(&connection.agent.pw_did, &connection.data.did_doc.id)?;

        let presentation_request =
            PresentationRequest::V1(
                PresentationRequestV1::create()
                    .set_comment(presentation_request.name.clone())
                    .set_service(connection.service()?)
                    .set_request_presentations_attach(&presentation_request)?
            );

        let thread = Thread::new()
            .set_thid(presentation_request.id())
            .set_opt_pthid(connection.data.thread.pthid.clone());

        connection.data.send_message(&presentation_request, &connection.agent)?;
        Ok(VerifierState::PresentationRequestSent((self, presentation_request, connection, thread).into()))
    }
}

impl PresentationRequestPreparedState {
    fn send_presentation_request(self, connection_handle: Handle<Connections>) -> VcxResult<VerifierState> {
        let connection: CompletedConnection = connection_handle.get_completed_connection()?;

        let presentation_request =
            self.presentation_request.clone()
                .set_service(connection.service()?);

        let thread = Thread::new()
            .set_thid(presentation_request.id().to_string())
            .set_opt_pthid(connection.data.thread.pthid.clone());

        connection.data.send_message(&presentation_request, &connection.agent)?;
        Ok(VerifierState::PresentationRequestSent((self, connection, thread).into()))
    }

    fn handle_received_presentation(self, presentation: Presentation) -> VcxResult<VerifierState> {
        let connection: &CompletedConnection = self.connection.as_ref()
            .ok_or(VcxError::from_msg(
                VcxErrorKind::InvalidState,
                "Invalid Verifier object state: `connection` not found")
            )?;

        let mut thread = presentation.thread().clone()
            .update_received_order(&connection.data.did_doc.id);

        match self.verify_presentation(&presentation, &thread) {
            Ok(thread) => {
                Ok(VerifierState::Finished((self, presentation, thread).into()))
            }
            Err(err) => {
                thread = thread.increment_sender_order();

                let problem_report =
                    ProblemReport::create()
                        .set_message_type(self.presentation_request.type_())
                        .set_description(ProblemReportCodes::InvalidPresentation)
                        .set_comment(format!("error occurred: {:?}", err))
                        .set_thread(thread.clone());

                connection.data.send_message(&problem_report, &connection.agent)?;
                Ok(VerifierState::Finished((self, Status::Failed(problem_report), thread).into()))
            }
        }
    }

    fn handle_received_presentation_proposal(self, proposal: PresentationProposal) -> VcxResult<VerifierState> {
        let connection: &CompletedConnection = self.connection.as_ref()
            .ok_or(VcxError::from_msg(
                VcxErrorKind::InvalidState,
                "Invalid Verifier object state: `connection` not found")
            )?;

        let thread = match self.presentation_request.thread() {
            Some(thread) => thread.clone(),
            None => Thread::new().set_thid(self.presentation_request.id())
        };
        let thread = thread.update_received_order(&connection.data.did_doc.id);
        Ok(VerifierState::PresentationProposalReceived((self, proposal, thread).into()))
    }

    fn handle_received_presentation_reject(self, problem_report: ProblemReport) -> VcxResult<VerifierState> {
        let connection: &CompletedConnection = self.connection.as_ref()
            .ok_or(VcxError::from_msg(
                VcxErrorKind::InvalidState,
                "Invalid Verifier object state: `connection` not found")
            )?;

        let thread = problem_report.thread.clone()
            .update_received_order(&connection.data.did_doc.id);
        Ok(VerifierState::Finished((self, Status::Rejected(Some(problem_report)), thread).into()))
    }

    fn verify_presentation(&self, presentation: &Presentation, thread: &Thread) -> VcxResult<Thread> {
        trace!("PresentationRequestSentState::verify_presentation >>> presentation: {:?}", secret!(presentation));

        let connection = self.connection.as_ref().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidState,
            "Unable to get connection data for Verifier state machine"))?;

        let mut thread = thread.clone();

        let (_, presentations_attach) = presentation.presentations_attach().content()?;
        let (_, request_presentations_attach) = self.presentation_request.request_presentations_attach().content()?;
        let valid = Proof::validate_indy_proof(&presentations_attach, &request_presentations_attach)?;

        if !valid {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidProof, "Presentation verification failed"));
        }

        if presentation.please_ack().is_some() {
            thread = thread.increment_sender_order();

            let ack = PresentationAck::create()
                .set_message_type(presentation.type_())
                .set_thread(thread.clone());

            connection.data.send_message(&ack, &connection.agent)?;
        }

        trace!("PresentationRequestSentState::verify_presentation <<<");
        Ok(thread)
    }
}


impl PresentationRequestSentState {
    fn handle_received_presentation(self, presentation: Presentation) -> VcxResult<VerifierState> {
        let mut thread = self.thread.clone()
            .update_received_order(&self.connection.data.did_doc.id);

        match self.verify_presentation(&presentation, &thread) {
            Ok(thread) => {
                Ok(VerifierState::Finished((self, presentation, thread).into()))
            }
            Err(err) => {
                thread = thread.increment_sender_order();

                let problem_report =
                    ProblemReport::create()
                        .set_message_type(self.presentation_request.type_())
                        .set_description(ProblemReportCodes::InvalidPresentation)
                        .set_comment(format!("error occurred: {:?}", err))
                        .set_thread(thread.clone());

                self.connection.data.send_message(&problem_report, &self.connection.agent)?;
                return Err(err);
            }
        }
    }

    fn verify_presentation(&self, presentation: &Presentation, thread: &Thread) -> VcxResult<Thread> {
        trace!("PresentationRequestSentState::verify_presentation >>> presentation: {:?}", secret!(presentation));
        debug!("verifier verifying received presentation");

        let mut thread = thread.clone();

        let (_, presentations_attach) = presentation.presentations_attach().content()?;
        let (_, request_presentations_attach) = self.presentation_request.request_presentations_attach().content()?;
        let valid = Proof::validate_indy_proof(&presentations_attach, &request_presentations_attach)?;
        if !valid {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidProof, "Presentation verification failed"));
        }

        if presentation.please_ack().is_some() {
            thread = thread.increment_sender_order();

            let ack = PresentationAck::create()
                .set_message_type(self.presentation_request.type_())
                .set_thread(thread.clone());

            self.connection.data.send_message(&ack, &self.connection.agent)?;
        }

        trace!("PresentationRequestSentState::verify_presentation <<<");
        Ok(thread)
    }
}

impl PresentationProposalReceivedState {
    fn send_presentation_request(self, connection_handle: Handle<Connections>,
                                 presentation_request_data: PresentationRequestData) -> VcxResult<VerifierState> {
        let connection = connection_handle.get_completed_connection()?;

        let thread = self.thread.clone()
            .update_received_order(&connection.data.did_doc.id)
            .set_opt_pthid(connection.data.thread.pthid.clone())
            .increment_sender_order();

        let presentation_request: PresentationRequestData =
            presentation_request_data
                .set_format_version_for_did(&connection.agent.pw_did, &connection.data.did_doc.id)?;


        let presentation_request = match &self.presentation_proposal {
            PresentationProposal::V1(_) => {
                PresentationRequest::V1(
                    PresentationRequestV1::create()
                        .set_comment(presentation_request.name.clone())
                        .set_request_presentations_attach(&presentation_request)?
                        .set_thread(thread.clone())
                        .set_service(connection.service()?)
                )
            }
            PresentationProposal::V2(_) => {
                let attach = json!(presentation_request).to_string();
                PresentationRequest::V2(
                    PresentationRequestV2::create()
                        .set_comment(presentation_request.name.clone())
                        .set_indy_request_presentations_attach(&attach)?
                        .set_thread(thread.clone())
                        .set_service(connection.service()?)
                )
            }
        };

        connection.data.send_message(&presentation_request, &connection.agent)?;
        Ok(VerifierState::PresentationRequestSent((self, presentation_request, connection, thread).into()))
    }

    fn send_presentation_request_from_proposal(self, connection_handle: Handle<Connections>) -> VcxResult<VerifierState> {
        let connection: CompletedConnection = connection_handle.get_completed_connection()?;

        let thread = self.thread.clone()
            .update_received_order(&connection.data.did_doc.id)
            .set_opt_pthid(connection.data.thread.pthid.clone())
            .increment_sender_order();

        let presentation_request = match &self.presentation_proposal {
            PresentationProposal::V1(presentation_proposal) => {
                let presentation_request: PresentationRequestData =
                    PresentationRequestData::create()
                        .set_name(presentation_proposal.comment.clone().unwrap_or_default())
                        .set_requested_attributes_value(presentation_proposal.to_proof_request_requested_attributes())
                        .set_requested_predicates_value(presentation_proposal.to_proof_request_requested_predicates())
                        .set_nonce()?
                        .set_format_version_for_did(&connection.agent.pw_did, &connection.data.did_doc.id)?;

                PresentationRequest::V1(
                    PresentationRequestV1::create()
                        .set_comment(presentation_request.name.clone())
                        .set_request_presentations_attach(&presentation_request)?
                        .set_thread(thread.clone())
                        .set_service(connection.service()?)
                )
            }
            PresentationProposal::V2(presentation_proposal) => {
                let presentation_request_data: PresentationRequestData =
                    PresentationRequestData::create()
                        .set_name(presentation_proposal.comment.clone().unwrap_or_default())
//                                        .set_requested_attributes_value(presentation_proposal.to_proof_request_requested_attributes())
//                                        .set_requested_predicates_value(presentation_proposal.to_proof_request_requested_predicates())
                        .set_nonce()?
                        .set_format_version_for_did(&connection.agent.pw_did, &connection.data.did_doc.id)?;

                let presentation_request_json = json!(presentation_request_data).to_string();

                PresentationRequest::V2(
                    PresentationRequestV2::create()
                        .set_comment(presentation_request_data.name.clone())
                        .set_indy_request_presentations_attach(&presentation_request_json)?
                        .set_thread(thread.clone())
                        .set_service(connection.service()?)
                )
            }
        };

        connection.data.send_message(&presentation_request, &connection.agent)?;
        Ok(VerifierState::PresentationRequestSent((self, presentation_request, connection, thread).into()))
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::utils::devsetup::SetupAriesMocks;
    use crate::aries::handlers::connection::tests::mock_connection;
    use crate::aries::test::source_id;
    use crate::aries::messages::proof_presentation::test::{_ack, _problem_report};
    use crate::aries::messages::proof_presentation::presentation_request::tests::_presentation_request;
    use crate::aries::messages::proof_presentation::presentation::tests::_presentation;
    use crate::aries::messages::proof_presentation::presentation_proposal::tests::_presentation_proposal;
    use crate::aries::messages::proof_presentation::v10::presentation_request::tests::_presentation_request_data;
    use crate::aries::messages::proof_presentation::v10::presentation::tests::_presentation as _presentation_v1;
    use crate::aries::messages::proof_presentation::v10::presentation_proposal::tests::_presentation_proposal as _presentation_proposal_v1;

    pub fn _verifier_sm() -> VerifierSM {
        VerifierSM::new(_presentation_request_data(), source_id())
    }

    impl VerifierSM {
        fn to_presentation_request_sent_state(mut self) -> VerifierSM {
            self = self.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
            self
        }

        fn to_finished_state(mut self) -> VerifierSM {
            self = self.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
            self = self.step(VerifierMessages::PresentationReceived(_presentation())).unwrap();
            self
        }
    }

    mod new {
        use super::*;

        #[test]
        fn test_verifier_new() {
            let _setup = SetupAriesMocks::init();

            let verifier_sm = _verifier_sm();

            assert_match!(VerifierState::Initiated(_), verifier_sm.state);
            assert_eq!(source_id(), verifier_sm.source_id());
        }
    }

    mod step {
        use super::*;

        #[test]
        fn test_verifier_init() {
            let _setup = SetupAriesMocks::init();

            let verifier_sm = _verifier_sm();
            assert_match!(VerifierState::Initiated(_), verifier_sm.state);
        }

        #[test]
        fn test_prover_handle_send_presentation_request_message_from_initiated_state() {
            let _setup = SetupAriesMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();

            assert_match!(VerifierState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[test]
        fn test_prover_handle_other_messages_from_initiated_state() {
            let _setup = SetupAriesMocks::init();

            let mut verifier_sm = _verifier_sm();

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(VerifierState::Initiated(_), verifier_sm.state);

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationReceived(_presentation())).unwrap();
            assert_match!(VerifierState::Initiated(_), verifier_sm.state);
        }

        #[test]
        fn test_prover_handle_verify_presentation_message_from_presentation_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationReceived(_presentation())).unwrap();

            assert_match!(VerifierState::Finished(_), verifier_sm.state);
            assert_eq!(VcxStateType::VcxStateAccepted as u32, verifier_sm.state());
        }

//    #[test]
//    fn test_prover_handle_verify_presentation_message_from_presentation_request_sent_state_for_invalid_presentation() {
//        let _setup = Setup::init();
//
//        let mut verifier_sm = _verifier_sm();
//        verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
//        verifier_sm = verifier_sm.step(VerifierMessages::VerifyPresentation(_presentation())).unwrap();
//
//        assert_match!(VerifierState::Finished(_), verifier_sm.state);
//        assert_eq!(Status::Failed(_problem_report()).code(), verifier_sm.presentation_status());
//    }

        #[test]
        fn test_prover_handle_presentation_proposal_message_from_presentation_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal())).unwrap();

            assert_match!(VerifierState::PresentationProposalReceived(_), verifier_sm.state);
            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, verifier_sm.state());
        }

        #[test]
        fn test_prover_handle_presentation_reject_message_from_presentation_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report())).unwrap();

            assert_match!(VerifierState::Finished(_), verifier_sm.state);
            assert_eq!(VcxStateType::VcxStateRejected as u32, verifier_sm.state());
        }

        #[test]
        fn test_prover_handle_other_messages_from_presentation_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();

            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
            assert_match!(VerifierState::PresentationRequestSent(_), verifier_sm.state);
        }

        #[test]
        fn test_prover_handle_messages_from_presentation_finished_state() {
            let _setup = SetupAriesMocks::init();

            let mut verifier_sm = _verifier_sm();
            verifier_sm = verifier_sm.step(VerifierMessages::SendPresentationRequest(mock_connection())).unwrap();
            verifier_sm = verifier_sm.step(VerifierMessages::PresentationReceived(_presentation())).unwrap();

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(VerifierState::Finished(_), verifier_sm.state);

            verifier_sm = verifier_sm.step(VerifierMessages::PresentationProposalReceived(_presentation_proposal())).unwrap();
            assert_match!(VerifierState::Finished(_), verifier_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        fn test_verifier_find_message_to_handle_from_initiated_state() {
            let _setup = SetupAriesMocks::init();

            let verifier = _verifier_sm();

// No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(verifier.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_verifier_find_message_to_handle_from_presentation_request_sent_state() {
            let _setup = SetupAriesMocks::init();

            let verifier = _verifier_sm().to_presentation_request_sent_state();

// Presentation
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack())
                );

                let (uid, message) = verifier.find_message_to_handle(messages).unwrap();
                assert_eq!("key_2", uid);
                assert_match!(A2AMessage::Presentation(_), message);
            }

// Presentation Proposal
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_2".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack())
                );

                let (uid, message) = verifier.find_message_to_handle(messages).unwrap();
                assert_eq!("key_2", uid);
                assert_match!(A2AMessage::PresentationProposal(_), message);
            }

// Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_2".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_3".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = verifier.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

// Presentation Reject
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_2".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_3".to_string() => A2AMessage::PresentationReject(_problem_report())
                );

                let (uid, message) = verifier.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

// No agent for different Thread ID
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(PresentationProposal::V1(_presentation_proposal_v1().set_thread_id(""))),
                    "key_2".to_string() => A2AMessage::Presentation(Presentation::V1(_presentation_v1().set_thread_id(""))),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack().set_thread_id("")),
                    "key_4".to_string() => A2AMessage::CommonProblemReport(_problem_report().set_thread_id(""))
                );

                assert!(verifier.find_message_to_handle(messages).is_none());
            }

// No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationRequest(_presentation_request())
                );

                assert!(verifier.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_verifier_find_message_to_handle_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let verifier = _verifier_sm().to_finished_state();

// No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(verifier.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[test]
        fn test_get_state() {
            let _setup = SetupAriesMocks::init();

            assert_eq!(VcxStateType::VcxStateInitialized as u32, _verifier_sm().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _verifier_sm().to_presentation_request_sent_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _verifier_sm().to_finished_state().state());
        }
    }
}
