use core::fmt::Debug;
use crate::api::VcxStateType;

use crate::aries::handlers::connection::agent::AgentInfo;
use crate::aries::messages::connection::invite::Invitation;
use crate::aries::messages::connection::request::Request;
use crate::aries::messages::connection::response::{Response, SignedResponse};
use crate::aries::messages::connection::problem_report::ProblemReport;
use crate::aries::messages::trust_ping::ping::Ping;
use crate::aries::messages::trust_ping::ping_response::PingResponse;
use crate::aries::messages::ack::Ack;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::discovery::disclose::ProtocolDescriptor;
use crate::aries::messages::outofband::invitation::Invitation as OutofbandInvitation;

use crate::aries::messages::thread::Thread;
use serde::Serialize;
use crate::aries::handlers::connection::types::{OutofbandMeta, Invitations};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorDidExchangeState {
    Inviter(DidExchangeState),
    Invitee(DidExchangeState),
}

/// Transitions of Inviter Connection state
/// Initialized -> Invited
/// Invited -> Responded, Failed
/// Responded -> Complete, Failed
/// Completed
/// Failed
///
/// Transitions of Invitee Connection state
/// Initialized -> Invited
/// Invited -> Requested, Failed
/// Requested -> Completed, Failed
/// Completed
/// Failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DidExchangeState {
    Initialized(InitializedState),
    Invited(InvitedState),
    Requested(RequestedState),
    Responded(RespondedState),
    Completed(CompleteState),
    Failed(FailedState),
}

