use serde::{de, Deserialize, Deserializer};

use crate::aries::messages::issuance::v10::credential_proposal::CredentialProposal as CredentialProposalV1;
use crate::aries::messages::issuance::v20::credential_proposal::CredentialProposal as CredentialProposalV2;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypeVersion};
use crate::aries::messages::thread::Thread;

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum CredentialProposal {
    V1(CredentialProposalV1),
    V2(CredentialProposalV2),
}

impl CredentialProposal {
    pub fn set_thread_id(self, thid: &str) -> Self {
        match self {
            CredentialProposal::V1(credential_proposal) => CredentialProposal::V1(credential_proposal.set_thread_id(thid)),
            CredentialProposal::V2(credential_proposal) => CredentialProposal::V2(credential_proposal.set_thread_id(thid))
        }
    }

    pub fn id(&self) -> String {
        match self {
            CredentialProposal::V1(credential_proposal) => {
                credential_proposal.id.to_string()
            },
            CredentialProposal::V2(credential_proposal) => {
                credential_proposal.id.to_string()
            },
        }
    }

    pub fn thread(&self) -> Option<&Thread> {
        match self {
            CredentialProposal::V1(credential_proposal) => {
                credential_proposal.thread.as_ref()
            },
            CredentialProposal::V2(credential_proposal) => {
                credential_proposal.thread.as_ref()
            },
        }
    }
}

deserialize_v1_v2_message!(CredentialProposal, CredentialProposalV1, CredentialProposalV2);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::issuance::v10::credential_proposal::tests::_credential_proposal as _credential_proposal_v1;

    pub fn _credential_proposal() -> CredentialProposal {
        CredentialProposal::V1(_credential_proposal_v1())
    }
}