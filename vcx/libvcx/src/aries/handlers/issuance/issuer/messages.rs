use crate::aries::messages::{
    error::ProblemReport,
    a2a::A2AMessage,
    ack::Ack,
    issuance::{
        credential_proposal::CredentialProposal,
        credential_request::CredentialRequest,
    },
};
use crate::connection::Connections;
use crate::utils::object_cache::Handle;


#[derive(Debug, Clone)]
pub enum IssuerMessages {
    CredentialInit(Handle<Connections>),
    CredentialRequest(CredentialRequest),
    CredentialSend(Handle<Connections>),
    CredentialProposal(CredentialProposal),
    CredentialAck(Ack),
    ProblemReport(ProblemReport),
    CredentialRejectSend((Handle<Connections>, Option<String>)),
    Unknown,
}

impl From<A2AMessage> for IssuerMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::CredentialProposal(proposal) => {
                IssuerMessages::CredentialProposal(proposal)
            }
            A2AMessage::CredentialRequest(request) => {
                IssuerMessages::CredentialRequest(request)
            }
            A2AMessage::Ack(ack) | A2AMessage::CredentialAck(ack) => {
                IssuerMessages::CredentialAck(ack)
            }
            A2AMessage::CommonProblemReport(report) |
            A2AMessage::CredentialReject(report) => {
                IssuerMessages::ProblemReport(report)
            }
            _ => {
                IssuerMessages::Unknown
            }
        }
    }
}

