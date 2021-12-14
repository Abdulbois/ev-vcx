use crate::api::VcxStateType;
use crate::connection::Connection as ConnectionV1;
use crate::aries::handlers::connection::Connection as ConnectionV3;
use crate::aries::handlers::connection::states::{ActorDidExchangeState, DidExchangeState, CompleteState};
use crate::agent::messages::connection_upgrade::ConnectionUpgradeInfo;
use crate::settings;
use crate::settings::protocol::ProtocolTypes;
use crate::aries::handlers::connection::agent::AgentInfo;
use crate::aries::handlers::connection::connection_fsm::DidExchangeSM;
use crate::aries::handlers::connection::types::Invitations;
use crate::aries::messages::a2a::MessageId;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::connection::invite::Invitation as InvitationV3;
use crate::agent::messages::connection::{InviteDetail, SenderDetail, SenderAgencyDetail};
use crate::aries::messages::thread::Thread;

impl Into<(ConnectionV1, ActorDidExchangeState)> for ConnectionV3 {
    fn into(self) -> (ConnectionV1, ActorDidExchangeState) {
        let invitation = self.get_invitation();
        let data = ConnectionV1 {
            source_id: self.source_id().clone(),
            pw_did: self.agent_info().pw_did.clone(),
            pw_verkey: self.agent_info().pw_vk.clone(),
            state: VcxStateType::from_u32(self.state()),
            uuid: String::new(),
            endpoint: invitation.as_ref().map(|invitation_| invitation_.service_endpoint()).unwrap_or_default(),
            invite_detail: Some(InviteDetail {
                sender_detail: SenderDetail {
                    name: invitation.as_ref().and_then(|invitation_| invitation_.name().map(String::from)),
                    verkey: invitation.as_ref().and_then(|invitation_| invitation_.recipient_key()).unwrap_or_default(),
                    logo_url: invitation.as_ref().and_then(|invitation_| invitation_.logo_url().map(String::from)),
                    public_did: invitation.as_ref().and_then(|invitation_| invitation_.public_did().map(String::from)),
                    ..SenderDetail::default()
                },
                ..InviteDetail::default()
            }),
            redirect_detail: None,
            invite_url: None,
            agent_did: self.agent_info().agent_did.clone(),
            agent_vk: self.agent_info().agent_vk.clone(),
            their_pw_did: self.remote_did().unwrap_or_default(),
            their_pw_verkey: self.remote_vk().unwrap_or_default(),
            public_did: settings::get_config_value(settings::CONFIG_INSTITUTION_DID).ok(),
            their_public_did: invitation.as_ref().and_then(|invitation_| invitation_.public_did().map(String::from)),
            version: Some(ProtocolTypes::V2), // TODO check correctness
        };

        (data, self.state_object().to_owned())
    }
}

impl From<(ConnectionV1, ActorDidExchangeState)> for ConnectionV3 {
    fn from((connection, state): (ConnectionV1, ActorDidExchangeState)) -> ConnectionV3 {
        let agent_info = AgentInfo {
            pw_did: connection.pw_did.clone(),
            pw_vk: connection.pw_verkey.clone(),
            agent_did: connection.agent_did.clone(),
            agent_vk: connection.agent_vk.clone(),
        };

        ConnectionV3::from_parts(connection.source_id.clone(), agent_info, state)
    }
}

