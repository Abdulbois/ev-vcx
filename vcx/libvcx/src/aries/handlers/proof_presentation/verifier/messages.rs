use crate::aries::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::aries::messages::proof_presentation::presentation::Presentation;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::proof_presentation::presentation_request::PresentationRequestData;

use crate::connection::Connections;
use crate::object_cache::Handle;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum VerifierMessages {
    PreparePresentationRequest(),
    SetConnection(Handle<Connections>),
    SendPresentationRequest(Handle<Connections>),
    PresentationReceived(Presentation),
    PresentationProposalReceived(PresentationProposal),
    PresentationRejectReceived(ProblemReport),
    RequestPresentation(Handle<Connections>, PresentationRequestData),
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
