pub mod agent;
pub mod connection_fsm;
pub mod messages;
pub mod types;
pub mod states;

use std::collections::HashMap;
use core::fmt::Debug;

use crate::error::prelude::*;
use crate::aries::handlers::connection::connection_fsm::{Actor, DidExchangeSM};
use crate::aries::handlers::connection::messages::DidExchangeMessages;
use crate::aries::handlers::connection::states::ActorDidExchangeState;
use crate::aries::handlers::connection::agent::AgentInfo;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::connection::invite::Invitation;
use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::basic_message::message::BasicMessage;
use crate::aries::handlers::connection::types::{SideConnectionInfo, PairwiseConnectionInfo, CompletedConnection, OutofbandMeta, Invitations};
use crate::aries::messages::outofband::invitation::Invitation as OutofbandInvitation;
use crate::aries::messages::questionanswer::question::{Question, QuestionResponse};
use crate::aries::messages::committedanswer::question::{Question as CommittedQuestion, QuestionResponse as CommittedQuestionResponse};
use crate::aries::messages::invite_action::invite::InviteActionData;
use crate::aries::messages::invite_action::invite::Invite as InviteForAction;
use crate::connection::ConnectionOptions;
use crate::aries::messages::connection::problem_report::ProblemReport;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    connection_sm: DidExchangeSM
}

impl Connection {
    pub fn create(source_id: &str) -> Connection {
        trace!("Connection::create >>> source_id: {}", source_id);
        debug!("Connection {}: Creating Connection state object", source_id);

        Connection {
            connection_sm: DidExchangeSM::new(Actor::Inviter, source_id, None),
        }
    }

    pub fn create_outofband(source_id: &str, goal_code: Option<String>, goal: Option<String>,
                            handshake: bool, request_attach: Option<String>) -> Connection {
        trace!("create_outofband_connection >>> source_id: {}, goal_code: {:?}, goal: {:?}, handshake: {}, request_attach: {:?}",
               source_id, secret!(goal_code), secret!(goal), secret!(handshake), secret!(request_attach));
        debug!("Connection {}: Creating out-of-band Connection state object", source_id);

        let meta = OutofbandMeta::new(goal_code, goal, handshake, request_attach);

        Connection {
            connection_sm: DidExchangeSM::new(Actor::Inviter, source_id, Some(meta)),
        }
    }

    pub fn from_parts(source_id: String, agent_info: AgentInfo, state: ActorDidExchangeState) -> Connection {
        Connection { connection_sm: DidExchangeSM::from(source_id, agent_info, state) }
    }

    pub fn create_with_invite(source_id: &str, invitation: Invitation) -> VcxResult<Connection> {
        trace!("Connection::create_with_invite >>> source_id: {}, invitation: {:?}", source_id, secret!(invitation));
        debug!("Connection {}: Creating Connection state object with invite", source_id);

        let mut connection = Connection {
            connection_sm: DidExchangeSM::new(Actor::Invitee, source_id, None),
        };

        connection.process_invite(invitation)?;

        Ok(connection)
    }

    pub fn create_with_outofband_invite(source_id: &str, mut invitation: OutofbandInvitation) -> VcxResult<Connection> {
        trace!("Connection::create_with_outofband_invite >>> source_id: {}, invitation: {:?}", source_id, secret!(invitation));
        debug!("Connection {}: Creating Connection state object with out-of-band invite", source_id);

        invitation.validate()?;

        // normalize service keys in case invitation is using did:key format
        invitation.normalize_service_keys()?;

        let mut connection = Connection {
            connection_sm: DidExchangeSM::new(Actor::Invitee, source_id, None),
        };

        connection.process_outofband_invite(invitation)?;

        Ok(connection)
    }

    pub fn source_id(&self) -> String { self.connection_sm.source_id().to_string() }

    pub fn state(&self) -> u32 { self.connection_sm.state() }

    pub fn agent_info(&self) -> &AgentInfo { self.connection_sm.agent_info() }

    pub fn remote_did(&self) -> VcxResult<String> {
        self.connection_sm.remote_did()
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        self.connection_sm.remote_vk()
    }

