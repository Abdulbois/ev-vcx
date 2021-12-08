use crate::error::prelude::*;

use crate::aries::messages::outofband::v10::invitation::Invitation as InvitationV10;
use crate::aries::messages::outofband::v11::invitation::Invitation as InvitationV11;
use crate::aries::messages::a2a::MessageId;
use crate::aries::messages::connection::did_doc::Service;
use crate::aries::messages::attachment::Attachments;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum Invitation {
    V10(InvitationV10),
    V11(InvitationV11),
}

impl Invitation {
    pub fn id(&self) -> &MessageId {
        match self {
            Invitation::V10(invitation) => &invitation.id,
            Invitation::V11(invitation) => &invitation.id,
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            Invitation::V10(invitation) => invitation.label.as_deref(),
            Invitation::V11(invitation) => invitation.label.as_deref(),
        }
    }

    pub fn public_did(&self) -> Option<&str> {
        match self {
            Invitation::V10(invitation) => invitation.public_did.as_deref(),
            Invitation::V11(invitation) => invitation.public_did.as_deref(),
        }
    }

    pub fn profile_url(&self) -> Option<&str> {
        match self {
            Invitation::V10(invitation) => invitation.profile_url.as_deref(),
            Invitation::V11(invitation) => invitation.profile_url.as_deref(),
        }
    }

    pub fn handshake_protocols(&self) -> &Vec<String> {
        match self {
            Invitation::V10(invitation) => &invitation.handshake_protocols,
            Invitation::V11(invitation) => &invitation.handshake_protocols,
        }
    }

    pub fn services(&self) -> &Vec<Service> {
        match self {
            Invitation::V10(invitation) => &invitation.service,
            Invitation::V11(invitation) => &invitation.services,
        }
    }

    pub fn requests_attach(&self) -> &Attachments {
        match self {
            Invitation::V10(invitation) => &invitation.request_attach,
            Invitation::V11(invitation) => &invitation.request_attach,
        }
    }

    pub fn validate(&self)-> VcxResult<()> {
        match self {
            Invitation::V10(invitation) => {
                invitation.validate()
            }
            Invitation::V11(invitation) => {
                invitation.validate()
            }
        }
    }

    pub fn normalize_service_keys(&mut self)-> VcxResult<()> {
        match self {
            Invitation::V10(invitation) => {
                invitation.normalize_service_keys()
            }
            Invitation::V11(invitation) => {
                invitation.normalize_service_keys()
            }
        }
    }
}
