use crate::aries::messages::discovery::disclose::ProtocolDescriptor;
use crate::aries::handlers::connection::agent::AgentInfo;
use crate::aries::handlers::connection::states::CompleteState;
use crate::aries::messages::connection::invite::Invitation;
use crate::aries::messages::outofband::invitation::Invitation as OutofbandInvitation;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::error::VcxResult;
use crate::aries::messages::connection::service::Service;

/*
    object returning by vcx_connection_info
*/

#[derive(Debug, Serialize)]
pub struct PairwiseConnectionInfo {
    pub my: SideConnectionInfo,
    pub their: Option<SideConnectionInfo>,
    pub invitation: Option<Invitations>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideConnectionInfo {
    pub did: String,
    pub recipient_keys: Vec<String>,
    pub routing_keys: Vec<String>,
    pub service_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<ProtocolDescriptor>>,
}

/*
    object store within Issuer / Holder / Verifier / Prover
    state machines as relationship to specific pairwise connection
*/

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CompletedConnection {
    pub agent: AgentInfo,
    pub data: CompleteState,
}

impl CompletedConnection {
    pub fn without_handshake(&self) -> bool {
        self.data.without_handshake()
    }

    pub fn service(&self) -> VcxResult<Option<Service>> {
        if self.without_handshake() && !self.agent.pw_did.is_empty() {
            Ok(Some(Service::create()
                .set_service_endpoint(self.agent.agency_endpoint()?)
                .set_recipient_keys(self.agent.recipient_keys())
                .set_routing_keys(self.agent.routing_keys()?)))
        } else {
            Ok(None)
        }
    }
}

/*
    helper structure to store Out-of-Band metadata
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutofbandMeta {
    pub goal_code: Option<String>,
    pub goal: Option<String>,
    pub handshake: bool,
    pub request_attach: Option<String>,
}

impl OutofbandMeta {
    pub fn new(goal_code: Option<String>, goal: Option<String>,
               handshake: bool, request_attach: Option<String>) -> OutofbandMeta {
        OutofbandMeta {
            goal_code,
            goal,
            handshake,
            request_attach,
        }
    }
}

/*
    Connection can be created with either Invitation of `connections` or `out-of-band` protocols
*/
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Invitations {
    ConnectionInvitation(Invitation),
    OutofbandInvitation(OutofbandInvitation),
}

impl From<Invitations> for DidDoc {
    fn from(invitation: Invitations) -> DidDoc {
        match invitation {
            Invitations::ConnectionInvitation(invitation_)=> DidDoc::from(invitation_),
            Invitations::OutofbandInvitation(invitation_)=> DidDoc::from(invitation_),
        }
    }
}

impl Invitations {
    pub fn recipient_key(&self) -> Option<String> {
        match self {
            Invitations::ConnectionInvitation(invitation_)=>
                invitation_.recipient_keys.get(0).cloned(),
            Invitations::OutofbandInvitation(invitation_)=>
                invitation_.services().get(0).and_then(|service| service.recipient_keys.get(0).cloned()),
        }
    }

    pub fn service_endpoint(&self) -> String {
        match self {
            Invitations::ConnectionInvitation(invitation_)=> invitation_.service_endpoint.clone(),
            Invitations::OutofbandInvitation(invitation_)=>
                invitation_.services().get(0).map(|service| service.service_endpoint.clone()).unwrap_or_default(),
        }
    }

    pub fn pthid(&self) -> Option<String>{
        match self {
            Invitations::ConnectionInvitation(_)=> None,
            Invitations::OutofbandInvitation(invitation_)=> Some(invitation_.id().to_string()),
        }
    }

    pub fn logo_url(&self) -> Option<&str>{
        match self {
            Invitations::ConnectionInvitation(invitation_)=> invitation_.profile_url.as_deref(),
            Invitations::OutofbandInvitation(invitation_)=> invitation_.profile_url(),
        }
    }

    pub fn public_did(&self) -> Option<&str>{
        match self {
            Invitations::ConnectionInvitation(invitation_)=> invitation_.public_did.as_deref(),
            Invitations::OutofbandInvitation(invitation_)=> invitation_.public_did(),
        }
    }

    pub fn name(&self) -> Option<&str>{
        match self {
            Invitations::ConnectionInvitation(invitation_)=> Some(invitation_.label.as_str()),
            Invitations::OutofbandInvitation(invitation_)=> invitation_.label(),
        }
    }
}