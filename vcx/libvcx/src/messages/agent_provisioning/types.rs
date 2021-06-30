use crate::settings::protocol::ProtocolTypes;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvisioningConfig {
    #[serde(default)]
    pub protocol_type: ProtocolTypes,

    // agency related options
    pub agency_alias: Option<String>,
    pub agency_url: Option<String>,
    pub agency_did: Option<String>,
    pub agency_verkey: Option<String>,

    // ????
    pub    agent_seed: Option<String>,

    // wallet related options
    pub wallet_name: Option<String>,
    pub wallet_key: String,
    pub wallet_type: Option<String>,
    pub enterprise_seed: Option<String>,
    pub wallet_key_derivation: Option<String>,
    pub storage_config: Option<String>,
    pub storage_credentials: Option<String>,

    // pool ledger related options
    pub path: Option<String>,
    // ledger genesis transactions
    pub genesis_path: Option<String>,
    pub pool_config: Option<String>,
    pub pool_networks: Option<serde_json::Value>,
    // predefined alias
    pub pool_network_alias: Option<String>,

    // meta
    pub name: Option<String>,
    pub logo: Option<String>,
    pub institution_name: Option<String>,
    pub institution_logo_url: Option<String>,

    // rest
    pub did_method: Option<String>,
}
