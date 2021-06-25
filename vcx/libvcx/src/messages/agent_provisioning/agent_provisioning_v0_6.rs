use error::prelude::*;
use messages::agent_provisioning::utils::{configure_wallet, get_final_config, send_message_to_agency};
use settings;
use utils::libindy::wallet;
use messages::{A2AMessage, A2AMessageV2};
use messages::provision::{ConnectResponse, SignUp, SignUpResponse, CreateAgent, CreateAgentResponse, Connect};
use messages::agent_provisioning::types::ProvisioningConfig;


pub fn provision(config: &ProvisioningConfig) -> VcxResult<String> {
    trace!("provision_0_6 >>> config: {:?}", secret!(config));

    debug!("***Configuring Wallet");
    let (my_did, my_vk, wallet_name) = configure_wallet(&config)?;

    let agency_did = settings::get_config_value(settings::CONFIG_AGENCY_DID)?;

    debug!("Connecting to Agency");
    let (agent_did, agent_vk) = onboarding_v2(&my_did, &my_vk, &agency_did)?;

    let config = get_final_config(&my_did, &my_vk, &agent_did, &agent_vk, &wallet_name, &config)?;

    wallet::close_wallet()?;

    Ok(config)
}

pub fn connect_v2(my_did: &str, my_vk: &str, agency_did: &str) -> VcxResult<(String, String)> {
    /* STEP 1 - CONNECT */
    trace!("Sending CONNECT message");
    let message = A2AMessage::Version2(
        A2AMessageV2::Connect(Connect::build(my_did, my_vk))
    );

    let mut response = send_message_to_agency(&message, agency_did)?;

    let ConnectResponse { from_vk: agency_pw_vk, from_did: agency_pw_did, .. } =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::ConnectResponse(resp)) =>
                resp,
            _ => return
                Err(VcxError::from_msg(
                    VcxErrorKind::InvalidAgencyResponse,
                    "Message does not match any variant of ConnectResponse")
                )
        };

    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agency_pw_vk);
    Ok((agency_pw_did, agency_pw_vk))
}

// it will be changed next
fn onboarding_v2(my_did: &str, my_vk: &str, agency_did: &str) -> VcxResult<(String, String)> {
    trace!("Running Onboarding V2");

    let (agency_pw_did, _) = connect_v2(my_did, my_vk, agency_did)?;

    /* STEP 2 - REGISTER */
    trace!("Sending REGISTER message");
    let message = A2AMessage::Version2(
        A2AMessageV2::SignUp(SignUp::build())
    );

    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let _response: SignUpResponse =
        match response.swap_remove(0) {
            A2AMessage::Version2(A2AMessageV2::SignUpResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    trace!("Sending CREATE AGENT message");
    let message = A2AMessage::Version2(
        A2AMessageV2::CreateAgent(CreateAgent::build())
    );

    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let response: CreateAgentResponse =
        match response.swap_remove(0) {
            A2AMessage::Version2(A2AMessageV2::CreateAgentResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of CreateAgentResponse"))
        };

    Ok((response.from_did, response.from_vk))
}