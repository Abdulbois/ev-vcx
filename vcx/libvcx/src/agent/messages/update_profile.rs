use crate::settings;
use crate::agent::messages::message_type::MessageTypes;
use crate::utils::{httpclient, validation};
use crate::utils::constants::*;
use crate::error::prelude::*;
use crate::utils::httpclient::AgencyMock;
use crate::settings::protocol::ProtocolTypes;
use crate::agent::messages::{A2AMessage, prepare_message_for_agency, A2AMessageV1, A2AMessageV2, A2AMessageKinds, parse_response_from_agency};

#[derive(Debug)]
pub struct UpdateProfileDataBuilder {
    to_did: String,
    agent_payload: String,
    configs: Vec<ConfigOption>,
    version: ProtocolTypes,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ConfigOption {
    name: String,
    value: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct UpdateConfigs {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    configs: Vec<ConfigOption>
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdateConfigsResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl UpdateProfileDataBuilder {
    pub fn create() -> UpdateProfileDataBuilder {
        trace!("UpdateProfileData::create_message >>>");

        UpdateProfileDataBuilder {
            to_did: String::new(),
            configs: Vec::new(),
            agent_payload: String::new(),
            version: ProtocolTypes::V1
        }
    }

    pub fn to(&mut self, did: &str) -> VcxResult<&mut Self> {
        validation::validate_did(did)?;
        self.to_did = did.to_string();
        Ok(self)
    }

    pub fn name(&mut self, name: &str) -> VcxResult<&mut Self> {
        let config = ConfigOption { name: "name".to_string(), value: name.to_string() };
        self.configs.push(config);
        Ok(self)
    }

    pub fn logo_url(&mut self, url: &str) -> VcxResult<&mut Self> {
        validation::validate_url(url)?;
        let config = ConfigOption { name: "logoUrl".to_string(), value: url.to_string() };
        self.configs.push(config);
        Ok(self)
    }

    pub fn use_public_did(&mut self, did: &Option<String>) -> VcxResult<&mut Self> {
        if let Some(x) = did {
            let config = ConfigOption { name: "publicDid".to_string(), value: x.to_string() };
            self.configs.push(config);
        };
        Ok(self)
    }

    pub fn version(&mut self, version: &Option<ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }


    pub fn send_secure(&mut self) -> VcxResult<()> {
        trace!("UpdateProfileData::send_secure >>>");

        AgencyMock::set_next_response(UPDATE_PROFILE_RESPONSE);

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(response)
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
        trace!("UpdateProfileData::prepare_request >>>");

        let message = match self.version {
            ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::UpdateConfigs(
                        UpdateConfigs {
                            msg_type: MessageTypes::MessageTypeV1(MessageTypes::build_v1(A2AMessageKinds::UpdateConfigs)),
                            configs: self.configs.clone()
                        }
                    )
                ),
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::UpdateConfigs(
                        UpdateConfigs {
                            msg_type: MessageTypes::MessageTypeV2(MessageTypes::build_v2(A2AMessageKinds::UpdateConfigs)),
                            configs: self.configs.clone(),
                        }
                    )
                )
        };

        trace!("UpdateProfileData::prepare_request >>> agent: {:?}", secret!(message));

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_response(&self, response: Vec<u8>) -> VcxResult<()> {
        trace!("UpdateProfileData::parse_response >>>");

        let response = parse_response_from_agency(&response, &self.version)?;

        match response.first().ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "No agency responses"))? {
            A2AMessage::Version1(A2AMessageV1::UpdateConfigsResponse(_)) => Ok(()),
            A2AMessage::Version2(A2AMessageV2::UpdateConfigsResponse(_)) => Ok(()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of UpdateConfigsResponse"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::messages::update_data;
    use crate::utils::libindy::crypto::create_and_store_my_did;
    use crate::utils::devsetup::*;

    #[test]
    fn test_update_data_post() {
        let _setup = SetupMocks::init();

        let to_did = "8XFh8yBzrpJQmNyZzgoTqB";
        let name = "name";
        let url = "https://random.com";
        let _msg = update_data()
            .to(to_did).unwrap()
            .name(&name).unwrap()
            .logo_url(&url).unwrap()
            .prepare_request().unwrap();
    }

    #[test]
    fn test_update_data_set_values_and_post() {
        let _setup = SetupLibraryWallet::init();

        let (agent_did, agent_vk) = create_and_store_my_did(Some(MY2_SEED), None).unwrap();
        let (_my_did, my_vk) = create_and_store_my_did(Some(MY1_SEED), None).unwrap();
        let (_agency_did, agency_vk) = create_and_store_my_did(Some(MY3_SEED), None).unwrap();

        settings::set_config_value(settings::CONFIG_AGENCY_VERKEY, &agency_vk);
        settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agent_vk);
        settings::set_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY, &my_vk);

        let msg = update_data()
            .to(agent_did.as_ref()).unwrap()
            .name("name").unwrap()
            .logo_url("https://random.com").unwrap()
            .prepare_request().unwrap();
        assert!(msg.len() > 0);
    }

    #[test]
    fn test_parse_update_profile_response() {
        let _setup = SetupIndyMocks::init();

        UpdateProfileDataBuilder::create().parse_response(UPDATE_PROFILE_RESPONSE.to_vec()).unwrap();
    }
}
