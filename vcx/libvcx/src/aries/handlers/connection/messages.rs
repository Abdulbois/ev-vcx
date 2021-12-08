use crate::aries::messages::connection::invite::Invitation;
use crate::aries::messages::connection::request::Request;
use crate::aries::messages::connection::problem_report::ProblemReport;
use crate::aries::messages::trust_ping::ping::Ping;
use crate::aries::messages::trust_ping::ping_response::PingResponse;
use crate::aries::messages::ack::Ack;
use crate::aries::messages::discovery::query::Query;
use crate::aries::messages::discovery::disclose::Disclose;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::outofband::invitation::Invitation as OutofbandInvitation;
use crate::aries::messages::outofband::handshake_reuse::HandshakeReuse;
use crate::aries::messages::outofband::handshake_reuse_accepted::HandshakeReuseAccepted;
use crate::aries::messages::questionanswer::question::{Question, QuestionResponse};
use crate::aries::messages::questionanswer::answer::Answer;
use crate::aries::messages::committedanswer::question::{Question as CommittedQuestion, QuestionResponse as CommittedQuestionResponse};
use crate::aries::messages::committedanswer::answer::Answer as CommitedAnswer;
use crate::aries::messages::invite_action::invite::Invite as InviteForAction;
use crate::connection::ConnectionOptions;
use crate::aries::messages::connection::response::SignedResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DidExchangeMessages {
    Connect(ConnectionOptions),
    InvitationReceived(Invitation),
    ExchangeRequestReceived(Request),
    ExchangeResponseReceived(SignedResponse),
    AckReceived(Ack),
    ProblemReportReceived(ProblemReport),
    SendPing(Option<String>),
    PingReceived(Ping),
    PingResponseReceived(PingResponse),
    DiscoverFeatures((Option<String>, Option<String>)),
    QueryReceived(Query),
    OutofbandInvitationReceived(OutofbandInvitation),
    SendHandshakeReuse(OutofbandInvitation),
    HandshakeReuseReceived(HandshakeReuse),
    HandshakeReuseAcceptedReceived(HandshakeReuseAccepted),
    DiscloseReceived(Disclose),
    QuestionReceived(Question),
    AnswerReceived(Answer),
    SendAnswer((Question, QuestionResponse)),
    SendCommittedAnswer((CommittedQuestion, CommittedQuestionResponse)),
    CommittedQuestionReceived(CommittedQuestion),
    CommittedAnswerReceived(CommitedAnswer),
    SendInviteAction(InviteForAction),
    InviteActionReceived(InviteForAction),
    Unknown
}

impl From<A2AMessage> for DidExchangeMessages {
    fn from(msg: A2AMessage) -> Self {
        match msg {
            A2AMessage::ConnectionInvitation(invite) => {
                DidExchangeMessages::InvitationReceived(invite)
            }
            A2AMessage::ConnectionRequest(request) => {
                DidExchangeMessages::ExchangeRequestReceived(request)
            }
            A2AMessage::ConnectionResponse(request) => {
                DidExchangeMessages::ExchangeResponseReceived(request)
            }
            A2AMessage::Ping(ping) => {
                DidExchangeMessages::PingReceived(ping)
            }
            A2AMessage::PingResponse(ping_response) => {
                DidExchangeMessages::PingResponseReceived(ping_response)
            }
            A2AMessage::Ack(ack) => {
                DidExchangeMessages::AckReceived(ack)
            }
            A2AMessage::Query(query) => {
                DidExchangeMessages::QueryReceived(query)
            }
            A2AMessage::Disclose(disclose) => {
                DidExchangeMessages::DiscloseReceived(disclose)
            }
            A2AMessage::HandshakeReuse(handshake_reuse) => {
                DidExchangeMessages::HandshakeReuseReceived(handshake_reuse)
            }
            A2AMessage::HandshakeReuseAccepted(handshake_reuse_accepted) => {
                DidExchangeMessages::HandshakeReuseAcceptedReceived(handshake_reuse_accepted)
            }
            A2AMessage::ConnectionProblemReport(report) => {
                DidExchangeMessages::ProblemReportReceived(report)
            }
            A2AMessage::Question(question) => {
                DidExchangeMessages::QuestionReceived(question)
            }
            A2AMessage::Answer(answer) => {
                DidExchangeMessages::AnswerReceived(answer)
            }
            A2AMessage::CommittedQuestion(question) => {
                DidExchangeMessages::CommittedQuestionReceived(question)
            }
            A2AMessage::CommittedAnswer(answer) => {
                DidExchangeMessages::CommittedAnswerReceived(answer)
            }
            A2AMessage::InviteForAction(invite) => {
                DidExchangeMessages::InviteActionReceived(invite)
            }
            _ => {
                DidExchangeMessages::Unknown
            }
        }
    }
}
