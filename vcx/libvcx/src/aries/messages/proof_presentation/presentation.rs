use std::convert::TryInto;
use serde::{de, Deserialize, Deserializer};

use crate::error::prelude::*;
use crate::aries::messages::proof_presentation::v10::presentation::Presentation as PresentationV1;
use crate::aries::messages::proof_presentation::v20::presentation::Presentation as PresentationV2;
use crate::aries::messages::ack::PleaseAck;
use crate::aries::messages::thread::Thread;
use crate::aries::messages::attachment::Attachments;
use crate::legacy::messages::proof_presentation::proof_message::ProofMessage;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypeVersion};

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum Presentation {
    V1(PresentationV1),
    V2(PresentationV2),
}

impl Presentation {
    pub fn reset_ack(self) -> Self {
        match self {
            Presentation::V1(presentation) => {
                Presentation::V1(presentation.reset_ack())
            }
            Presentation::V2(presentation) => {
                Presentation::V2(presentation.reset_ack())
            }
        }
    }

    pub fn set_thread(self, thread: Thread) -> Self {
        match self {
            Presentation::V1(presentation) => {
                Presentation::V1(presentation.set_thread(thread))
            }
            Presentation::V2(presentation) => {
                Presentation::V2(presentation.set_thread(thread))
            }
        }
    }

    pub fn type_(&self) -> &MessageType {
        match self {
            Presentation::V1(presentation) => &presentation.type_,
            Presentation::V2(presentation) => &presentation.type_,
        }
    }

    pub fn presentations_attach(&self) -> &Attachments {
        match self {
            Presentation::V1(presentation) => &presentation.presentations_attach,
            Presentation::V2(presentation) => &presentation.presentations_attach,
        }
    }

    pub fn please_ack(&self) -> Option<&PleaseAck> {
        match self {
            Presentation::V1(presentation) => presentation.please_ack.as_ref(),
            Presentation::V2(presentation) => presentation.please_ack.as_ref(),
        }
    }

    pub fn thread(&self) -> &Thread {
        match self {
            Presentation::V1(presentation) => &presentation.thread,
            Presentation::V2(presentation) => &presentation.thread,
        }
    }

    pub fn from_thread(&self, id: &str) -> bool {
        match self {
            Presentation::V1(presentation) => presentation.from_thread(id),
            Presentation::V2(presentation) => presentation.from_thread(id),
        }
    }
}

impl TryInto<Presentation> for ProofMessage {
    type Error = VcxError;

    fn try_into(self) -> Result<Presentation, Self::Error> {
        Ok(
            Presentation::V1(
                PresentationV1::create()
                    .set_presentations_attach(self.libindy_proof)?
                    .ask_for_ack()
            )
        )
    }
}

impl TryInto<ProofMessage> for Presentation {
    type Error = VcxError;

    fn try_into(self) -> Result<ProofMessage, Self::Error> {
        let mut proof = ProofMessage::new();
        let (_, attachment_content) = self.presentations_attach().content()?;
        proof.libindy_proof = attachment_content;
        Ok(proof)
    }
}

deserialize_v1_v2_message!(Presentation, PresentationV1, PresentationV2);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::proof_presentation::v10::presentation::tests::_presentation as _presentation_v1;

    pub fn _presentation() -> Presentation {
        Presentation::V1(_presentation_v1())
    }
}