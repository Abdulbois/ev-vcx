use crate::settings;
use crate::utils::{httpclient, constants, validation};
use crate::error::prelude::*;
use crate::utils::httpclient::AgencyMock;
use crate::settings::protocol::ProtocolTypes;
use crate::agent::messages::{A2AMessage, A2AMessageV1, A2AMessageKinds, A2AMessageV2, prepare_message_for_agency, parse_response_from_agency};
use crate::agent::messages::message_type::MessageTypes;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateKey {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "forDID")]
    for_did: String,
    #[serde(rename = "forDIDVerKey")]
    for_verkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    for_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    for_verkey: String,
}

#[derive(Debug)]
pub struct CreateKeyBuilder {
    for_did: String,
    for_verkey: String,
    version: ProtocolTypes,
}

impl CreateKeyBuilder {
    pub fn create() -> CreateKeyBuilder {
        trace!("CreateKeyBuilder::create_message >>>");

        CreateKeyBuilder {
            for_did: String::new(),
            for_verkey: String::new(),
            version: settings::get_protocol_type(),
        }
    }

    pub fn for_did(&mut self, did: &str) -> VcxResult<&mut Self> {
        validation::validate_did(did)?;
        self.for_did = did.to_string();
        Ok(self)
    }

    pub fn for_verkey(&mut self, verkey: &str) -> VcxResult<&mut Self> {
        validation::validate_verkey(verkey)?;
        self.for_verkey = verkey.to_string();
        Ok(self)
    }

    pub fn version(&mut self, version: &Option<ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&self) -> VcxResult<(String, String)> {
        trace!("CreateKeyBuilder::send_secure >>>");

        if settings::agency_mocks_enabled() {
            match self.version {
                ProtocolTypes::V1 => AgencyMock::set_next_response(constants::CREATE_KEYS_RESPONSE),
                ProtocolTypes::V2 |
                ProtocolTypes::V3 |
                ProtocolTypes::V4 => AgencyMock::set_next_response(constants::CREATE_KEYS_V2_RESPONSE),
            }
        }

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(&response)
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
        trace!("CreateKeyBuilder::prepare_request >>>");

        let message = match self.version {
            ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::CreateKey(CreateKey {
                        msg_type: MessageTypes::MessageTypeV1(MessageTypes::build_v1(A2AMessageKinds::CreateKey)),
                        for_did: self.for_did.to_string(),
                        for_verkey: self.for_verkey.to_string()
                    })
                ),
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::CreateKey(CreateKey {
                        msg_type: MessageTypes::MessageTypeV2(MessageTypes::build_v2(A2AMessageKinds::CreateKey)),
                        for_did: self.for_did.to_string(),
                        for_verkey: self.for_verkey.to_string()
                    })
                ),
        };

        trace!("CreateKeyBuilder::prepare_request >>> message: {:?}", secret!(message));

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_response(&self, response: &[u8]) -> VcxResult<(String, String)> {
        trace!("CreateKeyBuilder::parse_response >>>");

        let mut response = parse_response_from_agency(response, &self.version)?;

        match response.swap_remove(0) {
            A2AMessage::Version1(A2AMessageV1::CreateKeyResponse(res)) => Ok((res.for_did, res.for_verkey)),
            A2AMessage::Version2(A2AMessageV2::CreateKeyResponse(res)) => Ok((res.for_did, res.for_verkey)),
            r => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Agency response does not match any variant of CreateKeyResponse, got: {:#?}", r)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::{MY1_SEED, MY2_SEED, MY3_SEED, CREATE_KEYS_V2_RESPONSE};
    use crate::utils::constants::CREATE_KEYS_RESPONSE;
    use crate::utils::libindy::crypto::create_and_store_my_did;
    use crate::agent::messages::create_keys;
    use crate::utils::devsetup::*;

    #[test]
    fn test_create_key_set_values() {
        let _setup = SetupDefaults::init();

        let for_did = "11235yBzrpJQmNyZzgoTqB";
        let for_verkey = "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";

        create_keys()
            .for_did(for_did).unwrap()
            .for_verkey(for_verkey).unwrap();
    }

    #[test]
    fn test_create_key_set_values_and_serialize() {
        let _setup = SetupLibraryWallet::init();

        let (_agent_did, agent_vk) = create_and_store_my_did(Some(MY2_SEED), None).unwrap();
        let (my_did, my_vk) = create_and_store_my_did(Some(MY1_SEED), None).unwrap();
        let (_agency_did, agency_vk) = create_and_store_my_did(Some(MY3_SEED), None).unwrap();

        settings::set_config_value(settings::CONFIG_AGENCY_VERKEY, &agency_vk);
        settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agent_vk);
        settings::set_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY, &my_vk);

        let bytes = create_keys()
            .for_did(&my_did).unwrap()
            .for_verkey(&my_vk).unwrap()
            .prepare_request().unwrap();
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_parse_create_keys_v1_response() {
        let _setup = SetupMocks::init();

        let mut builder = create_keys();

        let (for_did, for_verkey) = builder.version(&Some(ProtocolTypes::V1)).unwrap().parse_response(&CREATE_KEYS_RESPONSE.to_vec()).unwrap();

        assert_eq!(for_did, "U5LXs4U7P9msh647kToezy");
        assert_eq!(for_verkey, "FktSZg8idAVzyQZrdUppK6FTrfAzW3wWVzAjJAfdUvJq");
    }

    #[test]
    fn test_parse_create_keys_v2_response() {
        let _setup = SetupMocks::init();

        let mut builder = create_keys();

        let (for_did, for_verkey) = builder.version(&Some(ProtocolTypes::V2)).unwrap().parse_response(&CREATE_KEYS_V2_RESPONSE.to_vec()).unwrap();

        assert_eq!(for_did, "MNepeSWtGfhnv8jLB1sFZC");
        assert_eq!(for_verkey, "C73MRnns4qUjR5N4LRwTyiXVPKPrA5q4LCT8PZzxVdt9");
    }

    #[test]
    fn test_create_key_set_invalid_did_errors() {
        let _setup = SetupDefaults::init();

        let for_did = "11235yBzrpJQmNyZzgoT";
        let res = create_keys()
            .for_did(for_did)
            .unwrap_err();
        assert_eq!(res.kind(), VcxErrorKind::InvalidDid);
    }
}

