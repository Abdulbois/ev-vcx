use std::fmt::Debug;
use crate::agent::messages::update_message::{UIDsByConn, update_messages as update_messages_status};
use crate::agent::messages::MessageStatusCode;
use crate::agent::messages::get_message::{Message, get_connection_messages};
use crate::agent::messages::update_connection::send_delete_connection_message;

use crate::aries::messages::connection::did_doc::DidDoc;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::utils::encryption_envelope::EncryptionEnvelope;

use std::collections::HashMap;

use crate::connection::create_agent_keys;
use crate::utils::httpclient;
use crate::utils::libindy::crypto::create_and_store_my_did;
use crate::settings;
use crate::error::prelude::*;
use crate::settings::protocol::ProtocolTypes;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AgentInfo {
    pub pw_did: String,
    pub pw_vk: String,
    pub agent_did: String,
    pub agent_vk: String,
}

impl Default for AgentInfo {
    fn default() -> AgentInfo {
        AgentInfo {
            pw_did: String::new(),
            pw_vk: String::new(),
            agent_did: String::new(),
            agent_vk: String::new(),
        }
    }
}

impl AgentInfo {
    pub fn create_agent() -> VcxResult<AgentInfo> {
        trace!("Agent::create_agent >>>");
        debug!("Agent: creating pairwise agent for connection");

        let method_name = settings::get_config_value(settings::CONFIG_DID_METHOD).ok();
        let (pw_did, pw_vk) = create_and_store_my_did(None, method_name.as_ref().map(String::as_str))?;

        /*
            Create User Pairwise Agent in old way.
            Send Messages corresponding to V2 Protocol to avoid code changes on Agency side.
        */
        let (agent_did, agent_vk) = create_agent_keys("", &pw_did, &pw_vk)?;

        let agent = AgentInfo { pw_did, pw_vk, agent_did, agent_vk };

        trace!("Agent::create_agent <<< pairwise_agent: {:?}", secret!(agent));
        Ok(agent)
    }

    pub fn agency_endpoint(&self) -> VcxResult<String> {
        trace!("Agent::agency_endpoint >>>");
        debug!("Agent: Getting Agency endpoint");

        settings::get_config_value(settings::CONFIG_AGENCY_ENDPOINT)
            .map(|str| format!("{}/agency/msg", str))
    }

    pub fn routing_keys(&self) -> VcxResult<Vec<String>> {
        trace!("Agent::routing_keys >>>");
        debug!("Agent: Getting routing keys");

        let agency_vk = settings::get_config_value(settings::CONFIG_AGENCY_VERKEY)?;
        Ok(vec![self.agent_vk.to_string(), agency_vk])
    }

    pub fn recipient_keys(&self) -> Vec<String> {
        trace!("Agent::recipient_keys >>>");
        debug!("Agent: Getting recipient keys");

        vec![self.pw_vk.to_string()]
    }

    pub fn update_message_status(&self, uid: String, pw_did: Option<String>) -> VcxResult<()> {
        trace!("Agent::update_message_status_as_reviewed >>> uid: {:?}", uid);
        debug!("Agent: Updating message {:?} status on reviewed", uid);

        let messages_to_update = vec![UIDsByConn {
            pairwise_did: pw_did.unwrap_or(self.pw_did.clone()),
            uids: vec![uid],
        }];

        update_messages_status(MessageStatusCode::Reviewed, messages_to_update)?;

        trace!("Agent::update_message_status_as_reviewed <<<");
        Ok(())
    }

    pub fn get_messages(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("Agent::get_messages >>>");
        debug!("Agent: Getting all received agent from the agent");

        let messages = get_connection_messages(&self.pw_did,
                                               &self.pw_vk,
                                               &self.agent_did,
                                               &self.agent_vk,
                                               None,
                                               Some(vec![MessageStatusCode::Received]),
                                               &Some(ProtocolTypes::V2))?;


        let mut a2a_messages: HashMap<String, A2AMessage> = HashMap::new();
        for message in messages {
            match Self::decode_message(&message) {
                Ok(decoded_message) => {
                    a2a_messages.insert(message.uid.clone(), decoded_message);
                },
                Err(err) => {
                    warn!("Unable to decode received message! Err: {:?}", err);
                    warn!("Ignore and updating message status as read");
                    self.update_message_status(message.uid.clone(), None)?;

                }
            }
        }

        trace!("Agent::get_messages <<< a2a_messages: {:?}", secret!(a2a_messages));
        Ok(a2a_messages)
    }

