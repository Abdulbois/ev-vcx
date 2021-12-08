use crate::aries::messages::{
    proof_presentation::{
        presentation::Presentation,
        presentation_proposal::PresentationProposal,
        presentation_request::PresentationRequest,
    },
    status::Status
};
use crate::aries::handlers::connection::types::CompletedConnection;
use crate::aries::messages::thread::Thread;
use crate::utils::libindy::anoncreds::proof_request::ProofRequest;

// Possible Transitions:
//
// Initial -> PresentationRequestSent
// PresentationRequestSent -> PresentationProposalReceived, Finished
// PresentationProposalReceived -> PresentationRequestSent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerifierState {
    Initiated(InitialState),
    PresentationRequestPrepared(PresentationRequestPreparedState),
    PresentationRequestSent(PresentationRequestSentState),
    PresentationProposalReceived(PresentationProposalReceivedState),
    Finished(FinishedState),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    pub presentation_request_data: ProofRequest
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestPreparedState {
    pub presentation_request: PresentationRequest,
    #[serde(default)]
    pub connection: Option<CompletedConnection>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationRequestSentState {
    pub presentation_request: PresentationRequest,
    pub connection: CompletedConnection,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationProposalReceivedState {
    pub presentation_proposal: PresentationProposal,
    pub connection: Option<CompletedConnection>,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    pub presentation_request: PresentationRequest,
    pub presentation: Option<Presentation>,
    pub status: Status,
    #[serde(default)]
    pub thread: Thread,
}

impl From<(InitialState, PresentationRequest, CompletedConnection, Thread)> for PresentationRequestSentState {
    fn from((_state, presentation_request, connection, thread): (InitialState, PresentationRequest, CompletedConnection, Thread)) -> Self {
        trace!("VerifierSM transit state from InitialState to PresentationRequestSentState");
        trace!("Thread: {:?}", thread);
        PresentationRequestSentState {
            connection,
            presentation_request,
            thread,
        }
    }
}

impl From<(InitialState, PresentationRequest)> for PresentationRequestPreparedState {
    fn from((_state, presentation_request): (InitialState, PresentationRequest)) -> Self {
        trace!("VerifierSM transit state from InitialState to PresentationRequestPreparedState");
        PresentationRequestPreparedState {
            presentation_request,
            connection: None,
        }
    }
}

impl From<(PresentationRequestPreparedState, CompletedConnection, Thread)> for PresentationRequestSentState {
    fn from((state, connection, thread): (PresentationRequestPreparedState, CompletedConnection, Thread)) -> Self {
        trace!("PresentationRequestPreparedState: transit state from InitialState to PresentationRequestSentState");
        trace!("Thread: {:?}", thread);
        PresentationRequestSentState {
            connection,
            presentation_request: state.presentation_request,
            thread,
        }
    }
}

impl From<(PresentationRequestPreparedState, PresentationProposal, Thread)> for PresentationProposalReceivedState {
    fn from((state, proposal, thread): (PresentationRequestPreparedState, PresentationProposal, Thread)) -> Self {
        trace!("VerifireSM transit state from PresentationRequestPreparedState to PresentationProposalReceivedState");
        trace!("Thread: {:?}", thread);
        PresentationProposalReceivedState {
            presentation_proposal: proposal,
            connection: state.connection,
            thread,
        }
    }
}


impl From<(PresentationRequestPreparedState, Presentation, Thread)> for FinishedState {
    fn from((state, presentation, thread): (PresentationRequestPreparedState, Presentation, Thread)) -> Self {
        trace!("OfferPreparedState: transit state from InitialState to FinishedState");
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: Some(presentation),
            status: Status::Success,
            thread,
        }
    }
}

impl From<(PresentationRequestPreparedState, Status, Thread)> for FinishedState {
    fn from((state, status, thread): (PresentationRequestPreparedState, Status, Thread)) -> Self {
        trace!("PresentationRequestPreparedState: transit state from InitialState to FinishedState");
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: None,
            status,
            thread,
        }
    }
}

impl From<(PresentationRequestPreparedState, CompletedConnection)> for PresentationRequestPreparedState {
    fn from((state, connection): (PresentationRequestPreparedState, CompletedConnection)) -> Self {
        trace!("OfferPreparedState: transit state from InitialState to PresentationRequestPreparedState");
        PresentationRequestPreparedState {
            presentation_request: state.presentation_request,
            connection: Some(connection),
        }
    }
}

impl From<(PresentationRequestSentState, Presentation, Thread)> for FinishedState {
    fn from((state, presentation, thread): (PresentationRequestSentState, Presentation, Thread)) -> Self {
        trace!("VerifierSM transit state from PresentationRequestSentState to FinishedState");
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: Some(presentation),
            status: Status::Success,
            thread,
        }
    }
}

impl From<(PresentationRequestSentState, Status, Thread)> for FinishedState {
    fn from((state, status, thread): (PresentationRequestSentState, Status, Thread)) -> Self {
        trace!("VerifierSM transit state from PresentationRequestSentState to FinishedState with Status: {:?}", status);
        trace!("Thread: {:?}", thread);
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: None,
            status,
            thread,
        }
    }
}

impl From<(PresentationRequestSentState, PresentationProposal, Thread)> for PresentationProposalReceivedState {
    fn from((state, presentation_proposal, thread): (PresentationRequestSentState, PresentationProposal, Thread)) -> Self {
        trace!("VerifierSM transit state from PresentationRequestSentState to PresentationProposalReceivedState with PresentationProposal: {:?}", presentation_proposal);
        trace!("Thread: {:?}", thread);
        PresentationProposalReceivedState {
            presentation_proposal,
            connection: Some(state.connection),
            thread,
        }
    }
}

impl From<(PresentationProposalReceivedState, PresentationRequest, CompletedConnection, Thread)> for PresentationRequestSentState {
    fn from((_state, presentation_request, connection, thread): (PresentationProposalReceivedState, PresentationRequest, CompletedConnection, Thread)) -> Self {
        trace!("VerifierSM transit state from PresentationProposalReceivedState to PresentationRequestSentState");
        trace!("Thread: {:?}", thread);
        PresentationRequestSentState {
            connection,
            presentation_request,
            thread,
        }
    }
}
