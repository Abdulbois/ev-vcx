use crate::aries::messages::{
    a2a::A2AMessage,
    error::ProblemReport,
    issuance::{
        credential_proposal::CredentialProposal,
        credential_offer::CredentialOffer,
        credential::Credential,
    },
};
use crate::connection::Connections;
use crate::utils::object_cache::Handle;


#[derive(Debug, Clone)]
pub enum HolderMessages {
    CredentialProposal(CredentialProposal),
    CredentialOffer(CredentialOffer),
    CredentialRequestSend(Handle<Connections>),
    Credential(Credential),
    ProblemReport(ProblemReport),
    CredentialRejectSend((Handle<Connections>, Option<String>)),
    Unknown,
}

impl From<A2AMessage> for HolderMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::CredentialProposal(proposal) => {
                HolderMessages::CredentialProposal(proposal)
            }
            A2AMessage::CredentialOffer(offer) => {
                HolderMessages::CredentialOffer(offer)
            }
            A2AMessage::Credential(credential) => {
                HolderMessages::Credential(credential)
            }
            A2AMessage::CommonProblemReport(report) |
            A2AMessage::CredentialReject(report) => {
                HolderMessages::ProblemReport(report)
            }
            _ => {
                HolderMessages::Unknown
            }
        }
    }
}
