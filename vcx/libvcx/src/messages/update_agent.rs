use crate::error::prelude::*;
use crate::messages::message_type::MessageTypes;
use crate::messages::{A2AMessageKinds, A2AMessage, A2AMessageV1, A2AMessageV2, prepare_message_for_agency, parse_response_from_agency};
use crate::{settings, messages, utils};
use crate::settings::protocol::ProtocolTypes;
use crate::utils::{httpclient, constants};
use crate::utils::httpclient::AgencyMock;

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethodUpdated {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateComMethod {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "comMethod")]
    com_method: ComMethod,
}

impl UpdateComMethod {
    fn build(com_method: ComMethod) -> UpdateComMethod {
        UpdateComMethod {
            msg_type: MessageTypes::build(A2AMessageKinds::UpdateComMethod),
            com_method,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComMethod {
    pub id: String,
    #[serde(rename = "type")]
    #[serde(default = "default_com_type")]
    pub e_type: i32,
    pub value: String,
}

fn default_com_type() -> i32 { 1 }

pub fn update_agent_profile(agent_did: &str,
                            public_did: &Option<String>,
                            protocol_type: ProtocolTypes) -> VcxResult<u32> {
    if let Ok(name) = settings::get_config_value(settings::CONFIG_INSTITUTION_NAME) {
        messages::update_data()
            .to(agent_did)?
            .name(&name)?
            .logo_url(&settings::get_config_value(settings::CONFIG_INSTITUTION_LOGO_URL)?)?
            .use_public_did(public_did)?
            .version(&Some(protocol_type))?
            .send_secure()
            .map_err(|err| err.extend("Cannot update agent profile"))?;
    }

    trace!("Connection::create_agent_pairwise <<<");

    Ok(utils::error::SUCCESS.code_num)
}

pub fn update_agent_info(com_method: ComMethod) -> VcxResult<()> {
    trace!("update_agent_info >>> com_method: {:?}", secret!(com_method));
    debug!("Updating agent information");

    let to_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

    AgencyMock::set_next_response(constants::REGISTER_RESPONSE);

    let message = match settings::get_protocol_type() {
        ProtocolTypes::V1 => {
            A2AMessage::Version1(
                A2AMessageV1::UpdateComMethod(
                    UpdateComMethod {
                        msg_type: MessageTypes::MessageTypeV1(MessageTypes::build_v1(A2AMessageKinds::UpdateComMethod)),
                        com_method,
                    }
                )
            )
        }
        ProtocolTypes::V2 |
        ProtocolTypes::V3 => {
            A2AMessage::Version2(
                A2AMessageV2::UpdateComMethod(UpdateComMethod::build(com_method))
            )
        }
    };

    let data = prepare_message_for_agency(&message, &to_did, &settings::get_protocol_type())?;

    let response = httpclient::post_u8(&data)?;

    parse_response_from_agency(&response, &settings::get_protocol_type())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::devsetup::*;

    #[test]
    fn test_update_agent_info() {
        let _setup = SetupMocks::init();
        let com_method = ComMethod {
            id: "123".to_string(),
            e_type: 1,
            value: "value".to_string()
        };
        update_agent_info(com_method).unwrap();
    }

    #[cfg(all(feature = "agency", feature = "pool_tests"))]
    #[test]
    fn test_update_agent_info_real() {
        let _setup = SetupLibraryAgencyV2NewProvisioning::init();

        crate::utils::devsetup::set_consumer();

        let com_method = ComMethod {
            id: "7b7f97f2".to_string(),
            e_type: 1,
            value: "FCM:Value".to_string()
        };
        update_agent_info(com_method).unwrap();
    }

    #[test]
    fn test_deserialize_com_method() {
        let _setup = SetupEmpty::init();

        let id = "7b7f97f2";
        let value = "FCM:Value";
        let type_ = 4;

        // no `type` specified. 1 is default
        let json = json!({"id": id, "value": value}).to_string();
        let com_method: ComMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(id, com_method.id);
        assert_eq!(value, com_method.value);
        assert_eq!(1, com_method.e_type);

        // passed `type`
        let json = json!({"id": id, "value": value, "type": type_}).to_string();
        let com_method: ComMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(id, com_method.id);
        assert_eq!(value, com_method.value);
        assert_eq!(type_, com_method.e_type);

        // passed invalid json. no `value` field
        let json = json!({"id": id}).to_string();
        serde_json::from_str::<ComMethod>(&json).unwrap_err();
    }
}
