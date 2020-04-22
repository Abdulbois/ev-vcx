use messages::{A2AMessage, A2AMessageV2, A2AMessageKinds, prepare_message_for_agency};
use error::prelude::*;
use messages::agent_utils::{set_config_values, configure_wallet, ComMethod, Config};
use messages::message_type::MessageTypes;
use utils::httpclient;
use settings::ProtocolTypes;
use settings::ProtocolTypes::V2;


#[derive(Serialize, Deserialize, Debug)]
pub struct TokenRequest {
    #[serde(rename = "@type")]
    pub msg_type: MessageTypes,
    id: String,
    #[serde(rename = "pushId")]
    push_id: ComMethod,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    id: String,
    sponsor: String,
    nonce: String,
    timestamp: String,
    sig: String,
    #[serde(rename = "sponsorVerKey")]
    sponsor_vk: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenRequestBuilder {
    id: Option<String>,
    push_id: Option<ComMethod>,
    version: Option<ProtocolTypes>,
    agency_did: Option<String>,
}
impl TokenRequestBuilder {
    pub fn build() -> TokenRequestBuilder {
        TokenRequestBuilder {
            id: None,
            push_id: None,
            version: None,
            agency_did: None,
        }
    }

    pub fn id(&mut self, id: &str) -> &mut Self { self.id = Some(id.to_string()); self}
    pub fn push_id(&mut self, push_id: ComMethod) -> &mut Self { self.push_id = Some(push_id); self}
    pub fn version(&mut self, version: ProtocolTypes) -> &mut Self { self.version = Some(version); self}
    pub fn agency_did(&mut self, did: &str) -> &mut Self { self.agency_did = Some(did.to_string()); self}

    pub fn send_secure(&mut self) -> VcxResult<()> {
        trace!("GetToken::send >>>");

        let data = self.prepare_request()?;

        httpclient::post_u8(&data)?;

        Ok(())
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
        let init_err = |e: &str| VcxError::from_msg(
            VcxErrorKind::CreateWalletBackup,
            format!("TokenRequest expects {} but got None", e)
        );

        let agency_did = self.agency_did.clone().ok_or(init_err("agency_did"))?;
        let version = self.version.clone().ok_or(init_err("protocol version"))?;
        let message = A2AMessage::Version2(
            A2AMessageV2::TokenRequest(
                TokenRequest {
                    msg_type: MessageTypes::build(A2AMessageKinds::TokenRequest),
                    id: self.id.clone().ok_or(init_err("id"))?,
                    push_id: self.push_id.clone().ok_or(init_err("push_id"))?,
                }
            )
        );

        prepare_message_for_agency(&message, &agency_did, &version)
    }
}

pub fn provision(my_config: Config, source_id: &str, com_method: ComMethod) -> VcxResult<()> {
    set_config_values(&my_config);
    configure_wallet(&my_config)?;

    TokenRequestBuilder::build()
        .id(source_id)
        .push_id(com_method)
        .version(V2)
        .agency_did(&my_config.agency_did)
        .send_secure()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use settings;
    use utils::constants;
    use utils::devsetup::{C_AGENCY_DID, C_AGENCY_VERKEY, C_AGENCY_ENDPOINT, cleanup_indy_env};
    use utils::plugins::init_plugin;
    use utils::libindy::wallet::delete_wallet;
    use messages::agent_utils::parse_config;

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_token_provisioning() {
        cleanup_indy_env();
        init_plugin(::settings::DEFAULT_PAYMENT_PLUGIN, ::settings::DEFAULT_PAYMENT_INIT_FUNCTION);

        let seed1 = ::utils::devsetup::create_new_seed();
        let enterprise_wallet_name = format!("{}_{}", ::utils::constants::ENTERPRISE_PREFIX, settings::DEFAULT_WALLET_NAME);

        let protocol_type = "2.0";
        let config = json!({
            "agency_url": C_AGENCY_ENDPOINT.to_string(),
            "agency_did": C_AGENCY_DID.to_string(),
            "agency_verkey": C_AGENCY_VERKEY.to_string(),
            "wallet_name": enterprise_wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
            "enterprise_seed": seed1,
            "agent_seed": seed1,
            "name": "institution".to_string(),
            "logo": "http://www.logo.com".to_string(),
            "path": constants::GENESIS_PATH.to_string(),
            "protocol_type": protocol_type,
        }).to_string();

        let com_method = ComMethod {
            id: "7b7f97f2".to_string(),
            value: "FCM:Value".to_string(),
            e_type: 1
        };

        provision(parse_config(&config).unwrap(), "123", com_method).unwrap();

        delete_wallet(&enterprise_wallet_name, None, None, None).unwrap();
    }
}
