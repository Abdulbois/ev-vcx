use crate::error::prelude::*;
use crate::agent::provisioning::types::ProvisioningConfig;
use crate::settings;
use crate::settings::agency::get_agency_config_values;
use crate::utils::option_util::get_or_default;
use crate::utils::libindy::crypto::create_and_store_my_did;
use crate::utils::libindy::{
    wallet,
    anoncreds::holder::Holder,
};
use crate::settings::protocol::ProtocolTypes;
use crate::agent::messages::{prepare_message_for_agency, parse_response_from_agency, A2AMessage};
use crate::utils::httpclient;
use crate::agent::messages::update_agent::update_agent_profile;

pub fn process_provisioning_config(config_json: &str) -> VcxResult<ProvisioningConfig> {
    let config: ProvisioningConfig = ::serde_json::from_str(&config_json)
        .map_err(|err|
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                format!("Cannot parse config from JSON. Err: {}", err),
            )
        )?;

    let agency_config = get_agency_config_values(config_json)?;
    settings::set_config_value(settings::CONFIG_AGENCY_ENDPOINT, &agency_config.agency_endpoint);
    settings::set_config_value(settings::CONFIG_AGENCY_DID, &agency_config.agency_did);
    settings::set_config_value(settings::CONFIG_AGENCY_VERKEY, &agency_config.agency_verkey);

    let wallet_name = get_or_default(&config.wallet_name, settings::DEFAULT_WALLET_NAME);
    settings::set_config_value(settings::CONFIG_WALLET_NAME, &wallet_name);
    settings::set_config_value(settings::CONFIG_WALLET_KEY, &config.wallet_key);

    settings::set_opt_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, &config.wallet_key_derivation);
    settings::set_opt_config_value(settings::CONFIG_WALLET_TYPE, &config.wallet_type);
    settings::set_opt_config_value(settings::CONFIG_WALLET_STORAGE_CONFIG, &config.storage_config);
    settings::set_opt_config_value(settings::CONFIG_WALLET_STORAGE_CREDS, &config.storage_credentials);
    settings::set_opt_config_value(settings::CONFIG_POOL_CONFIG, &config.pool_config);
    settings::set_opt_config_value(settings::CONFIG_DID_METHOD, &config.did_method);
    settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, &config.protocol_type.to_string());

    let agency_verkey = settings::get_config_value(settings::CONFIG_AGENCY_VERKEY).unwrap_or_default();
    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agency_verkey);

    let institution_name = config.name.as_ref().or(config.institution_name.as_ref());
    let institution_logo_url = config.logo.as_ref().or(config.institution_logo_url.as_ref());
    settings::set_opt_config_value(settings::CONFIG_INSTITUTION_NAME, &institution_name.map(String::from));
    settings::set_opt_config_value(settings::CONFIG_INSTITUTION_LOGO_URL, &institution_logo_url.map(String::from));

    Ok(config)
}

fn create_issuer_keys(my_did: &str, my_vk: &str, my_config: &ProvisioningConfig) -> VcxResult<(String, String)> {
    if my_config.enterprise_seed == my_config.agent_seed {
        Ok((my_did.to_string(), my_vk.to_string()))
    } else {
        create_and_store_my_did(
            my_config.enterprise_seed.as_ref().map(String::as_str),
            my_config.did_method.as_ref().map(String::as_str),
        )
    }
}

pub fn configure_wallet(my_config: &ProvisioningConfig) -> VcxResult<(String, String, String)> {
    let wallet_name = get_or_default(&my_config.wallet_name, settings::DEFAULT_WALLET_NAME);

    wallet::init_wallet(
        &wallet_name,
        my_config.wallet_type.as_ref().map(String::as_str),
        my_config.storage_config.as_ref().map(String::as_str),
        my_config.storage_credentials.as_ref().map(String::as_str),
    )?;
    trace!("initialized wallet");

    // If MS is already in wallet then just continue
    Holder::create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).ok();

    let (my_did, my_vk) = create_and_store_my_did(
        my_config.agent_seed.as_ref().map(String::as_str),
        my_config.did_method.as_ref().map(String::as_str),
    )?;

    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY, &my_vk);

    Ok((my_did, my_vk, wallet_name))
}

