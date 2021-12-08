use std::collections::HashMap;

use crate::api::VcxStateType;
use crate::aries::handlers::{
    connection::{
        Connection,
        types::CompletedConnection,
        agent::AgentInfo,
    },
    proof_presentation::prover::{
        messages::ProverMessages,
        states::*,
    },
};
use crate::aries::messages::{
    a2a::A2AMessage,
    ack::Ack,
    error::{ProblemReport, ProblemReportCodes, Reason},
    proof_presentation::{
        presentation_request::PresentationRequest,
        presentation_proposal::PresentationProposal,
        presentation_preview::PresentationPreview,
        presentation::Presentation,
        v10::presentation_proposal::PresentationProposal as PresentationProposalV1,
        v20::presentation_proposal::PresentationProposal as PresentationProposalV2,
        v10::presentation::Presentation as PresentationV1,
        v20::presentation::Presentation as PresentationV2,
    },
    status::Status,
    connection::did_doc::DidDoc,
};
use crate::utils::object_cache::Handle;
use crate::utils::libindy::anoncreds::holder::Holder as IndyHolder;
use crate::connection::Connections;
use crate::error::prelude::*;
use crate::aries::messages::thread::Thread;

/// A state machine that tracks the evolution of states for a Prover during
/// the Present Proof protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProverSM {
    source_id: String,
    state: ProverState,
}

impl ProverSM {
    pub fn new(presentation_request: PresentationRequest, source_id: String) -> ProverSM {
        let thid = presentation_request.thread().and_then(|thread| thread.thid.clone()).unwrap_or(presentation_request.id());
        let thread = Thread::new().set_thid(thid);
        trace!("Thread: {:?}", thread);
        ProverSM {
            source_id,
            state: ProverState::RequestReceived(
                RequestReceivedState {
                    thread,
                    presentation_request,
                    presentation_proposal: None,
                }
            ),
        }
    }

    pub fn new_proposal(presentation_proposal: PresentationProposal, source_id: String) -> ProverSM {
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

        ProverSM {
            source_id,
            state: ProverState::ProposalPrepared(
                ProposalPreparedState {
                    presentation_proposal,
                    thread,
                }
            ),
        }
    }
}


impl ProverSM {
    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("Prover::find_message_to_handle >>> agent: {:?}", secret!(messages));
        debug!("Prover: Finding message to update state");

