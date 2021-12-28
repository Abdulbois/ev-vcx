use crate::error::prelude::*;
use crate::settings::environment::LedgerEnvironments;
use super::*;
use crate::utils::libindy::vdr::DEFAULT_NETWORK;
use crate::utils::author_agreement::TxnAuthorAgreementAcceptanceData;
use crate::utils::libindy::ledger::types::TxnAuthorAgreement;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct LibraryInitializationPoolConfig {
    pub genesis_path: Option<String>,
    pub genesis_transactions: Option<String>,
    pub pool_network_alias: Option<LedgerEnvironments>,
    pub network: Option<String>,
    pub author_agreement: Option<serde_json::Value>,
    pub pool_networks: Option<Vec<PoolNetworkConfigVariants>>,
    pub indy_pool_networks: Option<Vec<IndyPoolNetworkConfigVariants>>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct IndyPoolConfigWithGenesisPath {
    pub genesis_path: String,
    pub namespace_list: Option<Vec<String>>,
    pub taa_config: Option<TxnAuthorAgreement>,
}

impl IndyPoolConfigWithGenesisPath {
    pub fn to_config(self) -> VcxResult<IndyPoolConfig> {
        let genesis_transactions = read_file(&self.genesis_path)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidGenesisTxnPath,
                                              format!("Could not read file with genesis transactions. Err: {:?}", err)))?;

        Ok(IndyPoolConfig {
            genesis_transactions,
            namespace_list: self.namespace_list.unwrap_or(vec![DEFAULT_NETWORK.to_string()]),
            taa_config: self.taa_config,
        })
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct IndyPoolConfigWithTransactions {
    pub genesis_transactions: String,
    pub namespace_list: Option<Vec<String>>,
    pub taa_config: Option<TxnAuthorAgreement>,
}

impl IndyPoolConfigWithTransactions {
    pub fn to_config(self) -> VcxResult<IndyPoolConfig> {
        Ok(IndyPoolConfig {
            genesis_transactions: self.genesis_transactions,
            namespace_list: self.namespace_list.unwrap_or(vec![DEFAULT_NETWORK.to_string()]),
            taa_config: self.taa_config,
        })
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct IndyPoolConfigWithPredefinedAlias {
    pub pool_network_alias: LedgerEnvironments,
    pub namespace_list: Option<Vec<String>>,
    pub taa_config: Option<TxnAuthorAgreement>,
}

impl IndyPoolConfigWithPredefinedAlias {
    pub fn to_config(self) -> VcxResult<IndyPoolConfig> {
        Ok(IndyPoolConfig {
            genesis_transactions: self.pool_network_alias.transactions().to_string(),
            namespace_list: self.namespace_list.unwrap_or(vec![DEFAULT_NETWORK.to_string()]),
            taa_config: self.taa_config,
        })
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct CombinedPoolConfig {
    pub indy_pool_networks: Vec<IndyPoolNetworkConfigVariants>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct IndyPoolConfig {
    pub genesis_transactions: String,
    pub namespace_list: Vec<String>,
    pub taa_config: Option<TxnAuthorAgreement>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum IndyPoolNetworkConfigVariants {
    IndyWithGenesisPath(IndyPoolConfigWithGenesisPath),
    IndyWithTransactions(IndyPoolConfigWithTransactions),
    IndyWithPredefinedAlias(IndyPoolConfigWithPredefinedAlias),
}

impl IndyPoolNetworkConfigVariants {
    pub fn to_config(self) -> VcxResult<IndyPoolConfig> {
        match self {
            IndyPoolNetworkConfigVariants::IndyWithGenesisPath(config) => {
                config.to_config()
            }
            IndyPoolNetworkConfigVariants::IndyWithTransactions(config) => {
                config.to_config()
            }
            IndyPoolNetworkConfigVariants::IndyWithPredefinedAlias(config) => {
                config.to_config()
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum PoolNetworkConfigVariants {
    IndyWithGenesisPath(IndyPoolConfigWithGenesisPath),
    IndyWithTransactions(IndyPoolConfigWithTransactions),
    IndyWithPredefinedAlias(IndyPoolConfigWithPredefinedAlias),
}

impl PoolNetworkConfigVariants {
    pub fn to_config(self) -> VcxResult<IndyPoolConfig> {
        match self {
            PoolNetworkConfigVariants::IndyWithGenesisPath(config) => {
                config.to_config()
            }
            PoolNetworkConfigVariants::IndyWithTransactions(config) => {
                config.to_config()
            }
            PoolNetworkConfigVariants::IndyWithPredefinedAlias(config) => {
                config.to_config()
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum InitializePoolConfigVariants {
    List(Vec<PoolNetworkConfigVariants>),
    Single(PoolNetworkConfigVariants),
    SingleCombined(CombinedPoolConfig),
}

pub fn get_pool_config_values(config: &str) -> VcxResult<Vec<IndyPoolConfig>> {
    trace!("get_pool_config_values >>> config {}", secret!(config));

    let pool_config: LibraryInitializationPoolConfig = ::serde_json::from_str(config)
        .map_err(|err|
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                format!("Cannot parse pool config from JSON. Err: {}", err),
            )
        )?;

    let mut indy_pool_configs: Vec<IndyPoolConfig> = Vec::new();

    // Variants:
    // 1: { genesis_path, network }
    // 2: { pool_network_alias, network }
    // 3: { indy_pool_networks }
    // 4: { cheqd_pool_networks }
    // 5: { pool_networks }

    let taa_config: Option<TxnAuthorAgreement> = match &pool_config.author_agreement {
        Some(author_agreement) => {
            let data: TxnAuthorAgreementAcceptanceData = serde_json::from_value(author_agreement.clone())
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                  format!("Could not parse TxnAuthorAgreementAcceptanceData from JSON. Err: {:?}", err)))?;

            Some(data.into())
        }
        None => None,
    };

    if let Some(genesis_path) = pool_config.genesis_path {
        let genesis_transactions = read_file(&genesis_path)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidGenesisTxnPath,
                                              format!("Could not read file with genesis transactions. Err: {:?}", err)))?;
        indy_pool_configs.push(IndyPoolConfig {
            genesis_transactions,
            namespace_list: vec![pool_config.network.clone().unwrap_or(DEFAULT_NETWORK.to_string())],
            taa_config: taa_config.clone(),
        })
    }

    if let Some(genesis_transactions) = pool_config.genesis_transactions {
        indy_pool_configs.push(IndyPoolConfig {
            genesis_transactions,
            namespace_list: vec![pool_config.network.clone().unwrap_or(DEFAULT_NETWORK.to_string())],
            taa_config: taa_config.clone(),
        })
    }

    if let Some(pool_network_alias) = pool_config.pool_network_alias {
        indy_pool_configs.push(IndyPoolConfig {
            genesis_transactions: pool_network_alias.transactions().to_string(),
            namespace_list: vec![pool_config.network.unwrap_or(DEFAULT_NETWORK.to_string())],
            taa_config,
        })
    }

    if let Some(indy_pool_networks) = pool_config.indy_pool_networks {
        for config in indy_pool_networks.into_iter() {
            indy_pool_configs.push(config.to_config()?)
        }
    }

    if let Some(pool_networks) = pool_config.pool_networks {
        for config in pool_networks.into_iter() {
            indy_pool_configs.push(config.to_config()?)
        }
    }

    trace!("get_pool_config_values <<< indy_pool_configs: {:?}",
           indy_pool_configs);
    Ok(indy_pool_configs)
}

pub fn get_init_pool_config_values(config: &str) -> VcxResult<Vec<IndyPoolConfig>> {
    trace!("process_pool_config_string >>> config {}", secret!(config));
    debug!("processing pool config");

    let pool_configs: InitializePoolConfigVariants = serde_json::from_str(config)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidConfiguration,
                                          format!("Cannot parse Pool Network configuration from provided config JSON. Err: {:?}", err)))?;

    let mut indy_pool_configs: Vec<IndyPoolConfig> = Vec::new();

    match pool_configs {
        InitializePoolConfigVariants::Single(config) => {
            indy_pool_configs.push(config.to_config()?)
        }
        InitializePoolConfigVariants::SingleCombined(config) => {
            for config in config.indy_pool_networks.into_iter() {
                indy_pool_configs.push(config.to_config()?)
            }
        }
        InitializePoolConfigVariants::List(configs) => {
            for config in configs.into_iter() {
                indy_pool_configs.push(config.to_config()?)
            }
        }
    };

    trace!("process_pool_config_string <<< indy_pool_configs: {:?}",
           indy_pool_configs);
    Ok(indy_pool_configs)
}

pub fn get_indy_pool_networks() -> VcxResult<Vec<IndyPoolConfig>> {
    let networks = get_config_value(CONFIG_INDY_POOL_NETWORKS)
        .map_err(|_| VcxError::from_msg(
            VcxErrorKind::InvalidConfiguration,
            format!("Cannot open Pool Network: Provided configuration JSON doesn't contain pool network information"),
        ))?;


    let networks: Vec<IndyPoolConfig> = serde_json::from_str(&networks)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidConfiguration,
            format!("Cannot read Pool Network information from library settings. Err: {:?}", err),
        ))?;

    Ok(networks)
}
