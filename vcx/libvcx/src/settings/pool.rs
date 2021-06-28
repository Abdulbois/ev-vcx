use error::prelude::*;
use settings::environment::LedgerEnvironments;
use super::*;
use utils::random::random_string;
use utils::libindy::pool::create_genesis_txn_file;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PoolNetworksConfig {
    pub pool_networks: Vec<PoolNetworkConfigVariants>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ExplicitPoolConfig {
    pub genesis_path: String,
    pub pool_name: Option<String>,
    pub pool_config: Option<serde_json::Value>,
}

impl ExplicitPoolConfig {
    pub fn to_config(self) -> VcxResult<PoolConfig> {
        let pool_name = self.pool_name.unwrap_or_else(|| random_string(10));
        Ok(PoolConfig {
            genesis_path: self.genesis_path,
            pool_name: Some(pool_name),
            pool_config: self.pool_config,
        })
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PredefinedPoolConfig {
    pub pool_network_alias: LedgerEnvironments,
    pub pool_name: Option<String>,
    pub pool_config: Option<serde_json::Value>,
}

impl PredefinedPoolConfig {
    pub fn to_config(self) -> VcxResult<PoolConfig> {
        let genesis_path = create_genesis_txn_file(
            self.pool_network_alias.name(),
            self.pool_network_alias.transactions())?;
        let pool_name = self.pool_name.unwrap_or_else(|| random_string(10));
        Ok(PoolConfig {
            genesis_path,
            pool_name: Some(pool_name),
            pool_config: self.pool_config,
        })
    }
}


#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PoolConfig {
    pub genesis_path: String,
    pub pool_name: Option<String>,
    pub pool_config: Option<serde_json::Value>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum LibraryInitializationPoolConfigVariants {
    Explicit(ExplicitPoolConfig),
    PoolNetworks(PoolNetworksConfig),
    Predefined(PredefinedPoolConfig),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum PoolNetworkConfigVariants {
    Explicit(ExplicitPoolConfig),
    Predefined(PredefinedPoolConfig),
}

impl PoolNetworkConfigVariants {
    pub fn to_config(self) -> VcxResult<PoolConfig> {
        match self {
            PoolNetworkConfigVariants::Explicit(config) => config.to_config(),
            PoolNetworkConfigVariants::Predefined(config) => config.to_config()
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum InitializePoolConfigVariants {
    List(Vec<PoolNetworkConfigVariants>),
    Single(PoolNetworkConfigVariants),
}

pub fn get_pool_config_values(config: &str) -> VcxResult<Vec<PoolConfig>> {
    trace!("get_pool_config_values >>> config {}", secret!(config));

    let pool_config: LibraryInitializationPoolConfigVariants = ::serde_json::from_str(config)
        .map_err(|err|
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                format!("Cannot parse pool config from JSON. Err: {}", err),
            )
        )?;

    let config = match pool_config {
        LibraryInitializationPoolConfigVariants::Explicit(config) => {
            let config = config.to_config()?;
            vec![config]
        }
        LibraryInitializationPoolConfigVariants::Predefined(config) => {
            let config = config.to_config()?;
            vec![config]
        }
        LibraryInitializationPoolConfigVariants::PoolNetworks(config) => {
            config.pool_networks
                .into_iter()
                .map(|config| config.to_config())
                .collect::<VcxResult<Vec<PoolConfig>>>()?
        }
    };

    trace!("get_pool_config_values <<< config: {:?}", config);
    Ok(config)
}

pub fn get_init_pool_config_values(config: &str) -> VcxResult<Vec<PoolConfig>> {
    trace!("process_pool_config_string >>> config {}", secret!(config));
    debug!("processing pool config");

    let pool_configs: InitializePoolConfigVariants = serde_json::from_str(config)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidConfiguration,
                                          format!("Cannot parse Pool Network configuration from provided config JSON. Err: {:?}", err)))?;

    let config = match pool_configs {
        InitializePoolConfigVariants::Single(config) => {
            let config = config.to_config()?;
            vec![config]
        }
        InitializePoolConfigVariants::List(configs) => {
            configs
                .into_iter()
                .map(|config| config.to_config())
                .collect::<VcxResult<Vec<PoolConfig>>>()?
        }
    };

    trace!("process_pool_config_string <<<");
    return Ok(config);
}

pub fn get_pool_networks() -> VcxResult<Vec<PoolConfig>> {
    let networks = get_config_value(CONFIG_POOL_NETWORKS)
        .map_err(|_| VcxError::from_msg(
            VcxErrorKind::InvalidConfiguration,
            format!("Cannot open Pool Network: Provided configuration JSON doesn't contain pool network information"),
        ))?;


    let networks: Vec<PoolConfig> = serde_json::from_str(&networks)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidConfiguration,
            format!("Cannot read Pool Network information from library settings. Err: {:?}", err),
        ))?;

    Ok(networks)
}