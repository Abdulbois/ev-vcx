use crate::aries::messages::{
    a2a::A2AMessage,
    error::ProblemReport,
    proof_presentation::{
        presentation_request::PresentationRequest,
        presentation_ack::PresentationAck,
        presentation_preview::PresentationPreview,
        presentation::Presentation
    }
};
use crate::connection::Connections;
use crate::utils::object_cache::Handle;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ProverMessages {
    PresentationRequestReceived(PresentationRequest),
    RejectPresentationRequest((Handle<Connections>, String)),
    SetPresentation(Presentation),
    PreparePresentation((String, String)),
    SendPresentation(Handle<Connections>),
    SendProposal(Handle<Connections>),
    PresentationAckReceived(PresentationAck),
    PresentationRejectReceived(ProblemReport),
    ProposePresentation((Handle<Connections>, PresentationPreview)),
    Unknown
}

impl From<A2AMessage> for ProverMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::Ack(ack) | A2AMessage::PresentationAck(ack) => {
                ProverMessages::PresentationAckReceived(ack)
            }
            A2AMessage::CommonProblemReport(report) |
            A2AMessage::PresentationReject(report) => {
                ProverMessages::PresentationRejectReceived(report)
            }
            A2AMessage::PresentationRequest(request) => {
                ProverMessages::PresentationRequestReceived(request)
            }
            _ => {
                ProverMessages::Unknown
            }
        }
    }
}
