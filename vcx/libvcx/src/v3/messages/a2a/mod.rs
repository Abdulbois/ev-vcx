pub mod message_family;
pub mod message_type;
pub mod protocol_registry;

use self::message_type::MessageType;
use self::message_family::MessageTypeFamilies;

use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::v3::messages::connection::invite::Invitation;
use crate::v3::messages::connection::request::Request;
use crate::v3::messages::connection::response::SignedResponse;
use crate::v3::messages::connection::problem_report::ProblemReport as ConnectionProblemReport;
use crate::v3::messages::trust_ping::ping::Ping;
use crate::v3::messages::trust_ping::ping_response::PingResponse;
use crate::v3::messages::forward::Forward;
use crate::v3::messages::error::ProblemReport as CommonProblemReport;
use crate::v3::messages::issuance::credential_proposal::CredentialProposal;
use crate::v3::messages::ack::Ack;
use crate::v3::messages::outofband::invitation::Invitation as OutofbandInvitation;
use crate::v3::messages::outofband::handshake_reuse::HandshakeReuse;
use crate::v3::messages::outofband::handshake_reuse_accepted::HandshakeReuseAccepted;

use crate::v3::messages::issuance::credential_offer::CredentialOffer;
use crate::v3::messages::issuance::credential_request::CredentialRequest;
use crate::v3::messages::issuance::credential::Credential;

use crate::v3::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::v3::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::v3::messages::proof_presentation::presentation::Presentation;

use crate::v3::messages::discovery::query::Query;
use crate::v3::messages::discovery::disclose::Disclose;

use crate::v3::messages::basic_message::message::BasicMessage;

use crate::v3::messages::questionanswer::question::Question;
use crate::v3::messages::questionanswer::answer::Answer;

use crate::v3::messages::committedanswer::question::Question as CommitedQuestion;
use crate::v3::messages::committedanswer::answer::Answer as CommitedAnswer;

use crate::v3::messages::invite_action::invite::Invite as InviteForAction;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum A2AMessage {
    /// routing
    Forward(Forward),

    /// DID Exchange
    ConnectionInvitation(Invitation),
    ConnectionRequest(Request),
    ConnectionResponse(SignedResponse),
    ConnectionProblemReport(ConnectionProblemReport),

    /// trust ping
    Ping(Ping),
    PingResponse(PingResponse),

    /// notification
    Ack(Ack),
    CommonProblemReport(CommonProblemReport),

    /// credential issuance
    CredentialProposal(CredentialProposal),
    CredentialOffer(CredentialOffer),
    CredentialRequest(CredentialRequest),
    Credential(Credential),
    CredentialAck(Ack),
    CredentialReject(CommonProblemReport),

    /// proof presentation
    PresentationProposal(PresentationProposal),
    PresentationRequest(PresentationRequest),
    Presentation(Presentation),
    PresentationAck(Ack),
    PresentationReject(CommonProblemReport),

    /// discovery features
    Query(Query),
    Disclose(Disclose),

    /// basic message
    BasicMessage(BasicMessage),

    /// questionanswer
    Question(Question),
    Answer(Answer),

    /// committedanswer
    CommittedQuestion(CommitedQuestion),
    CommittedAnswer(CommitedAnswer),

    /// Out-of-Band
    OutOfBandInvitation(OutofbandInvitation),
    HandshakeReuse(HandshakeReuse),
    HandshakeReuseAccepted(HandshakeReuseAccepted),

    /// invite-action
    InviteForAction(InviteForAction),
    InviteForActionAck(Ack),
    InviteForActionReject(CommonProblemReport),

    /// Any Raw Message
    Generic(Value),
}

