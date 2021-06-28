pub mod agent_provisioning_v0_5;
pub mod agent_provisioning_v0_6;
pub mod agent_provisioning_v0_7;
pub mod types;
pub mod utils;

use error::prelude::*;
use messages::agent_provisioning::utils::process_provisioning_config;
use settings::protocol::ProtocolTypes;


pub fn provision(config: &str) -> VcxResult<String> {
    trace!("provision >>> config: {:?}", secret!(config));
    debug!("***Configuring Library");
    let config = process_provisioning_config(&config)?;

    match config.protocol_type {
        ProtocolTypes::V1 => agent_provisioning_v0_5::provision(&config),
        ProtocolTypes::V2 |
        ProtocolTypes::V3=> agent_provisioning_v0_6::provision(&config),
    }
}