        for (uid, message) in messages {
            match self.state {
                ProverState::RequestReceived(_) => {
                    match message {
                        A2AMessage::PresentationRequest(_) => {
                            // ignore it here??
                        }
                        message => {
                            warn!("Prover: Unexpected message received in Initiated state: {:?}", message);
                        }
                    }
                }
                ProverState::PresentationPrepared(_) => {
                    // do not process agent
                }
                ProverState::PresentationPreparationFailed(_) => {
                    // do not process agent
                }
                ProverState::PresentationSent(ref state) => {
                    match message {
                        A2AMessage::Ack(ack) | A2AMessage::PresentationAck(ack) => {
                            if ack.from_thread(state.thread.thid.as_deref().unwrap_or_default()) {
                                debug!("Prover: Ack message received");
                                return Some((uid, A2AMessage::PresentationAck(ack)));
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) |
                        A2AMessage::PresentationReject(problem_report) => {
                            if problem_report.from_thread(state.thread.thid.as_deref().unwrap_or_default()) {
                                debug!("Prover: PresentationReject message received");
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        message => {
                            warn!("Prover: Unexpected message received in PresentationSent state: {:?}", message);
                        }
                    }
                }
                ProverState::ProposalPrepared(_) => {
                    // do not process agent
                }
                ProverState::ProposalSent(ref state) => {
                    match message {
                        A2AMessage::PresentationRequest(request) => {
                            return Some((uid, A2AMessage::PresentationRequest(request)));
                        }
                        A2AMessage::CommonProblemReport(problem_report) => {
                            if problem_report.from_thread(state.thread.thid.as_deref().unwrap_or_default()) {
                                debug!("Prover: ProposalReject message received");
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        message => {
                            warn!("Prover: Unexpected message received in PresentationSent state: {:?}", message);
                        }
                    }
                }
                ProverState::Finished(_) => {
                    // do not process agent
                }
            };
        }
        debug!("Prover: no message to update state");
        None
    }

    pub fn step(self, message: ProverMessages) -> VcxResult<ProverSM> {
        trace!("ProverSM::step >>> message: {:?}", secret!(message));
        debug!("Prover: Updating state");

        let ProverSM { source_id, state } = self;

        let state = match state {
            ProverState::RequestReceived(state) => {
                let thread = state.thread.clone();

                match message {
                    ProverMessages::SetPresentation(presentation) => {
                        let presentation = presentation.set_thread(thread.clone());
                        ProverState::PresentationPrepared((state, presentation, thread).into())
                    }
                    ProverMessages::PreparePresentation((credentials, self_attested_attrs)) => {
                        state.prepare_presentation(&credentials, &self_attested_attrs)?
                    }
                    ProverMessages::RejectPresentationRequest((connection_handle, reason)) => {
                        let (problem_report, thread) = _handle_reject_presentation_request(connection_handle,
                                                                                           &reason,
                                                                                           &state.presentation_request,
                                                                                           &thread)?;
                        ProverState::Finished((state, thread, problem_report, Reason::Reject).into())
                    }
                    ProverMessages::ProposePresentation((connection_handle, preview)) => {
                        state.propose_presentation(connection_handle, preview)?
                    }
                    message_ => {
                        warn!("Prover: Unexpected action to update state {:?}", message_);
                        ProverState::RequestReceived(state)
                    }
                }
            }
            ProverState::PresentationPrepared(state) => {
                match message {
                    ProverMessages::SendPresentation(connection_handle) => {
                        state.send_presentation(connection_handle)?
                    }
                    ProverMessages::RejectPresentationRequest((connection_handle, reason)) => {
                        let (problem_report, thread) = _handle_reject_presentation_request(connection_handle,
                                                                                           &reason,
                                                                                           &state.presentation_request,
                                                                                           &state.thread)?;
                        ProverState::Finished((state, thread, problem_report, Reason::Reject).into())
                    }
                    ProverMessages::ProposePresentation((connection_handle, preview)) => {
                        state.propose_presentation(connection_handle, preview)?
                    }
                    message_ => {
                        warn!("Prover: Unexpected action to update state {:?}", message_);
                        ProverState::PresentationPrepared(state)
                    }
                }
            }
            ProverState::PresentationPreparationFailed(state) => {
                match message {
                    ProverMessages::SendPresentation(connection_handle) => {
                        state.send_presentation_reject(connection_handle)?
                    }
                    message_ => {
                        warn!("Prover: Unexpected action to update state {:?}", message_);
                        ProverState::PresentationPreparationFailed(state)
                    }
                }
            }
            ProverState::PresentationSent(state) => {
                match message {
                    ProverMessages::PresentationAckReceived(ack) => {
                        state.handle_ack(ack)?
                    }
                    ProverMessages::PresentationRejectReceived(problem_report) => {
                        let thread = state.thread.clone()
                            .update_received_order(&state.connection.data.did_doc.id);
                        ProverState::Finished((state, problem_report, thread, Reason::Fail).into())
                    }
                    ProverMessages::RejectPresentationRequest(_) => {
                        return Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Presentation is already sent"));
                    }
                    message_ => {
                        warn!("Prover: Unexpected action to update state {:?}", message_);
                        ProverState::PresentationSent(state)
                    }
                }
            }
            ProverState::Finished(state) => ProverState::Finished(state),
            ProverState::ProposalPrepared(state) => {
                match message {
                    ProverMessages::SendProposal(connection_handle) => {
                        state.send_presentation_proposal(connection_handle)?
                    }
                    message_ => {
                        warn!("Prover: Unexpected action to update state {:?}", message_);
                        ProverState::ProposalPrepared(state)
                    }
                }
            }
            ProverState::ProposalSent(state) => {
                match message {
                    ProverMessages::PresentationRequestReceived(presentation_request) => {
                        let thread = state.thread.clone();
                        // we do not update received order here, because it will be updated before sending the response to this message.
                        // It needs to be that way, because there we cannot be sure if that was the first message on that connection.

                        ProverState::RequestReceived((state, presentation_request, thread).into())
                    }
                    ProverMessages::PresentationRejectReceived(problem_report) => {
                        let thread = state.thread.clone()
                            .update_received_order(&state.connection.data.did_doc.id);
                        ProverState::Finished((state, problem_report, thread, Reason::Fail).into())
                    }
                    message_ => {
                        warn!("Prover: Unexpected action to update state {:?}", message_);
                        ProverState::ProposalSent(state)
                    }
                }
            }
        };

        trace!("Prover::step <<< state: {:?}", secret!(state));
        Ok(ProverSM { source_id, state })
    }

    pub fn source_id(&self) -> &String { &self.source_id }

    pub fn state(&self) -> u32 {
        match self.state {
            ProverState::RequestReceived(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationPrepared(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationPreparationFailed(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationSent(_) => VcxStateType::VcxStateOfferSent as u32,
            ProverState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    Status::Rejected(_) => VcxStateType::VcxStateRejected as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
            ProverState::ProposalPrepared(_) => VcxStateType::VcxStateInitialized as u32,
            ProverState::ProposalSent(_) => VcxStateType::VcxStateOfferSent as u32,
        }
    }

    pub fn has_transitions(&self) -> bool {
        match self.state {
            ProverState::RequestReceived(_) => false,
            ProverState::PresentationPrepared(_) => true,
            ProverState::PresentationPreparationFailed(_) => true,
            ProverState::PresentationSent(_) => true,
            ProverState::Finished(_) => false,
            ProverState::ProposalPrepared(_) => false,
            ProverState::ProposalSent(_) => true
        }
    }

    pub fn get_agent_info(&self) -> Option<&AgentInfo> {
        match self.state {
            ProverState::RequestReceived(_) => None,
            ProverState::PresentationPrepared(_) => None,
            ProverState::PresentationPreparationFailed(_) => None,
            ProverState::PresentationSent(ref state) => Some(&state.connection.agent),
            ProverState::Finished(_) => None,
            ProverState::ProposalPrepared(_) => None,
            ProverState::ProposalSent(ref state) => Some(&state.connection.agent)
        }
    }

    pub fn presentation_request(&self) -> VcxResult<&PresentationRequest> {
        match self.state {
            ProverState::RequestReceived(ref state) => Ok(&state.presentation_request),
            ProverState::PresentationPrepared(ref state) => Ok(&state.presentation_request),
            ProverState::PresentationPreparationFailed(ref state) => Ok(&state.presentation_request),
            ProverState::PresentationSent(ref state) => Ok(&state.presentation_request),
            ProverState::Finished(ref state) => {
                state.presentation_request.as_ref()
                    .ok_or(VcxError::from_msg(VcxErrorKind::NotReady,
                                              format!("Prover object {} in state {} not ready to get Presentation Request message", self.source_id, self.state())))
            }
            ProverState::ProposalPrepared(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                       format!("Prover object {} in state {} not ready to get Presentation Request message", self.source_id, self.state()))),
            ProverState::ProposalSent(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                   format!("Prover object {} in state {} not ready to get Presentation Request message", self.source_id, self.state()))),
        }
    }

    pub fn presentation(&self) -> VcxResult<&Presentation> {
        match self.state {
            ProverState::RequestReceived(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                      format!("Prover object {} in state {} not ready to get Presentation message", self.source_id, self.state()))),
            ProverState::PresentationPrepared(ref state) => Ok(&state.presentation),
            ProverState::PresentationPreparationFailed(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation preparation failed")),
            ProverState::PresentationSent(ref state) => Ok(&state.presentation),
            ProverState::Finished(ref state) => {
                state.presentation.as_ref()
                    .ok_or(VcxError::from_msg(VcxErrorKind::NotReady,
                                              format!("Prover object {} in state {} not ready to get Presentation message", self.source_id, self.state())))
            }
            ProverState::ProposalPrepared(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                       format!("Prover object {} in state {} not ready to get Presentation message", self.source_id, self.state()))),
            ProverState::ProposalSent(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                                                   format!("Prover object {} in state {} not ready to get Presentation message", self.source_id, self.state()))),
        }
    }

    pub fn problem_report(&self) -> Option<&ProblemReport> {
        match self.state {
            ProverState::RequestReceived(_) |
            ProverState::PresentationPrepared(_) |
            ProverState::PresentationSent(_) |
            ProverState::ProposalPrepared(_) |
            ProverState::ProposalSent(_) => None,
            ProverState::PresentationPreparationFailed(ref state) => Some(&state.problem_report),
            ProverState::Finished(ref status) => {
                match &status.status {
                    Status::Success | Status::Undefined => None,
                    Status::Rejected(ref problem_report) => problem_report.as_ref(),
                    Status::Failed(problem_report) => Some(problem_report),
                }
            }
        }
    }
}


impl RequestReceivedState {
    fn propose_presentation(self, connection_handle: Handle<Connections>, preview: PresentationPreview) -> VcxResult<ProverState> {
        let connection: CompletedConnection = connection_handle.get_completed_connection()?;
        let thread = self.thread.clone()
            .update_received_order(&connection.data.did_doc.id);
        let presentation_proposal = _handle_presentation_proposal(&connection, preview, &self.presentation_request, &thread)?;
        Ok(ProverState::ProposalSent((self, connection, presentation_proposal, thread).into()))
    }

    fn prepare_presentation(self, credentials: &str, self_attested_attrs: &str) -> VcxResult<ProverState> {
        let thread = self.thread.clone();
        match self.build_presentation(&credentials, &self_attested_attrs) {
            Ok(presentation) => {
                Ok(ProverState::PresentationPrepared((self, presentation, thread).into()))
            }
            Err(err) => {
                let problem_report =
                    ProblemReport::create()
                        .set_message_type(self.presentation_request.type_())
                        .set_description(ProblemReportCodes::InvalidPresentationRequest)
                        .set_comment(err.to_string())
                        .set_thread(thread.clone());
                Ok(ProverState::PresentationPreparationFailed((self, problem_report, err.kind(), thread).into()))
            }
        }
    }

    fn build_presentation(&self, credentials: &str, self_attested_attrs: &str) -> VcxResult<Presentation> {
        let thread = self.thread.clone();
        let (_, attachment) = self.presentation_request.request_presentations_attach().content()?;
        let indy_proof = IndyHolder::generate_proof(credentials, self_attested_attrs, &attachment)?;
        let presentation = match self.presentation_request {
            PresentationRequest::V1(ref presentation_request) => {
                Presentation::V1(
                    PresentationV1::create()
                        .set_comment(presentation_request.comment.clone())
                        .ask_for_ack()
                        .set_thread(thread)
                        .set_presentations_attach(indy_proof)?
                )
            }
            PresentationRequest::V2(ref presentation_request) => {
                Presentation::V2(
                    PresentationV2::create()
                        .set_comment(presentation_request.comment.clone())
                        .ask_for_ack()
                        .set_thread(thread)
                        .set_indy_presentations_attach(&indy_proof)?
                )
            }
        };
        Ok(presentation)
    }
}

impl PresentationPreparedState {
    fn send_presentation(self, connection_handle: Handle<Connections>) -> VcxResult<ProverState> {
        let thread = self.thread.clone();
        if self.presentation_request.service().is_some() && connection_handle == 0 {
            // ephemeral proof request
            let did_doc: DidDoc = self.presentation_request.service().clone().unwrap().into();
            let presentation = self.presentation.clone().reset_ack();
            Connection::send_message_to_self_endpoint(&presentation, &did_doc)?;
            Ok(ProverState::Finished((self, thread).into()))
        } else {
            // regular proof request
            let connection = connection_handle.get_completed_connection()?;

            let thread = thread
                .update_received_order(&connection.data.did_doc.id)
                .set_opt_pthid(connection.data.thread.pthid.clone());

            let presentation = self.presentation.clone()
                .set_thread(thread.clone());

            connection.data.send_message(&presentation, &connection.agent)?;
            Ok(ProverState::PresentationSent((self, connection, presentation, thread).into()))
        }
    }

    fn propose_presentation(self, connection_handle: Handle<Connections>, preview: PresentationPreview) -> VcxResult<ProverState> {
        let connection = connection_handle.get_completed_connection()?;
        let thread = self.thread.clone()
            .update_received_order(&connection.data.did_doc.id);
        let presentation_proposal = _handle_presentation_proposal(&connection, preview, &self.presentation_request, &thread)?;
        Ok(ProverState::Finished((self, thread, presentation_proposal, Reason::Reject).into()))
    }
}

impl PresentationSentState {
    fn handle_ack(self, ack: Ack) -> VcxResult<ProverState> {
        trace!("PresentationSentState::handle_ack >>> ack: {:?}", secret!(ack));
        debug!("prover handling received presentation ack message");

        let mut thread = self.thread.clone()
            .update_received_order(&self.connection.data.did_doc.id);

        match self.thread.check_message_order(&self.connection.data.did_doc.id, &ack.thread) {
            Ok(()) => {
                Ok(ProverState::Finished((self, ack, thread).into()))
            }
            Err(err) => {
                thread = thread.increment_sender_order();

                let problem_report = ProblemReport::create()
                    .set_message_type(self.presentation_request.type_())
                    .set_description(ProblemReportCodes::Other(String::from("invalid-message-state")))
                    .set_comment(format!("error occurred: {:?}", err))
                    .set_thread(thread.clone());

                self.connection.data.send_message(&problem_report, &self.connection.agent)?;
                return Err(err);
            }
        }
    }
}

impl PresentationPreparationFailedState {
    fn send_presentation_reject(self, connection_handle: Handle<Connections>) -> VcxResult<ProverState> {
        let connection = connection_handle.get_completed_connection()?;
        let thread = self.thread.clone()
            .update_received_order(&connection.data.did_doc.id);
        let error_kind = self.error_kind
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Invalid Prover object state: `error_kind` not found"))?;

        let problem_report =
            self.problem_report.clone()
                .set_thread(thread);

        match self.presentation_request.service() {
            None => {
                connection.data.send_message(&problem_report, &connection.agent)?;
            }
            Some(service) => {
                let did_doc = service.clone().into();
                Connection::send_message_to_self_endpoint(&problem_report, &did_doc)?;
            }
        }
        return Err(VcxError::from_msg(error_kind, self.problem_report.comment.unwrap_or_default()));
    }
}

impl ProposalPreparedState {
    fn send_presentation_proposal(self, connection_handle: Handle<Connections>) -> VcxResult<ProverState> {
        let connection = connection_handle.get_completed_connection()?;
        let thread = self.thread.clone()
            .update_received_order(&connection.data.did_doc.id);
        let presentation_proposal = self.presentation_proposal.clone();
        connection.data.send_message(&presentation_proposal, &connection.agent)?;
        Ok(ProverState::ProposalSent((self, connection, presentation_proposal, thread).into()))
    }
}

fn _handle_reject_presentation_request(connection_handle: Handle<Connections>, reason: &str, presentation_request: &PresentationRequest, thread: &Thread) -> VcxResult<(ProblemReport, Thread)> {
    trace!("ProverSM::_handle_reject_presentation_request >>> reason: {:?}, presentation_request: {:?}", secret!(reason), secret!(presentation_request));
    debug!("Prover: Rejecting presentation request");

    let mut thread = thread.clone();
    let mut problem_report = ProblemReport::create()
        .set_message_type(presentation_request.type_())
        .set_description(ProblemReportCodes::PresentationRejected)
        .set_comment(reason.to_string())
        .set_thread(thread.clone());

    if presentation_request.service().is_some() && connection_handle == 0 {
        // ephemeral proof request
        let did_doc: DidDoc = presentation_request.service().unwrap().into();
        Connection::send_message_to_self_endpoint(&problem_report, &did_doc)?;
    } else {
        // regular proof request
        let connection = connection_handle.get_completed_connection()?;
        // we need to update thread and put it into problem report
        thread = thread.update_received_order(&connection.data.did_doc.id);
        problem_report = problem_report.set_thread(thread.clone());

        connection.data.send_message(&problem_report, &connection.agent)?;
    }

    trace!("ProverSM::_handle_reject_presentation_request <<<");
    Ok((problem_report, thread))
}

fn _handle_presentation_proposal(connection: &CompletedConnection, preview: PresentationPreview, presentation_request: &PresentationRequest, thread: &Thread) -> VcxResult<PresentationProposal> {
    trace!("ProverSM::_handle_presentation_proposal >>> preview: {:?}, presentation_request: {:?}", secret!(preview), secret!(presentation_request));
    debug!("Prover: Preparing presentation proposal");

    let proposal = match presentation_request {
        PresentationRequest::V1(_) => {
            PresentationProposal::V1(
                PresentationProposalV1::create()
                    .set_presentation_preview(preview)
                    .set_thread(thread.clone())
            )
        }
        PresentationRequest::V2(_) => {
            PresentationProposal::V2(
                PresentationProposalV2::create()
                    .set_thread(thread.clone())
            )
        }
    };

    match presentation_request.service().clone() {
        None => connection.data.send_message(&proposal, &connection.agent)?,
        Some(service) => Connection::send_message_to_self_endpoint(&proposal, &service.into())?
    }

    trace!("ProverSM::_handle_presentation_proposal <<<");
    Ok(proposal)
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
    use crate::aries::messages::proof_presentation::presentation_preview::tests::_presentation_preview;
    use crate::aries::messages::proof_presentation::v10::presentation_request::tests::_presentation_request_with_service;
    use crate::aries::messages::proof_presentation::v10::presentation_request::tests::_presentation_request as _presentation_request_v1;
    use crate::aries::messages::proof_presentation::v10::presentation::tests::_presentation as _presentation_v1;
    use crate::aries::messages::proof_presentation::v10::presentation_proposal::tests::_presentation_proposal as _presentation_proposal_v1;

    pub fn _prover_sm() -> ProverSM {
        ProverSM::new(_presentation_request(), source_id())
    }

    pub fn _prover_sm_proposal() -> ProverSM {
        ProverSM::new_proposal(_presentation_proposal(), source_id())
    }

    impl ProverSM {
        fn to_presentation_prepared_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            self
        }

        fn to_presentation_sent_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            self = self.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            self
        }

        fn to_finished_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            self = self.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            self = self.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            self
        }

        fn to_proposal_sent_state(mut self) -> ProverSM {
            // assert_eq!(self.state )
            self = self.step(ProverMessages::SendProposal(mock_connection())).unwrap();
            self
        }
    }

