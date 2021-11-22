use serde::{de, Deserialize, Deserializer};

use crate::aries::messages::proof_presentation::v10::presentation_proposal::PresentationProposal as PresentationProposalV1;
use crate::aries::messages::proof_presentation::v20::presentation_proposal::PresentationProposal as PresentationProposalV2;
use crate::aries::messages::thread::Thread;
use crate::error::VcxResult;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypeVersion};

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum PresentationProposal {
    V1(PresentationProposalV1),
    V2(PresentationProposalV2),
}

impl PresentationProposal {
    pub fn id(&self) -> String {
        match self {
            PresentationProposal::V1(presentation_proposal) => presentation_proposal.id.0.to_string(),
            PresentationProposal::V2(presentation_proposal) => presentation_proposal.id.0.to_string(),
        }
    }

    pub fn presentation_preview(&self) -> VcxResult<String> {
        match self {
            PresentationProposal::V1(presentation_proposal) => {
                Ok(json!(&presentation_proposal.presentation_proposal).to_string())
            }
            PresentationProposal::V2(presentation_proposal) => {
                presentation_proposal.proposals_attach.content().map(|(_, data)| data)
            }
        }
    }

    pub fn thread(&self) -> Option<&Thread> {
        match self {
            PresentationProposal::V1(presentation_proposal) => presentation_proposal.thread.as_ref(),
            PresentationProposal::V2(presentation_proposal) => presentation_proposal.thread.as_ref(),
        }
    }
}

deserialize_v1_v2_message!(PresentationProposal, PresentationProposalV1, PresentationProposalV2);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::proof_presentation::v10::presentation_proposal::tests::_presentation_proposal as _presentation_proposal_v1;

    pub fn _presentation_proposal() -> PresentationProposal {
        PresentationProposal::V1(_presentation_proposal_v1())
    }
}