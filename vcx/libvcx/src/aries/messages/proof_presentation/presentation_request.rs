use std::convert::TryInto;
use serde::{de, Deserialize, Deserializer};

use crate::error::prelude::*;
use crate::aries::messages::proof_presentation::v10::presentation_request::PresentationRequest as PresentationRequestV1;
use crate::aries::messages::proof_presentation::v20::presentation_request::PresentationRequest as PresentationRequestV2;
use crate::aries::messages::connection::service::Service;
use crate::aries::messages::thread::Thread;
use crate::aries::messages::attachment::Attachments;
use crate::legacy::messages::proof_presentation::proof_request::ProofRequestMessage;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypeVersion};
use crate::utils::libindy::anoncreds::proof_request::ProofRequest;

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum PresentationRequest {
    V1(PresentationRequestV1),
    V2(PresentationRequestV2),
}

impl PresentationRequest {
    pub fn set_thread(self, thread: Thread) -> Self {
        match self {
            PresentationRequest::V1(presentation_request) => {
                PresentationRequest::V1(presentation_request.set_thread(thread))
            }
            PresentationRequest::V2(presentation_request) => {
                PresentationRequest::V2(presentation_request.set_thread(thread))
            }
        }
    }

    pub fn set_service(self, service: Option<Service>) -> Self {
        match self {
            PresentationRequest::V1(presentation_request) => {
                PresentationRequest::V1(presentation_request.set_service(service))
            }
            PresentationRequest::V2(presentation_request) => {
                PresentationRequest::V2(presentation_request.set_service(service))
            }
        }
    }

    pub fn id(&self) -> String {
        match self {
            PresentationRequest::V1(presentation_request) => presentation_request.id.to_string(),
            PresentationRequest::V2(presentation_request) => presentation_request.id.to_string(),
        }
    }

    pub fn type_(&self) -> &MessageType {
        match self {
            PresentationRequest::V1(presentation_request) => &presentation_request.type_,
            PresentationRequest::V2(presentation_request) => &presentation_request.type_,
        }
    }

    pub fn comment(&self) -> Option<&String> {
        match self {
            PresentationRequest::V1(presentation_request) => presentation_request.comment.as_ref(),
            PresentationRequest::V2(presentation_request) => presentation_request.comment.as_ref(),
        }
    }

    pub fn thread(&self) -> Option<&Thread> {
        match self {
            PresentationRequest::V1(presentation_request) => presentation_request.thread.as_ref(),
            PresentationRequest::V2(presentation_request) => presentation_request.thread.as_ref(),
        }
    }

    pub fn service(&self) -> Option<Service> {
        match self {
            PresentationRequest::V1(presentation_request) => presentation_request.service.clone(),
            PresentationRequest::V2(presentation_request) => presentation_request.service.clone(),
        }
    }

    pub fn request_presentations_attach(&self) -> &Attachments {
        match self {
            PresentationRequest::V1(presentation_request) => &presentation_request.request_presentations_attach,
            PresentationRequest::V2(presentation_request) => &presentation_request.request_presentations_attach,
        }
    }
}

impl TryInto<PresentationRequest> for ProofRequestMessage {
    type Error = VcxError;

    fn try_into(self) -> Result<PresentationRequest, Self::Error> {
        Ok(
            PresentationRequest::V1(
                PresentationRequestV1::create()
                    .set_id(self.thread_id.unwrap_or_default())
                    .set_request_presentations_attach(&self.proof_request_data)?
                    .set_opt_comment(self.comment.clone())
                    .set_service(self.service)
            )
        )
    }
}

impl TryInto<ProofRequestMessage> for PresentationRequest {
    type Error = VcxError;

    fn try_into(self) -> Result<ProofRequestMessage, Self::Error> {
        let thid = self.thread().and_then(|thread| thread.thid.clone()).unwrap_or(self.id());
        let (_, attachment_content) = &self.request_presentations_attach().content()?;
        let proof_request_data: ProofRequest = ::serde_json::from_str(&attachment_content)
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidProof,
                format!("Cannot deserialize Proof: {:?}", err))
            )?;

        let proof_request: ProofRequestMessage = ProofRequestMessage::create()
            .set_proof_request_data(proof_request_data)?
            .type_version("1.0")?
            .proof_data_version("0.1")?
            .set_thread_id(thid)?
            .set_comment(self.comment().map(String::from))?
            .set_service(self.service())?
            .clone();

        Ok(proof_request)
    }
}

deserialize_v1_v2_message!(PresentationRequest, PresentationRequestV1, PresentationRequestV2);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::proof_presentation::v10::presentation_request::tests::_presentation_request as _presentation_request_v1;

    pub fn _presentation_request() -> PresentationRequest {
        PresentationRequest::V1(_presentation_request_v1())
    }
}