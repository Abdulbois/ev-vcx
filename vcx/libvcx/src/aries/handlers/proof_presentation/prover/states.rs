use crate::aries::messages::{
    proof_presentation::{
        presentation_request::PresentationRequest,
        presentation_proposal::PresentationProposal,
        presentation::Presentation
    },
    error::{ProblemReport, Reason},
    status::Status,
    ack::Ack
};
use crate::aries::handlers::connection::types::CompletedConnection;
use crate::error::prelude::*;
use crate::aries::messages::thread::Thread;

// Possible Transitions:
//
// RequestReceived -> PresentationPrepared, PresentationPreparationFailedState, ProposalSent, Finished
// PresentationPrepared -> PresentationSent, Finished
// PresentationPreparationFailedState -> Finished
// PresentationSent -> Finished
// ProposalPrepared -> ProposalSent
// ProposalSent -> RequestReceived, Finished
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProverState {
    RequestReceived(RequestReceivedState),
    PresentationPrepared(PresentationPreparedState),
    PresentationPreparationFailed(PresentationPreparationFailedState),
    ProposalPrepared(ProposalPreparedState),
    PresentationSent(PresentationSentState),
    ProposalSent(ProposalSentState),
    Finished(FinishedState),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestReceivedState {
    pub presentation_request: PresentationRequest,
    pub presentation_proposal: Option<PresentationProposal>,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparedState {
    pub presentation_request: PresentationRequest,
    pub presentation: Presentation,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparationFailedState {
    pub presentation_request: PresentationRequest,
    pub problem_report: ProblemReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<VcxErrorKind>,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationSentState {
    pub presentation_request: PresentationRequest,
    pub presentation: Presentation,
    pub connection: CompletedConnection,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProposalPreparedState {
    pub presentation_proposal: PresentationProposal,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProposalSentState {
    pub presentation_proposal: PresentationProposal,
    pub presentation_request: Option<PresentationRequest>,
    pub connection: CompletedConnection,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: Option<PresentationRequest>,
    pub presentation: Option<Presentation>,
    pub status: Status,
    #[serde(default)]
    pub thread: Thread,
}

impl From<(ProposalSentState, PresentationRequest, Thread)> for RequestReceivedState {
    fn from((state, presentation_request, thread): (ProposalSentState, PresentationRequest, Thread)) -> Self {
        trace!("ProverSM transit state from ProposalSentState to RequestReceivedState");
        trace!("Thread: {:?}", thread);
        RequestReceivedState {
            presentation_request,
            presentation_proposal: Some(state.presentation_proposal),
            thread,
        }
    }
}

impl From<(ProposalSentState, ProblemReport, Thread, Reason)> for FinishedState {
    fn from((state, problem_report, thread, reason): (ProposalSentState, ProblemReport, Thread, Reason)) -> Self {
        trace!("ProverSM transit state from ProposalSentState to FinishedState with ProblemReport: {:?}", problem_report);
        trace!("Thread: {:?}", problem_report.thread);
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: None,
            status: reason.to_status(problem_report),
            thread,
        }
    }
}

impl From<(RequestReceivedState, Presentation, Thread)> for PresentationPreparedState {
    fn from((state, presentation, thread): (RequestReceivedState, Presentation, Thread)) -> Self {
        trace!("ProverSM transit state from RequestReceivedState to PresentationPreparedState");
        trace!("Thread: {:?}", thread);
        PresentationPreparedState {
            presentation_request: state.presentation_request,
            thread,
            presentation,
        }
    }
}

impl From<(RequestReceivedState, ProblemReport, VcxErrorKind, Thread)> for PresentationPreparationFailedState {
    fn from((state, problem_report, error_kind, thread): (RequestReceivedState, ProblemReport, VcxErrorKind, Thread)) -> Self {
        trace!("ProverSM transit state from RequestReceivedState to PresentationPreparationFailedState with ProblemReport: {:?}", problem_report);
        trace!("Thread: {:?}", thread);
        PresentationPreparationFailedState {
            presentation_request: state.presentation_request,
            thread,
            problem_report,
            error_kind: Some(error_kind),
        }
    }
}

impl From<(RequestReceivedState, CompletedConnection, PresentationProposal, Thread)> for ProposalSentState {
    fn from((state, connection, presentation_proposal, thread): (RequestReceivedState, CompletedConnection, PresentationProposal, Thread)) -> Self {
        trace!("ProverSM transit state from RequestReceivedState to ProposalSentState");
        trace!("Thread: {:?}", thread);
        ProposalSentState {
            presentation_proposal,
            presentation_request: Some(state.presentation_request),
            connection,
            thread,
        }
    }
}

impl From<(ProposalPreparedState, CompletedConnection, PresentationProposal, Thread)> for ProposalSentState {
    fn from((_state, connection, presentation_proposal, thread): (ProposalPreparedState, CompletedConnection, PresentationProposal, Thread)) -> Self {
        trace!("ProverSM transit state from ProposalPreparedState to ProposalSentState");
        trace!("Thread: {:?}", thread);
        ProposalSentState {
            presentation_proposal,
            presentation_request: None,
            connection,
            thread,
        }
    }
}

impl From<(RequestReceivedState, Thread, ProblemReport, Reason)> for FinishedState {
    fn from((state, thread, problem_report, reason): (RequestReceivedState, Thread, ProblemReport, Reason)) -> Self {
        trace!("ProverSM transit state from RequestReceivedState to FinishedState with DeclineProof message");
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: reason.to_status(problem_report),
            thread,
        }
    }
}

impl From<(PresentationPreparedState, CompletedConnection, Presentation, Thread)> for PresentationSentState {
    fn from((state, connection, presentation, thread): (PresentationPreparedState, CompletedConnection, Presentation, Thread)) -> Self {
        trace!("ProverSM transit state from PresentationPreparedState to PresentationSentState");
        trace!("Thread: {:?}", thread);
        PresentationSentState {
            presentation_request: state.presentation_request,
            presentation,
            connection,
            thread,
        }
    }
}

impl From<(PresentationPreparedState, Thread)> for FinishedState {
    fn from((state, thread): (PresentationPreparedState, Thread)) -> Self {
        trace!("ProverSM transit state from PresentationPreparedState to FinishedState");
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: Some(state.presentation),
            status: Status::Success,
            thread,
        }
    }
}

impl From<(PresentationPreparedState, Thread, ProblemReport, Reason)> for FinishedState {
    fn from((state, thread, problem_report, reason): (PresentationPreparedState, Thread, ProblemReport, Reason)) -> Self {
        trace!("ProverSM transit state from PresentationPreparedState to FinishedState with DeclineProof message");
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: reason.to_status(problem_report),
            thread,
        }
    }
}

impl From<(PresentationPreparedState, Thread, PresentationProposal, Reason)> for FinishedState {
    fn from((state, thread, _presentation_proposal, _reason): (PresentationPreparedState, Thread, PresentationProposal, Reason)) -> Self {
        trace!("ProverSM transit state from PresentationPreparedState to FinishedState with DeclineProof message");
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Rejected(None),
            thread,
        }
    }
}

impl From<(PresentationPreparationFailedState, Thread)> for FinishedState {
    fn from((state, thread): (PresentationPreparationFailedState, Thread)) -> Self {
        trace!("ProverSM transit state from PresentationPreparationFailedState to FinishedState with ProblemReport: {:?}", state.problem_report);
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: None,
            status: Status::Failed(state.problem_report),
            thread,
        }
    }
}

impl From<(PresentationSentState, Ack, Thread)> for FinishedState {
    fn from((state, _ack, thread): (PresentationSentState, Ack, Thread)) -> Self {
        trace!("ProverSM transit state from PresentationSentState to FinishedState");
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: Some(state.presentation),
            status: Status::Success,
            thread,
        }
    }
}

impl From<(PresentationSentState, ProblemReport, Thread, Reason)> for FinishedState {
    fn from((state, problem_report, thread, reason): (PresentationSentState, ProblemReport, Thread, Reason)) -> Self {
        trace!("ProverSM transit state from PresentationSentState to FinishedState with ProblemReport: {:?}", problem_report);
        trace!("Thread: {:?}", problem_report.thread);
        FinishedState {
            presentation_request: Some(state.presentation_request),
            presentation: Some(state.presentation),
            status: reason.to_status(problem_report),
            thread,
        }
    }
}