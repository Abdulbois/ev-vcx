use crate::aries::messages::{
    issuance::credential_offer::CredentialOffer,
    issuance::credential::Credential,
    status::Status,
    error::{ProblemReport, Reason}
};
use crate::aries::handlers::connection::types::CompletedConnection;
use crate::aries::messages::thread::Thread;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HolderState {
    OfferReceived(OfferReceivedState),
    RequestSent(RequestSentState),
    Finished(FinishedHolderState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSentState {
    pub offer: Option<CredentialOffer>,
    pub req_meta: String,
    pub cred_def_json: String,
    pub connection: CompletedConnection,
    #[serde(default)]
    pub thread: Thread,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferReceivedState {
    pub offer: CredentialOffer,
    #[serde(default)]
    pub thread: Thread,
}

impl OfferReceivedState {
    pub fn new(offer: CredentialOffer) -> Self {
        let thread = match offer.thread() {
            Some(thread_) => thread_.clone(),
            None => Thread::new().set_thid(offer.id()),
        };
        trace!("Thread: {:?}", thread);
        OfferReceivedState {
            thread,
            offer,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedHolderState {
    pub offer: Option<CredentialOffer>,
    pub cred_id: Option<String>,
    pub credential: Option<Credential>,
    pub status: Status,
    #[serde(default)]
    pub thread: Thread,
}

impl From<(OfferReceivedState, String, String, CompletedConnection, Thread)> for RequestSentState {
    fn from((state, req_meta, cred_def_json, connection, thread): (OfferReceivedState, String, String, CompletedConnection, Thread)) -> Self {
        trace!("HolderSM: transit state from OfferReceivedState to RequestSentState");
        trace!("Thread: {:?}", state.thread);
        RequestSentState {
            offer: Some(state.offer),
            req_meta,
            cred_def_json,
            connection,
            thread,
        }
    }
}

impl From<(OfferReceivedState, String, Credential, Thread)> for FinishedHolderState {
    fn from((state, cred_id, credential, thread): (OfferReceivedState, String, Credential, Thread)) -> Self {
        trace!("HolderSM: transit state from OfferReceivedState to FinishedHolderState");
        trace!("Thread: {:?}", thread);
        FinishedHolderState {
            offer: Some(state.offer),
            cred_id: Some(cred_id),
            credential: Some(credential),
            status: Status::Success,
            thread,
        }
    }
}

impl From<(RequestSentState, String, Credential, Thread)> for FinishedHolderState {
    fn from((state, cred_id, credential, thread): (RequestSentState, String, Credential, Thread)) -> Self {
        trace!("HolderSM: transit state from RequestSentState to FinishedHolderState");
        trace!("Thread: {:?}", thread);
        FinishedHolderState {
            offer: state.offer,
            cred_id: Some(cred_id),
            credential: Some(credential),
            status: Status::Success,
            thread,
        }
    }
}

impl From<(RequestSentState, ProblemReport, Thread, Reason)> for FinishedHolderState {
    fn from((state, problem_report, thread, reason): (RequestSentState, ProblemReport, Thread, Reason)) -> Self {
        trace!("HolderSM: transit state from RequestSentState to FinishedHolderState with ProblemReport: {:?}", problem_report);
        trace!("Thread: {:?}", thread);
        FinishedHolderState {
            offer: state.offer,
            cred_id: None,
            credential: None,
            status: reason.to_status(problem_report),
            thread,
        }
    }
}

impl From<(OfferReceivedState, ProblemReport, Thread, Reason)> for FinishedHolderState {
    fn from((state, problem_report, thread, reason): (OfferReceivedState, ProblemReport, Thread, Reason)) -> Self {
        trace!("HolderSM: transit state from OfferReceivedState to FinishedHolderState with ProblemReport: {:?}", problem_report);
        trace!("Thread: {:?}", problem_report.thread);
        FinishedHolderState {
            offer: Some(state.offer),
            cred_id: None,
            credential: None,
            status: reason.to_status(problem_report),
            thread,
        }
    }
}