    pub fn state_object<'a>(&'a self) -> &'a ActorDidExchangeState {
        &self.connection_sm.state_object()
    }

    pub fn get_source_id(&self) -> String {
        self.connection_sm.source_id().to_string()
    }

    pub fn process_invite(&mut self, invitation: Invitation) -> VcxResult<()> {
        trace!("Connection::process_invite >>> invitation: {:?}", secret!(invitation));
        self.step(DidExchangeMessages::InvitationReceived(invitation))
    }

    pub fn process_outofband_invite(&mut self, invitation: OutofbandInvitation) -> VcxResult<()> {
        trace!("Connection::process_outofband_invite >>> invitation: {:?}", secret!(invitation));
        self.step(DidExchangeMessages::OutofbandInvitationReceived(invitation))
    }

    pub fn get_invitation(&self) -> Option<Invitations> {
        trace!("Connection::get_invite >>>");
        return self.connection_sm.get_invitation();
    }

    pub fn get_invite_details(&self) -> VcxResult<String> {
        trace!("Connection::get_invite_details >>>");
        debug!("Connection {}: Getting invitation", self.source_id());

        let invitation = match self.get_invitation() {
            Some(invitation) => match invitation {
                Invitations::ConnectionInvitation(invitation_) => {
                    json!(invitation_).to_string()
                }
                Invitations::OutofbandInvitation(invitation_) => {
                    json!(invitation_).to_string()
                }
            },
            None => json!({}).to_string()
        };

        return Ok(invitation);
    }

    pub fn connect(&mut self, options: ConnectionOptions) -> VcxResult<()> {
        trace!("Connection::connect >>> source_id: {}", self.connection_sm.source_id());
        debug!("Connection {}: Starting connection establishing process", self.source_id());

        self.step(DidExchangeMessages::Connect(options))
    }

    pub fn update_state(&mut self, message: Option<&str>) -> VcxResult<u32> {
        trace!("Connection::update_state >>> message: {:?}", secret!(message));
        debug!("Connection {}: Updating state", self.source_id());

        if let Some(message_) = message {
            return self.update_state_with_message(message_);
        }

        let messages = self.get_messages()?;
        let pw_did = self.agent_info().pw_did.clone();

        if let Some((uid, message)) = self.connection_sm.find_message_to_handle(messages) {
            self.handle_message(message.into())?;
            self.agent_info().update_message_status(uid, Some(pw_did))?;
        } else {
            if let Some(prev_agent_info) = self.connection_sm.prev_agent_info().cloned() {
                let messages = prev_agent_info.get_messages()?;

                if let Some((uid, message)) = self.connection_sm.find_message_to_handle(messages) {
                    self.handle_message(message.into())?;
                    prev_agent_info.update_message_status(uid, Some(pw_did))?;
                }
            }
        };

        let state = self.state();

        trace!("Connection::update_state <<< state: {:?}", state);
        Ok(state)
    }

    pub fn update_message_status(&self, uid: String) -> VcxResult<()> {
        trace!("Connection::update_message_status >>> uid: {:?}", uid);
        debug!("Connection {}: Updating message status as reviewed", self.source_id());

        self.connection_sm.agent_info().update_message_status(uid, None)
    }

    pub fn update_state_with_message(&mut self, message: &str) -> VcxResult<u32> {
        trace!("Connection: update_state_with_message: {}", secret!(message));
        debug!("Connection {}: Updating state with message", self.source_id());

        let message: A2AMessage = ::serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Cannot updated Connection state with agent: Message deserialization failed with: {:?}", err)))?;

        self.handle_message(message.into())?;

        let state = self.state();

        trace!("Connection: update_state_with_message: <<< state: {}", state);

        Ok(state)
    }

    pub fn get_messages(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("Connection: get_messages >>>");
        debug!("Connection {}: Getting agent", self.source_id());
        self.agent_info().get_messages()
    }

    pub fn get_message_by_id(&self, msg_id: &str) -> VcxResult<A2AMessage> {
        trace!("Connection: get_message_by_id >>>");
        debug!("Connection {}: Getting message by id {:?}", self.source_id(), msg_id);

        self.agent_info().get_message_by_id(msg_id)
    }

    pub fn handle_message(&mut self, message: DidExchangeMessages) -> VcxResult<()> {
        trace!("Connection: handle_message >>> {:?}", secret!(message));
        self.step(message)
    }

    pub fn send_message<T: Serialize + Debug>(&self, message: &T) -> VcxResult<()> {
        trace!("Connection::send_message >>> message: {:?}", secret!(message));
        debug!("Connection {}: Sending message", self.source_id());

        let did_doc = self.connection_sm.did_doc()
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot send message: Remote Connection DIDDoc is not set"))?;

        self.agent_info().send_message(message, &did_doc)
    }

    pub fn send_message_and_wait_result<T: Serialize + Debug>(message: &T, did_doc: &DidDoc) -> VcxResult<A2AMessage> {
        trace!("Connection::send_message_and_wait_result >>> message: {:?}, did_doc: {:?}",
               secret!(message), secret!(did_doc));

        AgentInfo::send_message_and_wait_result(message, did_doc)
    }

    pub fn send_message_to_self_endpoint<T: Serialize + Debug>(message: &T, did_doc: &DidDoc) -> VcxResult<()> {
        trace!("Connection::send_message_to_self_endpoint >>> message: {:?}, did_doc: {:?}", secret!(message), secret!(did_doc));

        AgentInfo::send_message_anonymously(message, did_doc)
    }

    fn parse_generic_message(message: &str, _message_options: &str) -> A2AMessage {
        match ::serde_json::from_str::<A2AMessage>(message) {
            Ok(a2a_message) => a2a_message,
            Err(_) => {
                A2AMessage::BasicMessage(
                    BasicMessage::create()
                        .set_content(message.to_string())
                        .set_time()
                )
            }
        }
    }

    pub fn send_generic_message(&self, message: &str, _message_options: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", secret!(message));
        debug!("Connection {}: Sending generic message", self.source_id());

        let message = Connection::parse_generic_message(message, _message_options);
        let message = match message {
            A2AMessage::Generic(message_) => message_,
            message => json!(message)
        };
        self.send_message(&message).map(|_| String::new())
    }

    pub fn send_ping(&mut self, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_ping >>> comment: {:?}", secret!(comment));
        debug!("Connection {}: Sending ping message", self.source_id());

        self.handle_message(DidExchangeMessages::SendPing(comment))
    }

    pub fn delete(&self) -> VcxResult<()> {
        trace!("Connection: delete >>> {:?}", self.connection_sm.source_id());
        self.agent_info().delete()
    }

    fn step(&mut self, message: DidExchangeMessages) -> VcxResult<()> {
        self.connection_sm = self.connection_sm.clone().step(message)?;
        Ok(())
    }

    pub fn send_discovery_features(&mut self, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_discovery_features_query >>> query: {:?}, comment: {:?}", secret!(query), secret!(comment));
        debug!("Connection {}: Sending discovery features message", self.source_id());

        self.handle_message(DidExchangeMessages::DiscoverFeatures((query, comment)))
    }

    pub fn send_reuse(&mut self, invitation: OutofbandInvitation) -> VcxResult<()> {
        trace!("Connection::send_reuse >>> invitation: {:?}", secret!(invitation));
        debug!("Connection {}: Sending reuse message", self.source_id());

        self.handle_message(DidExchangeMessages::SendHandshakeReuse(invitation))
    }

    pub fn send_answer(&mut self, question: String, response: String) -> VcxResult<()> {
        trace!("Connection::send_answer >>> question: {:?}, response: {:?}", secret!(question), secret!(response));
        debug!("Connection {}: Sending question answer message", self.source_id());

        let parsed_question = ::serde_json::from_str::<Question>(&question);
        let parser_response = ::serde_json::from_str::<QuestionResponse>(&response);

        if let (Ok(question_), Ok(response_)) = (parsed_question, parser_response) {
            self.handle_message(DidExchangeMessages::SendAnswer((question_, response_)))?;
            return Ok(());
        }

        let parsed_question = ::serde_json::from_str::<CommittedQuestion>(&question);
        let parser_response = ::serde_json::from_str::<CommittedQuestionResponse>(&response);

        match (parsed_question, parser_response) {
            (Ok(question_), Ok(response_)) => {
                self.handle_message(DidExchangeMessages::SendCommittedAnswer((question_, response_)))
            }
            (Err(err), _) => {
                Err(VcxError::from_msg(VcxErrorKind::InvalidJson,
                                       format!("Could not parse Question from message: {:?}. Err: {:?}",
                                               question, err)))
            }
            (_, Err(err)) => {
                Err(VcxError::from_msg(VcxErrorKind::InvalidJson,
                                       format!("Could not parse Question Response from message: {:?}. Err: {:?}",
                                               question, err)))
            }
        }
    }

    pub fn send_invite_action(&mut self, data: InviteActionData) -> VcxResult<String> {
        trace!("Connection::send_invite_action >>> data: {:?}", secret!(data));
        debug!("Connection {}: Sending invitation for taking an action", self.source_id());

        let invite = InviteForAction::create()
            .set_goal_code(data.goal_code)
            .set_ack_on(data.ack_on);

        let invite_json = json!(invite).to_string();

        self.handle_message(DidExchangeMessages::SendInviteAction(invite))?;

        Ok(invite_json)
    }

    pub fn get_connection_info(&self) -> VcxResult<String> {
        trace!("Connection::get_connection_info >>>");
        debug!("Connection {}: Getting information", self.source_id());

        let agent_info = self.agent_info().clone();

        let current = SideConnectionInfo {
            did: agent_info.pw_did.clone(),
            recipient_keys: agent_info.recipient_keys().clone(),
            routing_keys: agent_info.routing_keys()?,
            service_endpoint: agent_info.agency_endpoint()?,
            protocols: Some(self.connection_sm.get_protocols()),
        };

        let remote = match self.connection_sm.did_doc() {
            Some(did_doc) =>
                Some(SideConnectionInfo {
                    did: did_doc.id.clone(),
                    recipient_keys: did_doc.recipient_keys(),
                    routing_keys: did_doc.routing_keys(),
                    service_endpoint: did_doc.get_endpoint(),
                    protocols: self.connection_sm.get_remote_protocols(),
                }),
            None => None
        };

        let connection_info = PairwiseConnectionInfo {
            my: current,
            their: remote,
            invitation: self.get_invitation(),
        };

        return Ok(json!(connection_info).to_string());
    }

    pub fn get_completed_connection(&self) -> VcxResult<CompletedConnection> {
        self.connection_sm.completed_connection()
            .ok_or(VcxError::from_msg(VcxErrorKind::ConnectionNotCompleted,
                                      format!("Connection object {} in state {} not ready to send remote agent", self.connection_sm.source_id(), self.state())))
    }

    pub fn get_problem_report_message(&self) -> VcxResult<String> {
        trace!("Connection::get_problem_report_message >>>");
        debug!("Connection {}: Getting problem report message", self.get_source_id());

        let problem_report: Option<&ProblemReport> = self.connection_sm.problem_report();
        Ok(json!(&problem_report).to_string())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::aries::messages::a2a::A2AMessage;
    use crate::aries::handlers::connection::Connection;
    use crate::aries::messages::connection::invite::Invitation;
    use crate::aries::messages::connection::response::Response;
    use crate::aries::messages::connection::did_doc::tests::_service_endpoint;
    use crate::aries::messages::connection::request::tests::_request;
    use crate::connection::Connections;
    use crate::utils::object_cache::Handle;
    use crate::settings;

    #[test]
    fn test_parse_generic_message_plain_string_should_be_parsed_as_basic_msg() -> Result<(), String> {
        let message = "Some plain text message";
        let result = Connection::parse_generic_message(message, "");
        match result {
            A2AMessage::BasicMessage(basic_msg) => {
                assert_eq!(basic_msg.content, message);
                Ok(())
            }
            other => Err(format!("Result is not BasicMessage, but: {:?}", other))
        }
    }

    #[test]
    fn test_parse_generic_message_json_msg_should_be_parsed_as_generic() -> Result<(), String> {
        let message = json!({
            "@id": "some id",
            "@type": "some type",
            "content": "some content"
        }).to_string();
        let result = Connection::parse_generic_message(&message, "");
        match result {
            A2AMessage::Generic(value) => {
                assert_eq!(value.to_string(), message);
                Ok(())
            }
            other => Err(format!("Result is not Generic, but: {:?}", other))
        }
    }

    pub fn mock_connection() -> Handle<Connections> {
        let key = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL".to_string();
        let invitation =
            Invitation::default()
                .set_recipient_keys(vec![key.clone()]);

        let connection_handle = crate::connection::create_connection_with_invite("source_id", &json!(invitation).to_string()).unwrap();

        connection_handle.connect(None).unwrap();

        let response =
            Response::default()
                .set_service_endpoint(_service_endpoint())
                .set_keys(vec![key.to_string()], vec![])
                .set_thread_id(&_request().id.0)
                .encode(&key).unwrap();
        connection_handle.update_state(Some(json!(response).to_string())).unwrap();

        connection_handle
    }

    fn _setup() {
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "3.0");
    }

    fn _source_id() -> &'static str {
        "test connection"
    }

    #[cfg(feature = "aries")]
    mod aries {
        use super::*;

        use crate::aries::test::{Faber, Alice};
        use crate::aries::messages::ack::tests::_ack;
        use crate::aries::messages::a2a::A2AMessage;
        use crate::aries::messages::connection::invite::tests::_invitation_json;

        #[test]
        fn test_create_connection_works() {
            _setup();
            let connection_handle = crate::connection::create_connection(_source_id()).unwrap();
            assert!(connection_handle.is_valid_handle());
            assert_eq!(1, connection_handle.get_state());
        }

        #[test]
        fn test_create_connection_with_invite_works() {
            _setup();
            let connection_handle = crate::connection::create_connection_with_invite(_source_id(), &_invitation_json()).unwrap();
            assert!(connection_handle.is_valid_handle());
            assert_eq!(2, connection_handle.get_state());
        }

        #[test]
        fn test_get_connection_state_works() {
            _setup();
            let connection_handle = crate::connection::create_connection(_source_id()).unwrap();
            assert_eq!(1, connection_handle.get_state());
        }

        #[test]
        fn test_connection_send_works() {
            _setup();
            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            let invite = faber.create_invite();
            alice.accept_invite(&invite);

            faber.update_state(3);
            alice.update_state(4);
            faber.update_state(4);

            let uid: String;
            let message = _ack();

            // Send Message works
            {
                faber.send_message(&message);
            }

            {
                // Get Messages works
                alice.activate();

                let messages = alice.connection_handle.get_messages().unwrap();
                assert_eq!(1, messages.len());

                uid = messages.keys().next().unwrap().clone();
                let received_message = messages.values().next().unwrap().clone();

                match received_message {
                    A2AMessage::Ack(received_message) => assert_eq!(message, received_message.clone()),
                    _ => assert!(false)
                }
            }

            let _res = crate::agent::messages::get_message::download_messages(None, None, Some(vec![uid.clone()])).unwrap();

            // Get Message by id works
            {
                alice.activate();

                let message = alice.connection_handle.get_message_by_id(uid.clone()).unwrap();

                match message {
                    A2AMessage::Ack(ack) => assert_eq!(_ack(), ack),
                    _ => assert!(false)
                }
            }

            // Update Message Status works
            {
                alice.activate();
                alice.update_message_status(uid);
                let messages = alice.connection_handle.get_messages().unwrap();
                assert_eq!(0, messages.len());
            }

            // Send Basic Message works
            {
                faber.activate();

                let basic_message = r#"Hi there"#;
                faber.connection_handle.send_generic_message(basic_message, "").unwrap();

                alice.activate();

                let messages = alice.connection_handle.get_messages().unwrap();
                assert_eq!(1, messages.len());

                let uid = messages.keys().next().unwrap().clone();
                let message = messages.values().next().unwrap().clone();

                match message {
                    A2AMessage::BasicMessage(message) => assert_eq!(basic_message, message.content),
                    _ => assert!(false)
                }
                alice.update_message_status(uid);
            }

            // Download Messages
            {
                use crate::agent::messages::get_message::{download_messages, MessageByConnection, Message};

                let credential_offer = crate::aries::messages::issuance::v10::credential_offer::tests::_credential_offer();

                faber.send_message(&credential_offer);

                alice.activate();

                let messages: Vec<MessageByConnection> = download_messages(None, Some(vec!["MS-103".to_string()]), None).unwrap();
                let message: Message = messages[0].msgs[0].clone();
                assert_eq!(crate::agent::messages::RemoteMessageType::Other("aries".to_string()), message.msg_type);
                let payload: crate::agent::messages::payload::PayloadV1 = ::serde_json::from_str(&message.decrypted_payload.unwrap()).unwrap();
                let _payload: Vec<crate::legacy::messages::issuance::credential_offer::CredentialOffer> = ::serde_json::from_str(&payload.msg).unwrap();

                alice.update_message_status(message.uid);
            }

            // Helpers
            {
                faber.activate();

                faber.connection_handle.get_pw_did().unwrap();
                faber.connection_handle.get_pw_verkey().unwrap();
                faber.connection_handle.get_their_pw_verkey().unwrap();
                faber.connection_handle.get_source_id().unwrap();
            }
        }
    }
}