impl From<(&ConnectionV1, &InviteDetail, ConnectionUpgradeInfo)> for ConnectionV3 {
    fn from((connection, invitation, upgrade_data): (&ConnectionV1, &InviteDetail, ConnectionUpgradeInfo)) -> ConnectionV3 {
        // Connection upgrade change only Agency related information.
        // Agent and Pairwise leave the same as in legacy connection

        let recipient_keys = vec![connection.their_pw_verkey.clone()];
        let routing_keys = vec![connection.their_pw_verkey.clone(), upgrade_data.their_agency_verkey];

        let mut did_doc = DidDoc::default();
        did_doc.set_id(connection.their_pw_did.clone());
        did_doc.set_service_endpoint(upgrade_data.their_agency_endpoint.clone());
        did_doc.set_keys(recipient_keys.clone(), routing_keys.clone());

        ConnectionV3 {
            connection_sm: DidExchangeSM {
                source_id: connection.source_id.clone(),
                agent_info: AgentInfo {
                    pw_did: connection.pw_did.clone(),
                    pw_vk: connection.pw_verkey.clone(),
                    agent_did: connection.agent_did.clone(),
                    agent_vk: connection.agent_vk.clone(),
                },
                state: ActorDidExchangeState::Invitee(DidExchangeState::Completed(CompleteState {
                    invitation: Some(Invitations::ConnectionInvitation(InvitationV3 {
                        id: MessageId(invitation.sender_agency_detail.did.clone()),
                        label: invitation.sender_detail.name.clone().unwrap_or_default(),
                        recipient_keys,
                        routing_keys,
                        service_endpoint: upgrade_data.their_agency_endpoint,
                        profile_url: invitation.sender_detail.logo_url.clone(),
                        public_did: invitation.sender_detail.public_did.clone(),
                        ..InvitationV3::default()
                    })),
                    did_doc,
                    protocols: None,
                    thread: Thread::default(),
                })),
            }
        }
    }
}

impl From<(&ConnectionV3, Invitations, ConnectionUpgradeInfo)> for ConnectionV1 {
    fn from((connection, invitation, data): (&ConnectionV3, Invitations, ConnectionUpgradeInfo)) -> ConnectionV1 {
        ConnectionV1 {
            source_id: connection.source_id(),
            pw_did: connection.agent_info().pw_did.clone(),
            pw_verkey: connection.agent_info().pw_vk.clone(),
            state: VcxStateType::from_u32(connection.state()),
            uuid: String::new(),
            endpoint: data.their_agency_endpoint.clone(),
            invite_detail: Some(InviteDetail {
                sender_detail: SenderDetail {
                    did: connection.remote_did().unwrap_or_default(),
                    name: invitation.name().map(String::from),
                    verkey: invitation.recipient_key().unwrap_or_default(),
                    logo_url: invitation.logo_url().map(String::from),
                    public_did: invitation.public_did().map(String::from),
                    agent_key_dlg_proof: Default::default(),
                },
                sender_agency_detail: SenderAgencyDetail {
                    did: data.their_agency_did,
                    verkey: data.their_agency_verkey,
                    endpoint: data.their_agency_endpoint,
                },
                status_code: "MS-101".to_string(),
                conn_req_id: String::new(),
                target_name: "there".to_string(),
                status_msg: "message created".to_string(),
                thread_id: None,
                version: Some("1.0".to_string()),
            }),
            redirect_detail: None,
            invite_url: None,
            agent_did: connection.agent_info().agent_did.clone(),
            agent_vk: connection.agent_info().agent_vk.clone(),
            their_pw_did: connection.remote_did().unwrap_or_default(),
            their_pw_verkey: connection.remote_vk().unwrap_or_default(),
            public_did: settings::get_config_value(settings::CONFIG_INSTITUTION_DID).ok(),
            their_public_did: invitation.public_did().map(String::from),
            version: Some(settings::protocol::ProtocolTypes::V1),
        }
    }
}

impl From<(&ConnectionV3, ConnectionUpgradeInfo)> for ConnectionV3 {
    fn from((connection, upgrade_data): (&ConnectionV3, ConnectionUpgradeInfo)) -> ConnectionV3 {
        // Connection upgrade change only Agency related information.
        // Agent and Pairwise leave the same as in legacy connection

        let recipient_keys = vec![connection.remote_vk().unwrap_or_default()];
        let routing_keys = vec![connection.remote_vk().unwrap_or_default(), upgrade_data.their_agency_verkey];

        let mut did_doc = DidDoc::default();
        did_doc.set_id(connection.remote_did().unwrap_or_default());
        did_doc.set_service_endpoint(upgrade_data.their_agency_endpoint);
        did_doc.set_keys(recipient_keys, routing_keys);

        ConnectionV3 {
            connection_sm: DidExchangeSM {
                source_id: connection.source_id(),
                agent_info: connection.agent_info().clone(),
                state: ActorDidExchangeState::Invitee(DidExchangeState::Completed(CompleteState {
                    invitation: connection.get_invitation(),
                    did_doc,
                    protocols: None,
                    thread: connection.connection_sm.thread().cloned().unwrap_or_default(),
                })),
            }
        }
    }
}