impl<'de> Deserialize<'de> for A2AMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        trace!("deserializing aries a2a message");

        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;

        let message_type: MessageType = match serde_json::from_value(value["@type"].clone()) {
            Ok(message_type) => message_type,
            Err(_) => return Ok(A2AMessage::Generic(value))
        };

        match (message_type.family, message_type.type_.as_str()) {
            (MessageTypeFamilies::Routing, A2AMessage::FORWARD) => {
                Forward::deserialize(value)
                    .map(|msg| A2AMessage::Forward(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Connections, A2AMessage::CONNECTION_INVITATION) => {
                Invitation::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionInvitation(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Connections, A2AMessage::CONNECTION_REQUEST) => {
                Request::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Connections, A2AMessage::CONNECTION_RESPONSE) => {
                SignedResponse::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionResponse(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::TrustPing, A2AMessage::PING) => {
                Ping::deserialize(value)
                    .map(|msg| A2AMessage::Ping(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::TrustPing, A2AMessage::PING_RESPONSE) => {
                PingResponse::deserialize(value)
                    .map(|msg| A2AMessage::PingResponse(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Connections, A2AMessage::CONNECTION_PROBLEM_REPORT) => {
                ConnectionProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionProblemReport(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Notification, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::Ack(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::ReportProblem, A2AMessage::PROBLEM_REPORT) => {
                CommonProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::CommonProblemReport(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::CredentialIssuance, A2AMessage::CREDENTIAL) => {
                Credential::deserialize(value)
                    .map(|msg| A2AMessage::Credential(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::CredentialIssuance, A2AMessage::PROPOSE_CREDENTIAL) => {
                CredentialProposal::deserialize(value)
                    .map(|msg| A2AMessage::CredentialProposal(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::CredentialIssuance, A2AMessage::CREDENTIAL_OFFER) => {
                CredentialOffer::deserialize(value)
                    .map(|msg| A2AMessage::CredentialOffer(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::CredentialIssuance, A2AMessage::REQUEST_CREDENTIAL) => {
                CredentialRequest::deserialize(value)
                    .map(|msg| A2AMessage::CredentialRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::CredentialIssuance, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::CredentialAck(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::CredentialIssuance, A2AMessage::PROBLEM_REPORT) => {
                CommonProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::CredentialReject(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::PresentProof, A2AMessage::PROPOSE_PRESENTATION) => {
                PresentationProposal::deserialize(value)
                    .map(|msg| A2AMessage::PresentationProposal(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::PresentProof, A2AMessage::REQUEST_PRESENTATION) => {
                PresentationRequest::deserialize(value)
                    .map(|msg| A2AMessage::PresentationRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::PresentProof, A2AMessage::PRESENTATION) => {
                Presentation::deserialize(value)
                    .map(|msg| A2AMessage::Presentation(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::PresentProof, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::PresentationAck(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::PresentProof, A2AMessage::PROBLEM_REPORT) => {
                CommonProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::PresentationReject(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::DiscoveryFeatures, A2AMessage::QUERY) => {
                Query::deserialize(value)
                    .map(|msg| A2AMessage::Query(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::DiscoveryFeatures, A2AMessage::DISCLOSE) => {
                Disclose::deserialize(value)
                    .map(|msg| A2AMessage::Disclose(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Basicmessage, A2AMessage::BASIC_MESSAGE) => {
                BasicMessage::deserialize(value)
                    .map(|msg| A2AMessage::BasicMessage(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::QuestionAnswer, A2AMessage::QUESTION) => {
                Question::deserialize(value)
                    .map(|msg| A2AMessage::Question(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::QuestionAnswer, A2AMessage::ANSWER) => {
                Answer::deserialize(value)
                    .map(|msg| A2AMessage::Answer(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Committedanswer, A2AMessage::QUESTION) => {
                CommitedQuestion::deserialize(value)
                    .map(|msg| A2AMessage::CommittedQuestion(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Committedanswer, A2AMessage::ANSWER) => {
                CommitedAnswer::deserialize(value)
                    .map(|msg| A2AMessage::CommittedAnswer(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Outofband, A2AMessage::OUTOFBAND_INVITATION) => {
                OutofbandInvitation::deserialize(value)
                    .map(|msg| A2AMessage::OutOfBandInvitation(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Outofband, A2AMessage::OUTOFBAND_HANDSHAKE_REUSE) => {
                HandshakeReuse::deserialize(value)
                    .map(|msg| A2AMessage::HandshakeReuse(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::Outofband, A2AMessage::OUTOFBAND_HANDSHAKE_REUSE_ACCEPTED) => {
                HandshakeReuseAccepted::deserialize(value)
                    .map(|msg| A2AMessage::HandshakeReuseAccepted(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::InviteAction, A2AMessage::INVITE_FOR_ACTION) => {
                InviteForAction::deserialize(value)
                    .map(|msg| A2AMessage::InviteForAction(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::InviteAction, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::InviteForActionAck(msg))
                    .map_err(de::Error::custom)
            }
            (MessageTypeFamilies::InviteAction, A2AMessage::PROBLEM_REPORT) => {
                CommonProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::InviteForActionReject(msg))
                    .map_err(de::Error::custom)
            }
            (_, _) => {
                warn!("Unexpected @type field: {}", value["@type"]);
                Ok(A2AMessage::Generic(value))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MessageId(pub String);

impl MessageId {
    #[cfg(test)]
    pub fn id() -> MessageId {
        MessageId(String::from("testid"))
    }

    pub fn new() -> MessageId {
        MessageId::default()
    }
}

impl ToString for MessageId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for MessageId {
    #[cfg(all(test, not(feature = "aries")))]
    fn default() -> MessageId {
        MessageId::id()
    }

    #[cfg(any(not(test), feature = "aries"))]
    fn default() -> MessageId {
        use crate::utils::uuid;
        MessageId(uuid::uuid())
    }
}

impl A2AMessage {
    pub const FORWARD: &'static str = "forward";
    pub const CONNECTION_INVITATION: &'static str = "invitation";
    pub const CONNECTION_REQUEST: &'static str = "request";
    pub const CONNECTION_RESPONSE: &'static str = "response";
    pub const CONNECTION_PROBLEM_REPORT: &'static str = "problem_report";
    pub const PING: &'static str = "ping";
    pub const PING_RESPONSE: &'static str = "ping_response";
    pub const ACK: &'static str = "ack";
    pub const PROBLEM_REPORT: &'static str = "problem-report";
    pub const CREDENTIAL_OFFER: &'static str = "offer-credential";
    pub const CREDENTIAL: &'static str = "issue-credential";
    pub const PROPOSE_CREDENTIAL: &'static str = "propose-credential";
    pub const REQUEST_CREDENTIAL: &'static str = "request-credential";
    pub const PROPOSE_PRESENTATION: &'static str = "propose-presentation";
    pub const REQUEST_PRESENTATION: &'static str = "request-presentation";
    pub const PRESENTATION: &'static str = "presentation";
    pub const QUERY: &'static str = "query";
    pub const DISCLOSE: &'static str = "disclose";
    pub const BASIC_MESSAGE: &'static str = "message";
    pub const OUTOFBAND_INVITATION: &'static str = "invitation";
    pub const OUTOFBAND_HANDSHAKE_REUSE: &'static str = "handshake-reuse";
    pub const OUTOFBAND_HANDSHAKE_REUSE_ACCEPTED: &'static str = "handshake-reuse-accepted";
    pub const QUESTION: &'static str = "question";
    pub const ANSWER: &'static str = "answer";
    pub const INVITE_FOR_ACTION: &'static str = "invite";
}