    fn _credentials() -> String {
        json!({
            "attrs":{
            "attribute_0":{
                "credential":{
                    "cred_info":{
                        "attrs":{"name": "alice"},
                        "cred_def_id": "V4SGRU86Z58d6TV7PBUe6f:3:CL:419:tag",
                        "referent": "a1991de8-8317-43fd-98b3-63bac40b9e8b",
                        "schema_id": "V4SGRU86Z58d6TV7PBUe6f:2:QcimrRShWQniqlHUtIDddYP0n:1.0"
                        }
                    }
                }
            }
        }).to_string()
    }

    fn _self_attested() -> String {
        json!({}).to_string()
    }

    mod new {
        use super::*;

        #[test]
        fn test_prover_new() {
            let _setup = SetupAriesMocks::init();

            let prover_sm = _prover_sm();

            assert_match!(ProverState::RequestReceived(_), prover_sm.state);
            assert_eq!(source_id(), prover_sm.source_id().to_string());
        }

        #[test]
        fn test_prover_new_proposal() {
            let _setup = SetupAriesMocks::init();

            let prover_sm = _prover_sm_proposal();

            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);
            assert_eq!(source_id(), prover_sm.source_id().to_string());
        }
    }

    mod step {
        use super::*;

        #[test]
        fn test_prover_init() {
            let _setup = SetupAriesMocks::init();

            let prover_sm = _prover_sm();
            assert_match!(ProverState::RequestReceived(_), prover_sm.state);
        }

        #[test]
        fn test_prover_init_proposal() {
            let _setup = SetupAriesMocks::init();

            let prover_sm = _prover_sm_proposal();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_prepare_presentation_message_from_request_received() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();

            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_prepare_presentation_message_from_request_received_for_proof_request_with_thread() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let thread_id = "71fa23b0-427e-4064-bf24-b375b1a2c64b";
            let presentation_request = PresentationRequest::V1(_presentation_request_v1().set_thread_id(thread_id));

            let mut prover_sm = ProverSM::new(presentation_request, source_id());
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();

            match prover_sm.state {
                ProverState::PresentationPrepared(state) => {
                    assert_eq!(thread_id, state.thread.thid.unwrap());
                    assert_eq!(0, state.thread.sender_order);
                    Ok(())
                }
                other => Err(format!("State expected to be PresentationPrepared, but: {:?}", other))
            }
        }

        #[test]
        fn test_prover_handle_prepare_presentation_message_from_request_received_for_invalid_credentials() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested()))).unwrap();

            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_reject_presentation_request_message_from_request_received() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest((mock_connection(), String::from("reject request")))).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            match prover_sm.state {
                ProverState::Finished(state) => {
                    assert_eq!(3, state.status.code());
                    Ok(())
                }
                other => Err(format!("State expected to be Finished, but: {:?}", other))
            }
        }

        #[test]
        fn test_prover_handle_propose_presentation_message_from_request_received() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::ProposePresentation((mock_connection(), _presentation_preview()))).unwrap();

            assert_match!(ProverState::ProposalSent(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_other_messages_from_request_received() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();

            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            assert_match!(ProverState::RequestReceived(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::RequestReceived(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_send_presentation_message_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();

            assert_match!(ProverState::PresentationSent(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_send_presentation_message_from_presentation_prepared_state_for_presentation_request_contains_service_decorator() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = ProverSM::new(PresentationRequest::V1(_presentation_request_with_service()), source_id());
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(Handle::dummy())).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_other_messages_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm().to_presentation_prepared_state();

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_reject_presentation_request_message_from_presentation_prepared_state() -> Result<(), String> {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm().to_presentation_prepared_state();
            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest((mock_connection(), String::from("reject request")))).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            match prover_sm.state {
                ProverState::Finished(state) => {
                    assert_eq!(3, state.status.code());
                    Ok(())
                }
                other => Err(format!("State expected to be Finished, but: {:?}", other))
            }
        }

        #[test]
        fn test_prover_handle_propose_presentation_message_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm().to_presentation_prepared_state();
            prover_sm = prover_sm.step(ProverMessages::ProposePresentation((mock_connection(), _presentation_preview()))).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_send_presentation_message_from_presentation_preparation_failed_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested()))).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);

            prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap_err();
        }

        #[test]
        fn test_prover_handle_other_messages_from_presentation_preparation_failed_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested()))).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_send_proposal_message_from_proposal_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm_proposal();
            prover_sm = prover_sm.step(ProverMessages::SendProposal(mock_connection())).unwrap();

            assert_match!(ProverState::ProposalSent(_), prover_sm.state);
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, prover_sm.state());
        }

        #[test]
        fn test_prover_handle_other_messages_from_proposal_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm_proposal();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationRequestReceived(_presentation_request())).unwrap();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest((mock_connection(), "reason".to_string()))).unwrap();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SetPresentation(_presentation())).unwrap();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::ProposePresentation((mock_connection(), _presentation_preview()))).unwrap();
            assert_match!(ProverState::ProposalPrepared(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_ack_message_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(VcxStateType::VcxStateAccepted as u32, prover_sm.state());
        }

        #[test]
        fn test_prover_handle_reject_presentation_request_message_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let prover_sm = _prover_sm().to_presentation_sent_state();
            let err = prover_sm.step(ProverMessages::RejectPresentationRequest((mock_connection(), String::from("reject")))).unwrap_err();
            assert_eq!(VcxErrorKind::InvalidState, err.kind());
        }

        #[test]
        fn test_prover_handle_presentation_reject_message_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(VcxStateType::VcxStateNone as u32, prover_sm.state());
        }

        #[test]
        fn test_prover_handle_other_messages_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            assert_match!(ProverState::PresentationSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            assert_match!(ProverState::PresentationSent(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_presentation_request_received_message_from_proposal_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm_proposal();
            prover_sm = prover_sm.step(ProverMessages::SendProposal(mock_connection())).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationRequestReceived(_presentation_request())).unwrap();

            assert_match!(ProverState::RequestReceived(_), prover_sm.state);
            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, prover_sm.state());
        }

        #[test]
        fn test_prover_handle_presentation_reject_received_message_from_proposal_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm_proposal();
            prover_sm = prover_sm.step(ProverMessages::SendProposal(mock_connection())).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(VcxStateType::VcxStateNone as u32, prover_sm.state());
        }

        #[test]
        fn test_prover_handle_other_messages_from_proposal_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm_proposal();
            prover_sm = prover_sm.step(ProverMessages::SendProposal(mock_connection())).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest((mock_connection(), "reason".to_string()))).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SetPresentation(_presentation())).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendProposal(mock_connection())).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::ProposePresentation((mock_connection(), _presentation_preview()))).unwrap();
            assert_match!(ProverState::ProposalSent(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_messages_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::Finished(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(ProverState::Finished(_), prover_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        fn test_prover_find_message_to_handle_from_request_received() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm();

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_prover_find_message_to_handle_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm().to_presentation_prepared_state();

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_prover_find_message_to_handle_from_proposal_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm_proposal();

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_prover_find_message_to_handle_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm().to_presentation_sent_state();

            // Ack
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack())
                );

                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::PresentationAck(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_3".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

            // Presentation Reject
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_3".to_string() => A2AMessage::PresentationReject(_problem_report())
                );

                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
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

                assert!(prover.find_message_to_handle(messages).is_none());
            }

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationRequest(_presentation_request())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_prover_find_message_to_handle_from_proposal_sent_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm_proposal().to_proposal_sent_state();

            // Presentation Request
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_3".to_string() => A2AMessage::Presentation(_presentation())
                );
                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
                assert_eq!("key_2", uid);
                assert_match!(A2AMessage::PresentationRequest(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
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

                assert!(prover.find_message_to_handle(messages).is_none());
            }

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationAck(_ack().set_thread_id(""))
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_prover_find_message_to_handle_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm().to_finished_state();

            // No agent
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[test]
        fn test_get_state() {
            let _setup = SetupAriesMocks::init();

            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _prover_sm().state());
            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _prover_sm().to_presentation_prepared_state().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _prover_sm().to_presentation_sent_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _prover_sm().to_finished_state().state());
        }
    }
}
