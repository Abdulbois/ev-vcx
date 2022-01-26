use std::collections::HashMap;
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
use crate::utils::libindy::anoncreds::proof_request::{AttributeInfo, PredicateInfo, ProofRequest};

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

    pub fn parse(request: &str) -> VcxResult<String> {
        let request: PresentationRequest = ::serde_json::from_str(request)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest,
                                              format!("Cannot parse ProofRequest from `offer` JSON string. Err: {:?}", err)))?;

        let thid = request.thread().as_ref()
            .and_then(|thread| thread.thid.clone())
            .unwrap_or_else(|| request.id());

        let (_, presentation_request) = request.request_presentations_attach().content()?;

        let indy_request: ProofRequest = serde_json::from_str(&presentation_request)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer,
                                              format!("Cannot parse Indy Credential Offer from JSON string. Err: {:?}", err)))?;

        let info = PresentationRequestInfo {
            name: indy_request.name,
            version: indy_request.version,
            requested_attributes: indy_request.requested_attributes,
            requested_predicates: indy_request.requested_predicates,
            thid,
        };

        Ok(json!(info).to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PresentationRequestInfo {
    pub name: String,
    pub version: String,
    pub requested_attributes: HashMap<String, AttributeInfo>,
    pub requested_predicates: HashMap<String, PredicateInfo>,
    pub thid: String,
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
    #[test]
    fn test_parse_presentation_reqest(){
        let request = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/request-presentation","comment":"comment","request_presentations~attach":[{"@id":"libindy-request-presentation-0","data":{"base64":"eyJuYW1lIjoiIiwibm9uX3Jldm9rZWQiOm51bGwsIm5vbmNlIjoiIiwicmVxdWVzdGVkX2F0dHJpYnV0ZXMiOnsiYXR0cmlidXRlXzAiOnsibmFtZSI6Im5hbWUifX0sInJlcXVlc3RlZF9wcmVkaWNhdGVzIjp7fSwidmVyIjpudWxsLCJ2ZXJzaW9uIjoiMS4wIn0="},"mime-type":"application/json"}]}"#;
        let info = PresentationRequest::parse(request).unwrap();
        let attribute = AttributeInfo {
            name: Some("name".to_string()),
            names: None,
            restrictions: None,
            non_revoked: None,
            self_attest_allowed: None
        };
        let expected = PresentationRequestInfo {
            name: "".to_string(),
            version: "1.0".to_string(),
            requested_attributes: map!("attribute_0".to_string() => attribute),
            requested_predicates: Default::default(),
            thid: "testid".to_string()
        };
        assert_eq!(json!(expected).to_string(), info);
    }
}