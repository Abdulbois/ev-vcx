use crate::error::prelude::*;
use crate::agent::provisioning::utils::{configure_wallet, get_final_config, send_message_to_agency};
use crate::settings;
use crate::utils::libindy::wallet;
use crate::utils::httpclient::AgencyMock;
use crate::agent::messages::{A2AMessage, A2AMessageV1};
use crate::utils::constants;
use crate::agent::messages::provision::{ConnectResponse, SignUp, SignUpResponse, CreateAgent, CreateAgentResponse, Connect};
use crate::agent::provisioning::types::ProvisioningConfig;

pub fn provision(config: &ProvisioningConfig) -> VcxResult<String> {
    trace!("provision_0_5 >>> config: {:?}", secret!(config));

    debug!("***Configuring Wallet");
    let (my_did, my_vk, wallet_name) = configure_wallet(&config)?;

    let agency_did = settings::get_config_value(settings::CONFIG_AGENCY_DID)?;

    debug!("Connecting to Agency");
    let (agent_did, agent_vk) = onboarding_v1(&my_did, &my_vk, &agency_did)?;

    let config = get_final_config(&my_did, &my_vk, &agent_did, &agent_vk, &wallet_name, &config)?;

    wallet::close_wallet()?;

    Ok(config)
}

fn onboarding_v1(my_did: &str, my_vk: &str, agency_did: &str) -> VcxResult<(String, String)> {
    trace!("Running Onboarding V1");

    /* STEP 1 - CONNECT */
    trace!("Sending CONNECT message");
    AgencyMock::set_next_response(constants::CONNECTED_RESPONSE);

    let message = A2AMessage::Version1(
        A2AMessageV1::Connect(Connect::build(my_did, my_vk))
    );

    let mut response = send_message_to_agency(&message, agency_did)?;

    let ConnectResponse { from_vk: agency_pw_vk, from_did: agency_pw_did, .. } =
        match response.swap_remove(0) {
            A2AMessage::Version1(A2AMessageV1::ConnectResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of ConnectResponse"))
        };

    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agency_pw_vk);

    /* STEP 2 - REGISTER */
    trace!("Sending REGISTER message");
    AgencyMock::set_next_response(constants::REGISTER_RESPONSE);

    let message = A2AMessage::Version1(
        A2AMessageV1::SignUp(SignUp::build())
    );

    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let _response: SignUpResponse =
        match response.swap_remove(0) {
            A2AMessage::Version1(A2AMessageV1::SignUpResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    trace!("Sending CREATE_AGENT message");
    AgencyMock::set_next_response(constants::AGENT_CREATED);

    let message = A2AMessage::Version1(
        A2AMessageV1::CreateAgent(CreateAgent::build())
    );

    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let response: CreateAgentResponse =
        match response.swap_remove(0) {
            A2AMessage::Version1(A2AMessageV1::CreateAgentResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of CreateAgentResponse"))
        };

    Ok((response.from_did, response.from_vk))
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;
    use crate::utils::devsetup::*;
    use crate::api::vcx::vcx_shutdown;
    use crate::agent::provisioning::types::ProvisioningConfig;
    use crate::agent::provisioning::provision;

    #[test]
    #[ignore]
    fn test_connect_register_provision_config_path() {
        let agency_did = "LTjTWsezEmV4wJYD5Ufxvk";
        let agency_vk = "BcCSmgdfChLqmtBkkA26YotWVFBNnyY45WCnQziF4cqN";
        let host = "https://eas.pdev.evernym.com";
        let wallet_key = "test_key";

        let path = if cfg!(target_os = "android") {
            env::var("EXTERNAL_STORAGE").unwrap() + "/tmp/custom1/"
        } else {
            "/tmp/custom1/".to_owned()
        };

        let config = json!({
            "wallet_name": "test_wallet",
            "storage_config": json!({
                "path": path
            }).to_string(),
            "agency_url": host.to_string(),
            "agency_did": agency_did.to_string(),
            "agency_verkey": agency_vk.to_string(),
            "wallet_key": wallet_key.to_string(),
        });

        //Creates wallet at custom location
        provision(&config.to_string()).unwrap();
        assert!(std::path::Path::new(&(path + "test_wallet")).exists());
        vcx_shutdown(false);
        let my_config: ProvisioningConfig = serde_json::from_str(&config.to_string()).unwrap();

        //Opens already created wallet at custom location
        configure_wallet(&my_config).unwrap();
    }

    #[test]
    fn test_connect_register_provision() {
        let _setup = SetupMocks::init();

        let agency_did = "Ab8TvZa3Q19VNkQVzAWVL7";
        let agency_vk = "5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf";
        let host = "http://www.whocares.org";
        let wallet_key = "test_key";
        let config = json!({
            "agency_url": host.to_string(),
            "agency_did": agency_did.to_string(),
            "agency_verkey": agency_vk.to_string(),
            "wallet_key": wallet_key.to_string(),
        });

        let result = provision(&config.to_string()).unwrap();

        let expected = json!({
            "agency_did":"Ab8TvZa3Q19VNkQVzAWVL7",
            "agency_endpoint":"http://www.whocares.org",
            "agency_verkey":"5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf",
            "institution_did":"FhrSrYtQcw3p9xwf7NYemf",
            "institution_logo_url":"<CHANGE_ME>",
            "institution_name":"<CHANGE_ME>",
            "institution_verkey":"91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "protocol_type":"1.0",
            "remote_to_sdk_did":"A4a69qafqZHPLPPu5JFQrc",
            "remote_to_sdk_verkey":"5wTKXrdfUiTQ7f3sZJzvHpcS7XHHxiBkFtPCsynZtv4k",
            "sdk_to_remote_did":"FhrSrYtQcw3p9xwf7NYemf",
            "sdk_to_remote_verkey":"91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "wallet_key":"test_key",
            "wallet_name":"LIBVCX_SDK_WALLET"
        });

        assert_eq!(expected, ::serde_json::from_str::<serde_json::Value>(&result).unwrap());
    }

    #[ignore]
    #[test]
    fn test_real_connect_register_provision() {
        let _setup = SetupDefaults::init();

        let agency_did = "VsKV7grR1BUE29mG2Fm2kX";
        let agency_vk = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";
        let host = "http://localhost:8080";
        let wallet_key = "test_key";
        let config = json!({
            "agency_url": host.to_string(),
            "agency_did": agency_did.to_string(),
            "agency_verkey": agency_vk.to_string(),
            "wallet_key": wallet_key.to_string(),
        });

        let result = provision(&config.to_string()).unwrap();
        assert!(result.len() > 0);
    }
}
