use crate::settings;
use crate::agent::messages::message_type::MessageTypes;
use crate::utils::{httpclient, constants};
use crate::error::prelude::*;
use crate::utils::httpclient::AgencyMock;
use crate::settings::protocol::ProtocolTypes;
use crate::agent::messages::{A2AMessage, A2AMessageV2, A2AMessageKinds, prepare_message_for_agency, A2AMessageV1, parse_response_from_agency, MessageStatusCode};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageStatusByConnections {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    status_code: Option<MessageStatusCode>,
    uids_by_conns: Vec<UIDsByConn>
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMessageStatusByConnectionsResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    status_code: Option<String>,
    updated_uids_by_conns: Vec<UIDsByConn>
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UIDsByConn {
    #[serde(rename = "pairwiseDID")]
    pub pairwise_did: String,
    pub uids: Vec<String>,
}

struct UpdateMessageStatusByConnectionsBuilder {
    status_code: Option<MessageStatusCode>,
    uids_by_conns: Vec<UIDsByConn>,
    version: ProtocolTypes,
}

impl UpdateMessageStatusByConnectionsBuilder {
    pub fn create() -> UpdateMessageStatusByConnectionsBuilder {
        trace!("UpdateMessageStatusByConnectionsBuilder::create >>>");

        UpdateMessageStatusByConnectionsBuilder {
            status_code: None,
            uids_by_conns: Vec::new(),
            version: settings::get_protocol_type()
        }
    }

    pub fn uids_by_conns(&mut self, uids_by_conns: Vec<UIDsByConn>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.uids_by_conns = uids_by_conns;
        Ok(self)
    }

    pub fn status_code(&mut self, code: MessageStatusCode) -> VcxResult<&mut Self> {
        //Todo: validate that it can be parsed to number??
        self.status_code = Some(code.clone());
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn version(&mut self, version: &Option<ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<()> {
        trace!("UpdateMessages::send >>>");

        AgencyMock::set_next_response(constants::UPDATE_MESSAGES_RESPONSE);

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(&response)
    }

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>> {
        trace!("UpdateMessageStatusByConnections::prepare_request >>>");

        let message = match self.version {
            ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::UpdateMessageStatusByConnections(
                        UpdateMessageStatusByConnections {
                            msg_type: MessageTypes::build(A2AMessageKinds::UpdateMessageStatusByConnections),
                            uids_by_conns: self.uids_by_conns.clone(),
                            status_code: self.status_code.clone(),
                        }
                    )
                ),
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::UpdateMessageStatusByConnections(
                        UpdateMessageStatusByConnections {
                            msg_type: MessageTypes::build(A2AMessageKinds::UpdateMessageStatusByConnections),
                            uids_by_conns: self.uids_by_conns.clone(),
                            status_code: self.status_code.clone(),
                        }
                    )
                ),
        };

        trace!("UpdateMessageStatusByConnections::prepare_request >>> agent: {:?}", secret!(message));

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;
        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_response(&self, response: &[u8]) -> VcxResult<()> {
        trace!("UpdateMessageStatusByConnections::parse_response >>>");

        let response = parse_response_from_agency(response, &self.version)?;

        match response.first().ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "No agency responses"))? {
            A2AMessage::Version1(A2AMessageV1::UpdateMessageStatusByConnectionsResponse(_)) => Ok(()),
            A2AMessage::Version2(A2AMessageV2::UpdateMessageStatusByConnectionsResponse(_)) => Ok(()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of UpdateMessageStatusByConnectionsResponse"))
        }
    }
}

pub fn update_agency_messages(status_code: &str, msg_json: &str) -> VcxResult<()> {
    trace!("update_agency_messages >>> status_code: {:?}, msg_json: {:?}", status_code, secret!(msg_json));
    debug!("Agency: Updating agent {:?} status", secret!(msg_json));

    let status_code: MessageStatusCode = ::serde_json::from_str(&format!("\"{}\"", status_code))
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize `status_code`: {}", err)))?;

    let uids_by_conns: Vec<UIDsByConn> = serde_json::from_str(msg_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize list of agent to update: {}", err)))?;

    update_messages(status_code, uids_by_conns)?;

    debug!("Agency: Messages updated");
    trace!("update_agency_messages <<<");
    Ok(())
}

pub fn update_messages(status_code: MessageStatusCode, uids_by_conns: Vec<UIDsByConn>) -> VcxResult<()> {
    trace!("update_messages >>> status_code {:?}, uids_by_conns {:?}", status_code, secret!(uids_by_conns));

    UpdateMessageStatusByConnectionsBuilder::create()
        .uids_by_conns(uids_by_conns)?
        .status_code(status_code)?
        .send_secure()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::devsetup::*;
    #[test]
    fn test_parse_parse_update_messages_response() {
        let _setup = SetupMocks::init();

        UpdateMessageStatusByConnectionsBuilder::create().parse_response(crate::utils::constants::UPDATE_MESSAGES_RESPONSE).unwrap();
    }

    #[cfg(all(feature = "agency", feature = "pool_tests"))]
    #[test]
    fn test_update_agency_messages() {
        use std::thread;
        use std::time::Duration;
        let _setup = SetupLibraryAgencyV2NewProvisioning::init();

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (_faber, alice) = crate::connection::tests::create_connected_connections();

        let (_, cred_def_handle) = crate::credential_def::tests::create_cred_def_real(false);

        let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
        let credential_offer = crate::issuer_credential::issuer_credential_create(cred_def_handle,
                                                                             "1".to_string(),
                                                                             institution_did.clone(),
                                                                             "credential_name".to_string(),
                                                                             credential_data.to_owned(),
                                                                             1).unwrap();

        credential_offer.send_credential_offer(alice).unwrap();
        thread::sleep(Duration::from_millis(2000));
        // AS CONSUMER GET MESSAGES
        crate::utils::devsetup::set_consumer();
        let pending = crate::agent::messages::get_message::download_messages(None, Some(vec!["MS-103".to_string()]), None).unwrap();
        assert!(pending.len() > 0);
        let did = pending[0].pairwise_did.clone();
        let uid = pending[0].msgs[0].uid.clone();
        let message = serde_json::to_string(&[UIDsByConn { pairwise_did: did, uids: vec![uid] }]).unwrap();
        update_agency_messages("MS-106", &message).unwrap();
        let updated = crate::agent::messages::get_message::download_messages(None, Some(vec!["MS-106".to_string()]), None).unwrap();
        assert_eq!(pending[0].msgs[0].uid, updated[0].msgs[0].uid);
    }
}
