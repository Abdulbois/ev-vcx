use error::prelude::*;
use settings::environment::AgencyEnvironments;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ProvisioningAgencyConfig {
    pub agency_url: String,
    pub agency_did: String,
    pub agency_verkey: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ExplicitAgencyConfig {
    pub agency_endpoint: String,
    pub agency_did: String,
    pub agency_verkey: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct PredefinedAgencyConfig {
    pub agency_alias: AgencyEnvironments,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum AgencyConfigVariants {
    AgencyConfigProvisioning(ProvisioningAgencyConfig),
    AgencyConfig(ExplicitAgencyConfig),
    PredefinedAgencyConfig(PredefinedAgencyConfig),
}

pub fn get_agency_config_values(config: &str) -> VcxResult<ExplicitAgencyConfig> {
    let agency_config: AgencyConfigVariants = ::serde_json::from_str(config)
        .map_err(|err|
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                format!("Cannot parse agency config from JSON. Err: {:?}", err),
            )
        )?;

    let config = match agency_config {
        AgencyConfigVariants::AgencyConfigProvisioning(config) => {
            ExplicitAgencyConfig {
                agency_endpoint: config.agency_url,
                agency_did: config.agency_did,
                agency_verkey: config.agency_verkey
            }
        },
        AgencyConfigVariants::AgencyConfig(config) => config,
        AgencyConfigVariants::PredefinedAgencyConfig(config) => {
            ExplicitAgencyConfig {
                agency_endpoint: config.agency_alias.agency_endpoint().to_string(),
                agency_did: config.agency_alias.agency_did().to_string(),
                agency_verkey: config.agency_alias.agency_verkey().to_string()
            }
        }
    };

    Ok(config)
}