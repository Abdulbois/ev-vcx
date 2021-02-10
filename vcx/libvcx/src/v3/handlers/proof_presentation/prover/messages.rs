use v3::messages::proof_presentation::presentation_request::PresentationRequest;
use v3::messages::proof_presentation::presentation_ack::PresentationAck;
use v3::messages::proof_presentation::presentation_proposal::PresentationPreview;
use v3::messages::error::ProblemReport;
use v3::messages::a2a::A2AMessage;
use v3::messages::proof_presentation::presentation::Presentation;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ProverMessages {
    PresentationRequestReceived(PresentationRequest),
    RejectPresentationRequest((u32, String)),
    SetPresentation(Presentation),
    PreparePresentation((String, String)),
    SendPresentation(u32),
    SendProposal(u32),
    PresentationAckReceived(PresentationAck),
    PresentationRejectReceived(ProblemReport),
    ProposePresentation((u32, PresentationPreview)),
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