use crate::v3::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::v3::messages::proof_presentation::presentation_ack::PresentationAck;
use crate::v3::messages::proof_presentation::presentation_proposal::PresentationPreview;
use crate::v3::messages::error::ProblemReport;
use crate::v3::messages::a2a::A2AMessage;
use crate::v3::messages::proof_presentation::presentation::Presentation;

use crate::connection::Connections;
use crate::object_cache::Handle;

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