pub fn get_final_config(my_did: &str,
                        my_vk: &str,
                        agent_did: &str,
                        agent_vk: &str,
                        wallet_name: &str,
                        my_config: &ProvisioningConfig) -> VcxResult<String> {
    let (issuer_did, issuer_vk) = create_issuer_keys(my_did, my_vk, my_config)?;

    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_DID, &agent_did);
    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agent_vk);

    /* Update Agent Info */
    update_agent_profile(&agent_did,
                         &Some(issuer_did.to_string()),
                         ProtocolTypes::V1)?;


    let institution_name = my_config.name.as_ref().or(my_config.institution_name.as_ref());
    let institution_logo_url = my_config.logo.as_ref().or(my_config.institution_logo_url.as_ref());

    let agency_url = settings::get_config_value(settings::CONFIG_AGENCY_ENDPOINT)?;
    let agency_did = settings::get_config_value(settings::CONFIG_AGENCY_DID)?;
    let agency_verkey = settings::get_config_value(settings::CONFIG_AGENCY_VERKEY)?;

    let mut final_config = json!({
        "wallet_key": &my_config.wallet_key,
        "wallet_name": wallet_name,
        "agency_endpoint": &agency_url,
        "agency_did": &agency_did,
        "agency_verkey": &agency_verkey,
        "sdk_to_remote_did": my_did,
        "sdk_to_remote_verkey": my_vk,
        "institution_did": issuer_did,
        "institution_verkey": issuer_vk,
        "remote_to_sdk_did": agent_did,
        "remote_to_sdk_verkey": agent_vk,
        "institution_name": get_or_default(&institution_name.map(String::from), "<CHANGE_ME>"),
        "institution_logo_url": get_or_default(&institution_logo_url.map(String::from), "<CHANGE_ME>"),
        "protocol_type": &my_config.protocol_type,
    });


    let genesis_path = my_config.path.as_ref().or(my_config.genesis_path.as_ref());
    if let Some(genesis_path) = &genesis_path {
        final_config["genesis_path"] = json!(genesis_path);
    }

    if let Some(key_derivation) = &my_config.wallet_key_derivation {
        final_config["wallet_key_derivation"] = json!(key_derivation);
    }
    if let Some(wallet_type) = &my_config.wallet_type {
        final_config["wallet_type"] = json!(wallet_type);
    }
    if let Some(_storage_config) = &my_config.storage_config {
        final_config["storage_config"] = json!(_storage_config);
    }
    if let Some(_storage_credentials) = &my_config.storage_credentials {
        final_config["storage_credentials"] = json!(_storage_credentials);
    }
    if let Some(_pool_config) = &my_config.pool_config {
        final_config["pool_config"] = json!(_pool_config);
    }
    if let Some(_pool_networks) = &my_config.pool_networks {
        final_config["pool_networks"] = json!(_pool_networks);
    }
    if let Some(_indy_pool_networks) = &my_config.indy_pool_networks {
        final_config["indy_pool_networks"] = json!(_indy_pool_networks);
    }
    if let Some(pool_network_alias) = &my_config.pool_network_alias {
        final_config["pool_network_alias"] = json!(pool_network_alias);
    }
    if let Some(author_agreement) = &my_config.author_agreement {
        final_config["author_agreement"] = json!(author_agreement);
    }

    Ok(final_config.to_string())
}

pub fn send_message_to_agency(message: &A2AMessage, did: &str) -> VcxResult<Vec<A2AMessage>> {
    let data = prepare_message_for_agency(message, &did, &settings::get_protocol_type())?;

    let response = httpclient::post_u8(&data)?;

    parse_response_from_agency(&response, &settings::get_protocol_type())
}
