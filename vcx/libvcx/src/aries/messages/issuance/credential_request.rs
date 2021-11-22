use serde::{de, Deserialize, Deserializer};

use crate::aries::messages::issuance::v10::credential_request::CredentialRequest as CredentialRequestV1;
use crate::aries::messages::issuance::v20::credential_request::CredentialRequest as CredentialRequestV2;
use crate::aries::messages::attachment::Attachments;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypeVersion};
use crate::aries::messages::thread::Thread;

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum CredentialRequest {
    V1(CredentialRequestV1),
    V2(CredentialRequestV2),
}

impl CredentialRequest {
    pub fn set_thread(self, thread: Thread) -> Self {
        match self {
            CredentialRequest::V1(credential_request) => {
                CredentialRequest::V1(credential_request.set_thread(thread))
            }
            CredentialRequest::V2(credential_request) => {
                CredentialRequest::V2(credential_request.set_thread(thread))
            }
        }
    }

    pub fn set_thread_id(self, thid: &str) -> Self {
        match self {
            CredentialRequest::V1(credential_request) => CredentialRequest::V1(credential_request.set_thread_id(thid)),
            CredentialRequest::V2(credential_request) => CredentialRequest::V2(credential_request.set_thread_id(thid))
        }
    }

    pub fn request_return_route(self) -> Self {
        match self {
            CredentialRequest::V1(credential_request) => {
                CredentialRequest::V1(credential_request.request_return_route())
            }
            CredentialRequest::V2(credential_request) => {
                CredentialRequest::V2(credential_request.request_return_route())
            }
        }
    }

    pub fn type_(&self) -> &MessageType {
        match self {
            CredentialRequest::V1(credential_request) => &credential_request.type_,
            CredentialRequest::V2(credential_request) => &credential_request.type_,
        }
    }

    pub fn requests_attach(&self) -> &Attachments {
        match self {
            CredentialRequest::V1(credential_request) => {
                &credential_request.requests_attach
            },
            CredentialRequest::V2(credential_request) => {
                &credential_request.requests_attach
            }
        }
    }

    pub fn thread(&self) -> &Thread {
        match self {
            CredentialRequest::V1(credential_request) => {
                &credential_request.thread
            },
            CredentialRequest::V2(credential_request) => {
                &credential_request.thread
            },
        }
    }

    pub fn from_thread(&self, id: &str) -> bool {
        match self {
            CredentialRequest::V1(credential_request) => {
                credential_request.from_thread(id)
            },
            CredentialRequest::V2(credential_request) => {
                credential_request.from_thread(id)
            },
        }
    }
}

deserialize_v1_v2_message!(CredentialRequest, CredentialRequestV1, CredentialRequestV2);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::issuance::v10::credential_request::tests::_credential_request as _credential_request_v1;

    pub fn _credential_request() -> CredentialRequest {
        CredentialRequest::V1(_credential_request_v1())
    }
}