    pub fn get_message_by_id(&self, msg_id: &str) -> VcxResult<A2AMessage> {
        trace!("Agent::get_message_by_id >>> msg_id: {:?}", msg_id);
        debug!("Agent: Getting message by id {}", msg_id);

        let mut messages = get_connection_messages(&self.pw_did,
                                                   &self.pw_vk,
                                                   &self.agent_did,
                                                   &self.agent_vk,
                                                   Some(vec![msg_id.to_string()]),
                                                   None,
                                                   &Some(ProtocolTypes::V2))?;

        let message =
            messages
                .pop()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Message not found for id: {:?}", msg_id)))?;

        let message = Self::decode_message(&message)?;

        trace!("Agent::get_message_by_id <<< message: {:?}", secret!(message));
        Ok(message)
    }

    pub fn decode_message(message: &Message) -> VcxResult<A2AMessage> {
        trace!("Agent::decode_message >>> message: {:?}", secret!(message));
        debug!("Agent: Decoding received message");

        let message = match message.decrypted_payload {
            Some(ref payload) => {
                debug!("Agent: Message Payload is already decoded");

                let message: crate::agent::messages::payload::PayloadV1 = ::serde_json::from_str(&payload)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot deserialize message: {}", err)))?;

                ::serde_json::from_str::<A2AMessage>(&message.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Cannot deserialize A2A message: {}", err)))
            }
            None => EncryptionEnvelope::open(message.payload()?)
        }?;

        trace!("Agent::decode_message <<< message: {:?}", secret!(message));
        Ok(message)
    }

    pub fn send_message<T: Serialize + Debug>(&self, message: &T, did_doc: &DidDoc) -> VcxResult<()> {
        trace!("Agent::send_message >>> message: {:?}, did_doc: {:?}", secret!(message), secret!(did_doc));
        debug!("Agent: Sending message on the remote endpoint");

        let pw_key = if self.pw_vk.is_empty() { None} else {Some(self.pw_vk.clone())};
        let envelope = EncryptionEnvelope::create(&message, pw_key.as_ref().map(String::as_str), &did_doc)?;
        httpclient::post_message(&envelope.0, &did_doc.get_endpoint())?;
        trace!("Agent::send_message <<<");
        Ok(())
    }

    pub fn send_message_and_wait_result<T: Serialize + Debug>(message: &T, did_doc: &DidDoc) -> VcxResult<A2AMessage> {
        trace!("Agent::send_message_and_wait_result >>> message: {:?}, did_doc: {:?}",
               secret!(message), secret!(did_doc));
        debug!("Agent: Sending message on the remote endpoint and wait for result");

        let (_, sender_vk) = create_and_store_my_did(None, None)?;
        let envelope = EncryptionEnvelope::create(&message, Some(&sender_vk), &did_doc)?;
        let response = httpclient::post_message(&envelope.0, &did_doc.get_endpoint())?;
        let message = EncryptionEnvelope::open(response)?;

        trace!("Agent::send_message_and_wait_result <<< message: {:?}", secret!(message));
        Ok(message)
    }

    pub fn send_message_anonymously<T: Serialize + Debug>(message: &T, did_dod: &DidDoc) -> VcxResult<()> {
        trace!("Agent::send_message_anonymously >>> message: {:?}, did_doc: {:?}", secret!(message), secret!(did_dod));
        debug!("Agent: Sending message on the remote anonymous endpoint");

        let envelope = EncryptionEnvelope::create(&message, None, &did_dod)?;
        httpclient::post_message(&envelope.0, &did_dod.get_endpoint())?;
        trace!("Agent::send_message_anonymously <<<");
        Ok(())
    }

    pub fn delete(&self) -> VcxResult<()> {
        trace!("Agent::delete >>>");
        debug!("Agent: deleting");

        send_delete_connection_message(&self.pw_did, &self.pw_vk, &self.agent_did, &self.agent_vk)?;
        trace!("Agent::delete <<<");
        Ok(())
    }
}
