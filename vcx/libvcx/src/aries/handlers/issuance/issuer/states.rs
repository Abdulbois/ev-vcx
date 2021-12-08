use crate::aries::messages::{
    issuance::{
        credential_request::CredentialRequest,
        credential_offer::CredentialOffer,
    },
    status::Status,
};
use crate::aries::handlers::connection::types::CompletedConnection;
use crate::aries::messages::thread::Thread;

// Possible Transitions:
// Initial -> OfferSent
// Initial -> Finished
// OfferSent -> CredentialSent
// OfferSent -> Finished
// CredentialSent -> Finished
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IssuerState {
    Initial(InitialState),
    OfferSent(OfferSentState),
    RequestReceived(RequestReceivedState),
    CredentialSent(CredentialSentState),
    Finished(FinishedState),
}

impl InitialState {
    pub fn new(cred_def_id: &str, credential_json: &str, rev_reg_id: Option<String>, tails_file: Option<String>, credential_name: Option<String>) -> Self {
        InitialState {
            cred_def_id: cred_def_id.to_string(),
            credential_json: credential_json.to_string(),
            rev_reg_id,
            tails_file,
            credential_name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialState {
    pub cred_def_id: String,
    pub credential_json: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferSentState {
    pub offer: CredentialOffer,
    pub cred_data: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    pub connection: CompletedConnection,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestReceivedState {
    pub offer: CredentialOffer,
    pub cred_data: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    pub request: CredentialRequest,
    pub connection: CompletedConnection,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CredentialSentState {
    pub offer: CredentialOffer,
    pub connection: CompletedConnection,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedState {
    pub offer: Option<CredentialOffer>,
    pub cred_id: Option<String>,
    pub status: Status,
    #[serde(default)]
    pub thread: Thread,
}

impl From<(InitialState, CredentialOffer, CompletedConnection, Thread)> for OfferSentState {
    fn from((state, offer, connection, thread): (InitialState, CredentialOffer, CompletedConnection, Thread)) -> Self {
        trace!("IssuerSM: transit state from InitialState to OfferSentState");
        trace!("Thread: {:?}", thread);
        OfferSentState {
            offer,
            cred_data: state.credential_json,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
            connection,
            thread,
        }
    }
}

impl From<(OfferSentState, CredentialRequest, Thread)> for RequestReceivedState {
    fn from((state, request, thread): (OfferSentState, CredentialRequest, Thread)) -> Self {
        trace!("IssuerSM: transit state from OfferSentState to RequestReceivedState");
        trace!("Thread: {:?}", thread);
        RequestReceivedState {
            offer: state.offer,
            cred_data: state.cred_data,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
            request,
            connection: state.connection,
            thread,
        }
    }
}

impl From<(RequestReceivedState, Thread)> for CredentialSentState {
    fn from((state, thread): (RequestReceivedState, Thread)) -> Self {
        trace!("IssuerSM: transit state from RequestReceivedState to CredentialSentState");
        trace!("Thread: {:?}", thread);
        CredentialSentState {
            offer: state.offer,
            connection: state.connection,
            thread,
        }
    }
}

impl From<(OfferSentState, Status, Thread)> for FinishedState {
    fn from((state, status, thread): (OfferSentState, Status, Thread)) -> Self {
        trace!("IssuerSM: transit state from OfferSentState to FinishedState with ProblemReport: {:?}", status);
        trace!("Thread: {:?}", thread);
        FinishedState {
            cred_id: None,
            offer: Some(state.offer),
            status,
            thread,
        }
    }
}

impl From<(RequestReceivedState, Thread)> for FinishedState {
    fn from((state, thread): (RequestReceivedState, Thread)) -> Self {
        trace!("IssuerSM: transit state from RequestReceivedState to FinishedState");
        trace!("Thread: {:?}", thread);
        FinishedState {
            cred_id: None,
            offer: Some(state.offer),
            status: Status::Success,
            thread,
        }
    }
}

impl From<(RequestReceivedState, Status, Thread)> for FinishedState {
    fn from((state, status, thread): (RequestReceivedState, Status, Thread)) -> Self {
        trace!("IssuerSM: transit state from RequestReceivedState to FinishedState with ProblemReport: {:?}", status);
        trace!("Thread: {:?}", thread);
        FinishedState {
            cred_id: None,
            offer: Some(state.offer),
            status,
            thread,
        }
    }
}

impl From<(CredentialSentState, Thread)> for FinishedState {
    fn from((state, thread): (CredentialSentState, Thread)) -> Self {
        trace!("IssuerSM: transit state from CredentialSentState to FinishedState");
        trace!("Thread: {:?}", thread);
        FinishedState {
            cred_id: None,
            offer: Some(state.offer),
            status: Status::Success,
            thread,
        }
    }
}

impl From<(CredentialSentState, Status, Thread)> for FinishedState {
    fn from((state, status, thread): (CredentialSentState, Status, Thread)) -> Self {
        trace!("IssuerSM: transit state from CredentialSentState to FinishedState with ProblemReport: {:?}", status);
        trace!("Thread: {:?}", thread);
        FinishedState {
            cred_id: None,
            offer: Some(state.offer),
            status,
            thread,
        }
    }
}