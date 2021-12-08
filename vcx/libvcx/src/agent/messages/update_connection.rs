use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::agent::messages::message_type::MessageTypes;
use crate::settings;
use crate::utils::httpclient;
use crate::error::prelude::*;
use crate::utils::httpclient::AgencyMock;
use crate::utils::constants::DELETE_CONNECTION_RESPONSE;
use crate::settings::protocol::ProtocolTypes;
use crate::agent::messages::{A2AMessageKinds, A2AMessage, A2AMessageV2, prepare_message_for_agent, A2AMessageV1, parse_response_from_agency, delete_connection, GeneralMessage};

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConnection {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "statusCode")]
    status_code: ConnectionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionStatus {
    AlreadyConnected,
    NotConnected,
    Deleted,
}

impl Serialize for ConnectionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            ConnectionStatus::AlreadyConnected => "CS-101",
            ConnectionStatus::NotConnected => "CS-102",
            ConnectionStatus::Deleted => "CS-103",
        };
        serde_json::Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ConnectionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("CS-101") => Ok(ConnectionStatus::AlreadyConnected),
            Some("CS-102") => Ok(ConnectionStatus::NotConnected),
            Some("CS-103") => Ok(ConnectionStatus::Deleted),
            _ => Err(de::Error::custom("Unexpected message type."))
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdateConnectionResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "statusCode")]
    status_code: ConnectionStatus,
}

#[derive(Debug)]
pub struct DeleteConnectionBuilder {
    to_did: String,
    to_vk: String,
    status_code: ConnectionStatus,
    agent_did: String,
    agent_vk: String,
    version: ProtocolTypes,
}

impl DeleteConnectionBuilder {
    pub fn create() -> DeleteConnectionBuilder {
        trace!("DeleteConnection::create_message >>>");

        DeleteConnectionBuilder {
            to_did: String::new(),
            to_vk: String::new(),
            status_code: ConnectionStatus::Deleted,
            agent_did: String::new(),
            agent_vk: String::new(),
            version: ProtocolTypes::V1,
        }
    }

    pub fn version(&mut self, version: &Option<ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<()> {
        trace!("DeleteConnection::send >>>");

        AgencyMock::set_next_response(DELETE_CONNECTION_RESPONSE);

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(&response)
    }

    fn parse_response(&self, response: &[u8]) -> VcxResult<()> {
        trace!("DeleteConnection::parse_response >>>");

        let response = parse_response_from_agency(response, &self.version)?;

        match response.first().ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "No agency responses"))? {
            A2AMessage::Version1(A2AMessageV1::UpdateConnectionResponse(_)) => Ok(()),
            A2AMessage::Version2(A2AMessageV2::UpdateConnectionResponse(_)) => Ok(()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of UpdateConnectionResponse"))
        }
    }
}

pub fn send_delete_connection_message(pw_did: &str, pw_verkey: &str, agent_did: &str, agent_vk: &str) -> VcxResult<()> {
    delete_connection()
        .to(pw_did)?
        .to_vk(pw_verkey)?
        .agent_did(agent_did)?
        .agent_vk(agent_vk)?
        .send_secure()
        .map_err(|err| err.extend("Cannot delete connection"))
}

//TODO Every GeneralMessage extension, duplicates code
impl GeneralMessage for DeleteConnectionBuilder {
    type Msg = DeleteConnectionBuilder;

    fn set_agent_did(&mut self, did: String) {
        self.agent_did = did;
    }

    fn set_agent_vk(&mut self, vk: String) {
        self.agent_vk = vk;
    }

    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }
    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>> {
        trace!("DeleteConnection::prepare_request >>>");

        let message = match self.version {
            ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::UpdateConnection(
                        UpdateConnection {
                            msg_type: MessageTypes::build(A2AMessageKinds::UpdateConnectionStatus),
                            status_code: self.status_code.clone(),
                        }
                    )
                ),
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::UpdateConnection(
                        UpdateConnection {
                            msg_type: MessageTypes::build(A2AMessageKinds::UpdateConnectionStatus),
                            status_code: self.status_code.clone(),
                        }
                    )
                )
        };

        trace!("DeleteConnection::prepare_request >>> agent: {:?}", secret!(message));

        prepare_message_for_agent(vec![message], &self.to_vk, &self.agent_did, &self.agent_vk, &self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::devsetup::SetupDefaults;

    #[test]
    fn test_deserialize_delete_connection_payload() {
        let _setup = SetupDefaults::init();

        let payload = [130, 165, 64, 116, 121, 112, 101, 130, 164, 110, 97, 109, 101, 179, 67, 79, 78, 78, 95, 83, 84, 65, 84, 85, 83, 95, 85, 80, 68, 65, 84, 69, 68, 163, 118, 101, 114, 163, 49, 46, 48, 170, 115, 116, 97, 116, 117, 115, 67, 111, 100, 101, 166, 67, 83, 45, 49, 48, 51];
        let msg_str = r#"{ "@type": { "name": "CONN_STATUS_UPDATED", "ver": "1.0" }, "statusCode": "CS-103" }"#;
        let delete_connection_payload: UpdateConnectionResponse = serde_json::from_str(&msg_str).unwrap();
        assert_eq!(delete_connection_payload, rmp_serde::from_slice(&payload).unwrap());
    }
}
