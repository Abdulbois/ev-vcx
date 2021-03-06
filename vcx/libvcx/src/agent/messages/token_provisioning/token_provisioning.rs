use crate::agent::messages::{A2AMessage, A2AMessageV2, A2AMessageKinds, prepare_message_for_agency, parse_response_from_agency};
use crate::error::prelude::*;
use crate::agent::messages::message_type::MessageTypes;
use crate::utils::httpclient;
use crate::settings::protocol::ProtocolTypes;
use crate::settings;
use crate::utils::libindy::wallet;
use crate::agent::provisioning::types::ProvisioningConfig;
use crate::agent::provisioning::utils::{process_provisioning_config, configure_wallet};

pub static VALID_SIGNATURE_ALGORITHMS: [&'static str; 2] = ["SafetyNet", "DeviceCheck"];

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenRequest {
    #[serde(rename = "@type")]
    pub msg_type: MessageTypes,
    #[serde(rename = "sponseeId")]
    sponsee_id: String,
    #[serde(rename = "sponsorId")]
    sponsor_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "sponsorId")]
    sponsor_id: String,
    #[serde(rename = "sponseeId")]
    sponsee_id: String,
    nonce: String,
    timestamp: String,
    sig: String,
    #[serde(rename = "sponsorVerKey")]
    sponsor_vk: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenRequestBuilder {
    sponsee_id: Option<String>,
    sponsor_id: Option<String>,
    version: Option<ProtocolTypes>,
    agency_did: Option<String>,
}

impl TokenRequestBuilder {
    pub fn build() -> TokenRequestBuilder {
        TokenRequestBuilder {
            sponsee_id: None,
            sponsor_id: None,
            version: None,
            agency_did: None,
        }
    }

    pub fn sponsee_id(&mut self, id: &str) -> &mut Self {
        self.sponsee_id = Some(id.to_string());
        self
    }
    pub fn sponsor_id(&mut self, id: &str) -> &mut Self {
        self.sponsor_id = Some(id.to_string());
        self
    }

    pub fn version(&mut self, version: ProtocolTypes) -> &mut Self {
        self.version = Some(version);
        self
    }
    pub fn agency_did(&mut self, did: &str) -> &mut Self {
        self.agency_did = Some(did.to_string());
        self
    }

    pub fn send_secure(&mut self) -> VcxResult<String> {
        trace!("TokenRequestBuilder::send >>>");

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(&response)
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
        trace!("TokenRequestBuilder::prepare_request >>>");

        let init_err = |e: &str| VcxError::from_msg(
            VcxErrorKind::CreateWalletBackup,
            format!("TokenRequest expects {} but got None", e),
        );

        let agency_did = self.agency_did.clone().ok_or(init_err("agency_did"))?;
        let message = A2AMessage::Version2(
            A2AMessageV2::TokenRequest(
                TokenRequest {
                    msg_type: MessageTypes::MessageTypeV2(MessageTypes::build_v2(A2AMessageKinds::TokenRequest)),
                    sponsee_id: self.sponsee_id.clone().ok_or(init_err("sponsee_id"))?,
                    sponsor_id: self.sponsor_id.clone().ok_or(init_err("sponsor_id"))?,
                }
            )
        );

        trace!("TokenRequestBuilder::prepare_request >>> message: {:?}", secret!(message));

        prepare_message_for_agency(&message, &agency_did, &ProtocolTypes::V3)
    }

    fn parse_response(&self, response: &[u8]) -> VcxResult<String> {
        trace!("TokenRequestBuilder::parse_response >>>");

        let response = parse_response_from_agency(response, &ProtocolTypes::V2)?;

        match response.first().ok_or_else(|| VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "No agency responses"))? {
            A2AMessage::Version1(_) => {
                Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response expected to be of version 2"))
            }
            A2AMessage::Version2(A2AMessageV2::TokenResponse(res)) => Ok(json!(res).to_string()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of TokenResponse"))
        }
    }
}

pub fn provision(config: ProvisioningConfig, sponsee_id: &str, sponsor_id: &str) -> VcxResult<String> {
    debug!("***Configuring Library");
    let config = process_provisioning_config(&json!(config).to_string())?;

    debug!("***Configuring Wallet");
    configure_wallet(&config)?;

    let agency_did = settings::get_config_value(settings::CONFIG_AGENCY_DID)?;

    debug!("Getting Token");
    let token = TokenRequestBuilder::build()
        .sponsee_id(sponsee_id)
        .sponsor_id(sponsor_id)
        .version(ProtocolTypes::V2)
        .agency_did(&agency_did)
        .send_secure()?;

    wallet::close_wallet()?;

    Ok(token)
}

#[cfg(all(test, feature = "agency", feature = "pool_tests"))]
mod tests {
    use super::*;
    use crate::settings;
    use crate::utils::constants;
    use crate::utils::devsetup::{C_AGENCY_DID, C_AGENCY_VERKEY, C_AGENCY_ENDPOINT, cleanup_indy_env};
    use crate::utils::libindy::wallet::delete_wallet;

    #[test]
    fn test_token_provisioning() {
        cleanup_indy_env();

        let seed1 = crate::utils::devsetup::create_new_seed();
        let enterprise_wallet_name = format!("{}_{}", crate::utils::constants::ENTERPRISE_PREFIX, settings::DEFAULT_WALLET_NAME);

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

        let config: ProvisioningConfig = ::serde_json::from_str(&config).unwrap();

        provision(config, "123", "456").unwrap();

        delete_wallet(&enterprise_wallet_name, None, None, None).unwrap();
    }
}