impl DidExchangeState {
    pub fn code(&self) -> u32 {
        match self {
            DidExchangeState::Initialized(_) => VcxStateType::VcxStateInitialized as u32,
            DidExchangeState::Invited(_) => VcxStateType::VcxStateOfferSent as u32,
            DidExchangeState::Requested(_) => VcxStateType::VcxStateRequestReceived as u32,
            DidExchangeState::Responded(_) => VcxStateType::VcxStateRequestReceived as u32,
            DidExchangeState::Completed(_) => VcxStateType::VcxStateAccepted as u32,
            DidExchangeState::Failed(_) => VcxStateType::VcxStateNone as u32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializedState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outofband_meta: Option<OutofbandMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitedState {
    pub invitation: Invitations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestedState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation: Option<Invitations>,
    pub request: Request,
    pub did_doc: DidDoc,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondedState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation: Option<Invitations>,
    pub response: SignedResponse,
    pub did_doc: DidDoc,
    pub prev_agent_info: AgentInfo,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation: Option<Invitations>,
    pub did_doc: DidDoc,
    pub protocols: Option<Vec<ProtocolDescriptor>>,
    #[serde(default)]
    pub thread: Thread,
}

impl CompleteState {
    pub fn without_handshake(&self) -> bool {
        if let Some(Invitations::OutofbandInvitation(invitation)) = self.invitation.as_ref() {
            invitation.handshake_protocols().is_empty()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation: Option<Invitations>,
    pub error: Option<ProblemReport>,
    #[serde(default)]
    pub thread: Thread,
}

impl From<(InitializedState, Invitation)> for InvitedState {
    fn from((_state, invitation): (InitializedState, Invitation)) -> InvitedState {
        trace!("DidExchangeStateSM: transit state from InitializedState to InvitedState with ConnectionInvitation");
        InvitedState { invitation: Invitations::ConnectionInvitation(invitation) }
    }
}

impl From<(InitializedState, OutofbandInvitation)> for InvitedState {
    fn from((_state, invitation): (InitializedState, OutofbandInvitation)) -> InvitedState {
        trace!("DidExchangeStateSM: transit state from InitializedState to InvitedState with OutofbandInvitation");
        InvitedState { invitation: Invitations::OutofbandInvitation(invitation) }
    }
}

impl From<(InitializedState, OutofbandInvitation)> for CompleteState {
    fn from((_state, invitation): (InitializedState, OutofbandInvitation)) -> CompleteState {
        trace!("DidExchangeStateSM: transit state from InitializedState to CompleteState with Out-of-Band Invitation");
        let thread = Thread::new()
            .set_pthid(invitation.id().to_string());

        CompleteState {
            did_doc: DidDoc::from(invitation.clone()),
            protocols: None,
            thread,
            invitation: Some(Invitations::OutofbandInvitation(invitation)),
        }
    }
}

impl From<(InvitedState, ProblemReport, Thread)> for FailedState {
    fn from((state, error, thread): (InvitedState, ProblemReport, Thread)) -> FailedState {
        trace!("DidExchangeStateSM: transit state from InvitedState to FailedState with ProblemReport message: {:?}", error);
        trace!("Thread: {:?}", thread);
        FailedState {
            invitation: Some(state.invitation),
            error: Some(error),
            thread,
        }
    }
}

impl From<(InvitedState, Request, Thread)> for RequestedState {
    fn from((state, request, thread): (InvitedState, Request, Thread)) -> RequestedState {
        trace!("DidExchangeStateSM: transit state from InvitedState to RequestedState");
        trace!("Thread: {:?}", thread);
        RequestedState {
            invitation: Some(state.invitation.clone()),
            request,
            did_doc: DidDoc::from(state.invitation),
            thread,
        }
    }
}

impl From<(InvitedState, Request, SignedResponse, AgentInfo, Thread)> for RespondedState {
    fn from((state, request, response, prev_agent_info, thread): (InvitedState, Request, SignedResponse, AgentInfo, Thread)) -> RespondedState {
        trace!("DidExchangeStateSM: transit state from InvitedState to RequestedState");
        trace!("Thread: {:?}", thread);
        RespondedState {
            invitation: Some(state.invitation),
            response,
            did_doc: request.connection.did_doc,
            prev_agent_info,
            thread,
        }
    }
}

impl From<(RespondedState, Ping, Thread)> for RespondedState {
    fn from((state, _ping, thread): (RespondedState, Ping, Thread)) -> RespondedState {
        trace!("DidExchangeStateSM: transit state from RespondedState to RespondedState");
        trace!("Thread: {:?}", thread);
        RespondedState {
            invitation: state.invitation,
            response: state.response,
            did_doc: state.did_doc,
            prev_agent_info:
            state.prev_agent_info,
            thread,
        }
    }
}

impl From<(RequestedState, ProblemReport, Thread)> for FailedState {
    fn from((state, error, thread): (RequestedState, ProblemReport, Thread)) -> FailedState {
        trace!("DidExchangeStateSM: transit state from RequestedState to FailedState with ProblemReport: {:?}", error);
        trace!("Thread: {:?}", thread);
        FailedState {
            invitation: state.invitation,
            error: Some(error),
            thread,
        }
    }
}

impl From<(RequestedState, Response, Thread)> for CompleteState {
    fn from((state, response, thread): (RequestedState, Response, Thread)) -> CompleteState {
        trace!("DidExchangeStateSM: transit state from RequestedState to RespondedState");
        trace!("Thread: {:?}", thread);
        CompleteState {
            did_doc: response.connection.did_doc,
            protocols: None,
            thread: Thread {
                thid: thread.thid,
                pthid: state.thread.pthid,
                sender_order: thread.sender_order,
                received_orders: thread.received_orders
            },
            invitation: state.invitation,
        }
    }
}

impl From<(RespondedState, ProblemReport, Thread)> for FailedState {
    fn from((state, error, thread): (RespondedState, ProblemReport, Thread)) -> FailedState {
        trace!("DidExchangeStateSM: transit state from RespondedState to FailedState with ProblemReport message: {:?}", error);
        trace!("Thread: {:?}", thread);
        FailedState {
            invitation: state.invitation,
            error: Some(error),
            thread,
        }
    }
}

impl From<(RespondedState, Ack, Thread)> for CompleteState {
    fn from((state, _ack, thread): (RespondedState, Ack, Thread)) -> CompleteState {
        trace!("DidExchangeStateSM: transit state from RespondedState to CompleteState with Ack");
        trace!("Thread: {:?}", thread);
        CompleteState {
            did_doc: state.did_doc,
            protocols: None,
            thread,
            invitation: state.invitation,
        }
    }
}

impl From<(RespondedState, Ping, Thread)> for CompleteState {
    fn from((state, _ping, thread): (RespondedState, Ping, Thread)) -> CompleteState {
        trace!("DidExchangeStateSM: transit state from RespondedState to CompleteState with Ping");
        trace!("Thread: {:?}", thread);
        CompleteState {
            did_doc: state.did_doc,
            protocols: None,
            thread,
            invitation: state.invitation,
        }
    }
}

impl From<(RespondedState, PingResponse, Thread)> for CompleteState {
    fn from((state, _ping_response, thread): (RespondedState, PingResponse, Thread)) -> CompleteState {
        trace!("DidExchangeStateSM: transit state from RespondedState to CompleteState with PingResponse");
        trace!("Thread: {:?}", thread);
        CompleteState {
            did_doc: state.did_doc,
            protocols: None,
            thread,
            invitation: state.invitation,
        }
    }
}

impl From<(CompleteState, Vec<ProtocolDescriptor>)> for CompleteState {
    fn from((state, protocols): (CompleteState, Vec<ProtocolDescriptor>)) -> CompleteState {
        trace!("DidExchangeStateSM: transit state from CompleteState to CompleteState");
        CompleteState {
            did_doc: state.did_doc,
            protocols: Some(protocols),
            thread: state.thread,
            invitation: state.invitation,
        }
    }
}