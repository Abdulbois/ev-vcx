use crate::aries::messages::{
    a2a::A2AMessage,
    proof_presentation::{
        presentation_proposal::PresentationProposal,
        presentation::Presentation,
    },
    error::ProblemReport,
};
use crate::connection::Connections;
use crate::utils::object_cache::Handle;
use crate::utils::libindy::anoncreds::proof_request::ProofRequest;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum VerifierMessages {
    PreparePresentationRequest(),
    SetConnection(Handle<Connections>),
    SendPresentationRequest(Handle<Connections>),
    PresentationReceived(Presentation),
    PresentationProposalReceived(PresentationProposal),
    PresentationRejectReceived(ProblemReport),
    RequestPresentation(Handle<Connections>, ProofRequest),
    Unknown
}

impl From<A2AMessage> for VerifierMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::Presentation(presentation) => {
                VerifierMessages::PresentationReceived(presentation)
            }
            A2AMessage::PresentationProposal(presentation_proposal) => {
                VerifierMessages::PresentationProposalReceived(presentation_proposal)
            }
            A2AMessage::CommonProblemReport(report) |
            A2AMessage::PresentationReject(report)=> {
                VerifierMessages::PresentationRejectReceived(report)
            }
            _ => {
                VerifierMessages::Unknown
            }
        }
    }
}
