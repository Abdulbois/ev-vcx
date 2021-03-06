pub mod agency;
pub mod pool;
pub mod environment;
pub mod wallet;
pub mod protocol;

use std::collections::HashMap;
use std::sync::RwLock;
use crate::utils::{get_temp_dir_path, error};
use std::path::Path;
use crate::utils::validation;
use serde_json::Value;
use strum::IntoEnumIterator;
use std::borrow::Borrow;
use reqwest::Url;
use crate::error::prelude::*;
use crate::utils::file::read_file;
use crate::aries::messages::a2a::protocol_registry::Actors;
use crate::settings::protocol::ProtocolTypes;

pub static CONFIG_POOL_NAME: &str = "pool_name";
pub static CONFIG_PROTOCOL_TYPE: &str = "protocol_type";
pub static CONFIG_AGENCY_ENDPOINT: &str = "agency_endpoint";
pub static CONFIG_AGENCY_DID: &str = "agency_did";
pub static CONFIG_AGENCY_VERKEY: &str = "agency_verkey";
pub static CONFIG_REMOTE_TO_SDK_DID: &str = "remote_to_sdk_did";
pub static CONFIG_REMOTE_TO_SDK_VERKEY: &str = "remote_to_sdk_verkey";
pub static CONFIG_SDK_TO_REMOTE_DID: &str = "sdk_to_remote_did";
pub static CONFIG_SDK_TO_REMOTE_VERKEY: &str = "sdk_to_remote_verkey";
pub static CONFIG_SDK_TO_REMOTE_ROLE: &str = "sdk_to_remote_role";
pub static CONFIG_INSTITUTION_DID: &str = "institution_did";
pub static CONFIG_INSTITUTION_VERKEY: &str = "institution_verkey";
pub static CONFIG_INSTITUTION_NAME: &str = "institution_name";
pub static CONFIG_INSTITUTION_LOGO_URL: &str = "institution_logo_url";
pub static CONFIG_ENABLE_TEST_MODE: &str = "enable_test_mode";
pub static CONFIG_GENESIS_PATH: &str = "genesis_path";
pub static CONFIG_EXPORTED_WALLET_PATH: &str = "exported_wallet_path";
pub static CONFIG_WALLET_BACKUP_KEY: &str = "backup_key";
pub static CONFIG_WALLET_KEY: &str = "wallet_key";
pub static CONFIG_WALLET_NAME: &'static str = "wallet_name";
pub static CONFIG_WALLET_TYPE: &'static str = "wallet_type";
pub static CONFIG_WALLET_STORAGE_CONFIG: &'static str = "storage_config";
pub static CONFIG_WALLET_STORAGE_CREDS: &'static str = "storage_credentials";
pub static CONFIG_WALLET_HANDLE: &'static str = "wallet_handle";
pub static CONFIG_THREADPOOL_SIZE: &'static str = "threadpool_size";
pub static CONFIG_WALLET_KEY_DERIVATION: &'static str = "wallet_key_derivation";
pub static CONFIG_PAYMENT_METHOD: &'static str = "payment_method";
pub static CONFIG_TXN_AUTHOR_AGREEMENT: &'static str = "author_agreement";
pub static CONFIG_POOL_CONFIG: &'static str = "pool_config";
pub static CONFIG_DID_METHOD: &str = "did_method";
pub static CONFIG_ACTORS: &str = "actors"; // inviter, invitee, issuer, holder, prover, verifier, sender, receiver
pub static CONFIG_POOL_NETWORKS: &str = "pool_networks";
pub static CONFIG_USE_LATEST_PROTOCOLS: &'static str = "use_latest_protocols";
pub static CONFIG_INDY_POOL_NETWORKS: &str = "indy_pool_networks";

pub static UNINITIALIZED_WALLET_KEY: &str = "<KEY_IS_NOT_SET>";
pub static DEFAULT_GENESIS_PATH: &str = "genesis.txn";
pub static DEFAULT_EXPORTED_WALLET_PATH: &str = "wallet.txn";
pub static DEFAULT_WALLET_NAME: &str = "LIBVCX_SDK_WALLET";
pub static DEFAULT_WALLET_STORAGE_CONFIG: &str = "{}";
pub static DEFAULT_WALLET_STORAGE_CREDENTIALS: &str = "{}";
pub static DEFAULT_POOL_NAME: &str = "pool1";
pub static DEFAULT_LINK_SECRET_ALIAS: &str = "main";
pub static DEFAULT_DEFAULT: &str = "default";
pub static DEFAULT_URL: &str = "http://127.0.0.1:8080";
pub static DEFAULT_DID: &str = "2hoqvcwupRTUNkXn6ArYzs";
pub static DEFAULT_VERKEY: &str = "FuN98eH2eZybECWkofW6A9BKJxxnTatBCopfUiNxo6ZB";
pub static DEFAULT_ROLE: &str = "0";
pub static DEFAULT_ENABLE_TEST_MODE: &str = "false";
pub static DEFAULT_WALLET_BACKUP_KEY: &str = "backup_wallet_key";
pub static DEFAULT_WALLET_KEY: &str = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";
pub static DEFAULT_THREADPOOL_SIZE: usize = 8;
pub static MASK_VALUE: &str = "********";
pub static DEFAULT_WALLET_KEY_DERIVATION: &str = "RAW";
pub static DEFAULT_PAYMENT_PLUGIN: &str = "libsovtoken.so";
pub static DEFAULT_PAYMENT_INIT_FUNCTION: &str = "sovtoken_init";
pub static DEFAULT_PAYMENT_METHOD: &str = "sov";
pub static DEFAULT_PROTOCOL_TYPE: &str = "3.0";
pub static MAX_THREADPOOL_SIZE: usize = 128;
pub static DEFAULT_USE_LATEST_PROTOCOLS: &str = "false";

lazy_static! {
    static ref SETTINGS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

trait ToString {
    fn to_string(&self) -> Self;
}

impl ToString for HashMap<String, String> {
    fn to_string(&self) -> Self {
        let mut v = self.clone();
        v.insert(CONFIG_WALLET_KEY.to_string(), MASK_VALUE.to_string());
        v
    }
}

pub fn set_defaults() -> u32 {
    trace!("set default settings >>>");

    // if this fails the program should exit
    let mut settings = SETTINGS.write().unwrap();

    #[cfg(all(feature = "mysql"))]
    wallet::init_mysql_wallet();

    settings.insert(CONFIG_POOL_NAME.to_string(), DEFAULT_POOL_NAME.to_string());
    settings.insert(CONFIG_WALLET_NAME.to_string(), DEFAULT_WALLET_NAME.to_string());
    settings.insert(CONFIG_WALLET_TYPE.to_string(), wallet::get_wallet_type());
    settings.insert(CONFIG_WALLET_STORAGE_CONFIG.to_string(), wallet::get_wallet_storage_config());
    settings.insert(CONFIG_WALLET_STORAGE_CREDS.to_string(), wallet::get_wallet_storage_credentials());
    settings.insert(CONFIG_AGENCY_ENDPOINT.to_string(), DEFAULT_URL.to_string());
    settings.insert(CONFIG_AGENCY_DID.to_string(), DEFAULT_DID.to_string());
    settings.insert(CONFIG_AGENCY_VERKEY.to_string(), DEFAULT_VERKEY.to_string());
    settings.insert(CONFIG_REMOTE_TO_SDK_DID.to_string(), DEFAULT_DID.to_string());
    settings.insert(CONFIG_REMOTE_TO_SDK_VERKEY.to_string(), DEFAULT_VERKEY.to_string());
    settings.insert(CONFIG_INSTITUTION_DID.to_string(), DEFAULT_DID.to_string());
    settings.insert(CONFIG_INSTITUTION_NAME.to_string(), DEFAULT_DEFAULT.to_string());
    settings.insert(CONFIG_INSTITUTION_LOGO_URL.to_string(), DEFAULT_URL.to_string());
    settings.insert(CONFIG_SDK_TO_REMOTE_DID.to_string(), DEFAULT_DID.to_string());
    settings.insert(CONFIG_SDK_TO_REMOTE_VERKEY.to_string(), DEFAULT_VERKEY.to_string());
    settings.insert(CONFIG_SDK_TO_REMOTE_ROLE.to_string(), DEFAULT_ROLE.to_string());
    settings.insert(CONFIG_WALLET_KEY.to_string(), DEFAULT_WALLET_KEY.to_string());
    settings.insert(CONFIG_WALLET_KEY_DERIVATION.to_string(), DEFAULT_WALLET_KEY_DERIVATION.to_string());
    settings.insert(CONFIG_EXPORTED_WALLET_PATH.to_string(),
                    get_temp_dir_path(DEFAULT_EXPORTED_WALLET_PATH).to_str().unwrap_or("").to_string());
    settings.insert(CONFIG_WALLET_BACKUP_KEY.to_string(), DEFAULT_WALLET_BACKUP_KEY.to_string());
    settings.insert(CONFIG_THREADPOOL_SIZE.to_string(), DEFAULT_THREADPOOL_SIZE.to_string());
    settings.insert(CONFIG_PAYMENT_METHOD.to_string(), DEFAULT_PAYMENT_METHOD.to_string());

    error::SUCCESS.code_num
}

pub fn validate_config(config: &HashMap<String, String>) -> VcxResult<u32> {
    trace!("validate_config >>> config: {:?}", secret!(config));
    debug!("validating config");

    //Mandatory parameters
    if !config.contains_key(CONFIG_WALLET_KEY) {
        return Err(VcxError::from(VcxErrorKind::MissingWalletKey));
    }

    // If values are provided, validate they're in the correct format
    validate_optional_config_val(config.get(CONFIG_INSTITUTION_DID), VcxErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_INSTITUTION_VERKEY), VcxErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_AGENCY_DID), VcxErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_AGENCY_VERKEY), VcxErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_SDK_TO_REMOTE_DID), VcxErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_SDK_TO_REMOTE_VERKEY), VcxErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_REMOTE_TO_SDK_DID), VcxErrorKind::InvalidDid, validation::validate_did)?;
    validate_optional_config_val(config.get(CONFIG_REMOTE_TO_SDK_VERKEY), VcxErrorKind::InvalidVerkey, validation::validate_verkey)?;

    validate_optional_config_val(config.get(CONFIG_AGENCY_ENDPOINT), VcxErrorKind::InvalidUrl, Url::parse)?;
    validate_optional_config_val(config.get(CONFIG_INSTITUTION_LOGO_URL), VcxErrorKind::InvalidUrl, Url::parse)?;

    validate_optional_config_val(config.get(CONFIG_ACTORS), VcxErrorKind::InvalidConfiguration, validation::validate_actors)?;

    trace!("validate_config <<<");

    Ok(error::SUCCESS.code_num)
}

fn _validate_mandatory_config_val<F, S, E>(val: Option<&String>, err: VcxErrorKind, closure: F) -> VcxResult<u32>
    where F: Fn(&str) -> Result<S, E> {
    closure(val.as_ref().ok_or(VcxError::from(err))?)
        .or(Err(VcxError::from(err)))?;

    Ok(error::SUCCESS.code_num)
}

fn validate_optional_config_val<F, S, E>(val: Option<&String>, err: VcxErrorKind, closure: F) -> VcxResult<()>
    where F: Fn(&str) -> Result<S, E> {
    match val {
        Some(val_) => {
            closure(val_)
                .or(Err(VcxError::from(err)))?;
            Ok(())
        }
        None => Ok(())
    }
}

pub fn log_settings() {
    let settings = SETTINGS.read().unwrap();
    trace!("loaded settings: {:?}", secret!(settings.to_string()));
}

pub fn indy_mocks_enabled() -> bool {
    let config = SETTINGS.read().unwrap();

    match config.get(CONFIG_ENABLE_TEST_MODE) {
        None => false,
        Some(value) => value == "true" || value == "indy"
    }
}

pub fn agency_mocks_enabled() -> bool {
    let config = SETTINGS.read().unwrap();

    match config.get(CONFIG_ENABLE_TEST_MODE) {
        None => false,
        Some(value) => value == "true" || value == "agency"
    }
}

pub fn process_config_string(config: &str, do_validation: bool) -> VcxResult<u32> {
    trace!("process_config_string >>> config {}", secret!(config));
    debug!("processing config");

    let configuration: Value = serde_json::from_str(config)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidConfiguration, format!("Cannot parse config from JSON. Err: {}", err)))?;

    if let Value::Object(ref map) = configuration {
        for (key, value) in map {
            match value {
                Value::String(value_) => set_config_value(key, &value_),
                Value::Array(value_) => set_config_value(key, &json!(value_).to_string()),
                Value::Object(value_) => set_config_value(key, &json!(value_).to_string()),
                Value::Bool(value_) => set_config_value(key, &json!(value_).to_string()),
                _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, format!("Unsupported type of the value {} is used for \"{}\" key.", value, key))),
            }
        }
    }

    if let Ok(agency_config) = agency::get_agency_config_values(config) {
        set_config_value(CONFIG_AGENCY_ENDPOINT, &agency_config.agency_endpoint);
        set_config_value(CONFIG_AGENCY_DID, &agency_config.agency_did);
        set_config_value(CONFIG_AGENCY_VERKEY, &agency_config.agency_verkey);
    }

    let indy_pool_configs = pool::get_pool_config_values(config)?;
    if !indy_pool_configs.is_empty() {
        set_config_value(CONFIG_INDY_POOL_NETWORKS, &json!(indy_pool_configs).to_string());
    }

    if do_validation {
        let setting = SETTINGS.read()
            .or(Err(VcxError::from(VcxErrorKind::InvalidConfiguration)))?;
        validate_config(&setting.borrow())?;
    }

    trace!("process_config_string <<<");

    Ok(error::SUCCESS.code_num)
}

pub fn process_config_file(path: &str) -> VcxResult<u32> {
    trace!("process_config_file >>> path: {}", secret!(path));

    if !Path::new(path).is_file() {
        error!("Configuration path was invalid");
        return Err(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, format!("Cannot find config file by specified path: {:?}", path)));
    }

    let config = read_file(path)?;
    process_config_string(&config, true)
}

pub fn process_init_pool_config_string(config: &str) -> VcxResult<()> {
    trace!("process_init_pool_config_string >>> config {}", secret!(config));

    let indy_pool_configs = pool::get_init_pool_config_values(config)?;
    set_config_value(CONFIG_INDY_POOL_NETWORKS, &json!(indy_pool_configs).to_string());

    trace!("process_init_pool_config_string <<<");
    Ok(())
}

pub fn get_threadpool_size() -> usize {
    let size = match get_config_value(CONFIG_THREADPOOL_SIZE) {
        Ok(x) => x.parse::<usize>().unwrap_or(DEFAULT_THREADPOOL_SIZE),
        Err(_) => DEFAULT_THREADPOOL_SIZE,
    };

    if size > MAX_THREADPOOL_SIZE {
        MAX_THREADPOOL_SIZE
    } else {
        size
    }
}

pub fn get_opt_config_value(key: &str) -> Option<String> {
    trace!("get_opt_config_value >>> key: {}", key);
    let value = match SETTINGS.read() {
        Ok(x) => x,
        Err(_) => return None
    }
        .get(key)
        .map(|v| v.to_string());

    trace!("get_opt_config_value <<< value: {:?}", secret!(value));
    value
}

pub fn get_config_value(key: &str) -> VcxResult<String> {
    trace!("get_config_value >>> key: {}", key);

    let get_config_value = get_opt_config_value(key)
        .ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidConfiguration,
            format!("Cannot read the value for \"{}\" key from library settings", key),
        ))?;

    trace!("get_config_value <<< value: {}", secret!(get_config_value));
    Ok(get_config_value)
}

pub fn set_opt_config_value(key: &str, value: &Option<String>) {
    trace!("set_opt_config_value >>> key: {}, key: {:?}", key, secret!(value));

    if let Some(v) = value {
        set_config_value(key, v.as_str())
    }
}

pub fn set_config_value(key: &str, value: &str) {
    trace!("set_config_value >>> key: {}, key: {}", key, secret!(value));
    SETTINGS
        .write().unwrap()
        .insert(key.to_string(), value.to_string());
}

pub fn unset_config_value(key: &str) {
    trace!("unset_config_value >>> key: {}", key);
    SETTINGS
        .write().unwrap()
        .remove(key);
}

pub fn get_payment_method() -> VcxResult<String> {
    get_config_value(CONFIG_PAYMENT_METHOD)
        .map_err(|_| VcxError::from_msg(VcxErrorKind::MissingPaymentMethod, "Payment Method is not set."))
}

pub fn is_aries_protocol_set() -> bool {
    let protocol_type = get_protocol_type();
    protocol_type == ProtocolTypes::V3 || protocol_type == ProtocolTypes::V4
}

pub fn is_strict_aries_protocol_set() -> bool {
    get_protocol_type() == ProtocolTypes::V4
}

pub fn get_actors() -> Vec<Actors> {
    get_config_value(CONFIG_ACTORS)
        .and_then(|actors|
            ::serde_json::from_str(&actors)
                .map_err(|_| VcxError::from(VcxErrorKind::InvalidConfiguration))
        ).unwrap_or_else(|_| Actors::iter().collect())
}

pub fn get_connecting_protocol_version() -> ProtocolTypes {
    trace!("get_connecting_protocol_version >>> ");

    let protocol = get_config_value(CONFIG_USE_LATEST_PROTOCOLS).unwrap_or(DEFAULT_USE_LATEST_PROTOCOLS.to_string());
    let protocol = match protocol.as_ref() {
        "true" | "TRUE" | "True" => ProtocolTypes::V2,
        "false" | "FALSE" | "False" | _ => ProtocolTypes::V1,
    };
    trace!("get_connecting_protocol_version >>> protocol: {:?}", protocol);
    protocol
}

pub fn get_protocol_type() -> ProtocolTypes {
    trace!("get_protocol_type >>> ");

    let protocol_type = ProtocolTypes::from(get_config_value(CONFIG_PROTOCOL_TYPE)
        .unwrap_or(DEFAULT_PROTOCOL_TYPE.to_string()));

    trace!("get_protocol_type >>> protocol_type: {:?}", protocol_type);
    protocol_type
}

pub fn clear_config() {
    trace!("clear_config >>>");
    let mut config = SETTINGS.write().unwrap();
    config.clear();
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils::devsetup::{TempFile, SetupDefaults};
    use crate::settings::pool::get_indy_pool_networks;

    fn _institution_name() -> String {
        "enterprise".to_string()
    }

    fn _pool_config() -> String {
        r#"{"timeout":40}"#.to_string()
    }

    fn base_config() -> serde_json::Value {
        json!({
            "pool_name" : "pool1",
            "config_name":"config1",
            "wallet_name":"test_read_config_file",
            "agency_endpoint" : "https://agency.com",
            "agency_did" : "72x8p4HubxzUK1dwxcc5FU",
            "agency_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "remote_to_sdk_did" : "UJGjM6Cea2YVixjWwHN9wq",
            "sdk_to_remote_did" : "AB3JM851T4EQmhh8CdagSP",
            "sdk_to_remote_verkey" : "888MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "institution_name" : _institution_name(),
            "remote_to_sdk_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "genesis_transactions":"{}",
            "wallet_key":"key",
            "pool_config": _pool_config(),
            "payment_method": "null"
        })
    }

    #[test]
    fn test_process_config_str_for_aliases() {
        let _setup = SetupDefaults::init();

        let config = json!({
            "wallet_name":"test_read_config_file",
            "agency_alias" : "demo",
            "pool_network_alias" : "demo",
            "remote_to_sdk_did" : "UJGjM6Cea2YVixjWwHN9wq",
            "sdk_to_remote_did" : "AB3JM851T4EQmhh8CdagSP",
            "sdk_to_remote_verkey" : "888MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "remote_to_sdk_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "institution_name" : _institution_name(),
            "wallet_key":"key",
        }).to_string();

        assert_eq!(process_config_string(&config, true).unwrap(), error::SUCCESS.code_num);

        assert_eq!(pool::get_indy_pool_networks().unwrap().len(), 1);

        assert_eq!(get_config_value(CONFIG_AGENCY_ENDPOINT).unwrap(), environment::DEMO_AGENCY_ENDPOINT);
        assert_eq!(get_config_value(CONFIG_AGENCY_DID).unwrap(), environment::DEMO_AGENCY_DID);
        assert_eq!(get_config_value(CONFIG_AGENCY_VERKEY).unwrap(), environment::DEMO_AGENCY_VERKEY);
    }

    pub fn config_json() -> String {
        base_config().to_string()
    }

    #[test]
    fn test_bad_path() {
        let _setup = SetupDefaults::init();

        let path = "garbage.txt";
        assert_eq!(process_config_file(&path).unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
    }

    #[test]
    fn test_read_config_file() {
        let _setup = SetupDefaults::init();

        let mut config_file: TempFile = TempFile::create("test_init.json");
        config_file.write(&config_json());

        assert_eq!(read_file(&config_file.path).unwrap(), config_json());
    }

    #[test]
    fn test_process_file() {
        let _setup = SetupDefaults::init();

        let mut config_file: TempFile = TempFile::create("test_init.json");
        config_file.write(&config_json());

        assert_eq!(process_config_file(&config_file.path).unwrap(), error::SUCCESS.code_num);

        assert_eq!(get_config_value("institution_name").unwrap(), _institution_name());
    }

    #[test]
    fn test_process_config_str() {
        let _setup = SetupDefaults::init();

        assert_eq!(process_config_string(&config_json(), true).unwrap(), error::SUCCESS.code_num);

        assert_eq!(get_config_value("institution_name").unwrap(), _institution_name());
        assert_eq!(get_config_value("pool_config").unwrap(), _pool_config());
    }

    #[test]
    fn test_validate_config() {
        let _setup = SetupDefaults::init();

        let config: HashMap<String, String> = serde_json::from_str(&config_json()).unwrap();
        assert_eq!(validate_config(&config).unwrap(), error::SUCCESS.code_num);
    }

    fn _mandatory_config() -> HashMap<String, String> {
        let mut config: HashMap<String, String> = HashMap::new();
        config.insert(CONFIG_WALLET_KEY.to_string(), "password".to_string());
        config
    }

    #[test]
    fn test_validate_config_failures() {
        let _setup = SetupDefaults::init();

        let invalid = "invalid";

        let config = HashMap::new();
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::MissingWalletKey);

        let mut config = _mandatory_config();
        config.insert(CONFIG_INSTITUTION_DID.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidDid);

        let mut config = _mandatory_config();
        config.insert(CONFIG_INSTITUTION_VERKEY.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidVerkey);

        let mut config = _mandatory_config();
        config.insert(CONFIG_AGENCY_DID.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidDid);

        let mut config = _mandatory_config();
        config.insert(CONFIG_AGENCY_VERKEY.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidVerkey);

        let mut config = _mandatory_config();
        config.insert(CONFIG_SDK_TO_REMOTE_DID.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidDid);

        let mut config = _mandatory_config();
        config.insert(CONFIG_SDK_TO_REMOTE_VERKEY.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidVerkey);

        let mut config = _mandatory_config();
        config.insert(CONFIG_REMOTE_TO_SDK_DID.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidDid);

        let mut config = _mandatory_config();
        config.insert(CONFIG_SDK_TO_REMOTE_VERKEY.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidVerkey);

        let mut config = _mandatory_config();
        config.insert(CONFIG_INSTITUTION_LOGO_URL.to_string(), invalid.to_string());
        assert_eq!(validate_config(&config).unwrap_err().kind(), VcxErrorKind::InvalidUrl);
    }

    #[test]
    fn test_validate_optional_config_val() {
        let _setup = SetupDefaults::init();

        let closure = Url::parse;
        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("valid".to_string(), DEFAULT_URL.to_string());
        config.insert("invalid".to_string(), "invalid_url".to_string());

        //Success
        validate_optional_config_val(config.get("valid"), VcxErrorKind::InvalidUrl, closure).unwrap();

        // Success with No config
        validate_optional_config_val(config.get("unknown"), VcxErrorKind::InvalidUrl, closure).unwrap();

        // Fail with failed fn call
        assert_eq!(validate_optional_config_val(config.get("invalid"),
                                                VcxErrorKind::InvalidUrl,
                                                closure).unwrap_err().kind(), VcxErrorKind::InvalidUrl);
    }

    #[test]
    fn test_get_and_set_values() {
        let _setup = SetupDefaults::init();

        let key = "key1".to_string();
        let value1 = "value1".to_string();

        // Fails with invalid key
        assert_eq!(get_config_value(&key).unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);

        set_config_value(&key, &value1);
        assert_eq!(get_config_value(&key).unwrap(), value1);
    }

    #[test]
    fn test_clear_config() {
        let _setup = SetupDefaults::init();

        let content = json!({
            "agency_alias" : "demo",
            "pool_name" : "pool1",
            "config_name":"config1",
            "wallet_name":"test_clear_config",
            "institution_name" : "evernym enterprise",
            "genesis_transactions":"/tmp/pool1.txn",
            "wallet_key":"key",
        }).to_string();

        assert_eq!(process_config_string(&content, false).unwrap(), error::SUCCESS.code_num);

        assert_eq!(get_config_value("pool_name").unwrap(), "pool1".to_string());
        assert_eq!(get_config_value("config_name").unwrap(), "config1".to_string());
        assert_eq!(get_config_value("wallet_name").unwrap(), "test_clear_config".to_string());
        assert_eq!(get_config_value("institution_name").unwrap(), "evernym enterprise".to_string());
        assert_eq!(get_config_value("genesis_transactions").unwrap(), "/tmp/pool1.txn".to_string());
        assert_eq!(get_config_value("wallet_key").unwrap(), "key".to_string());

        clear_config();

        // Fails after config is cleared
        assert_eq!(get_config_value("pool_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("config_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("wallet_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("institution_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("genesis_transactions").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        assert_eq!(get_config_value("wallet_key").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
    }

    #[test]
    fn test_process_config_str_for_actors() {
        let _setup = SetupDefaults::init();

        let mut config = base_config();
        config["actors"] = json!(["invitee", "holder"]);

        process_config_string(&config.to_string(), true).unwrap();

        assert_eq!(vec![Actors::Invitee, Actors::Holder], get_actors());

        // passed invalid actor
        config["actors"] = json!(["wrong"]);
        assert_eq!(process_config_string(&config.to_string(), true).unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
    }

    #[test]
    fn test_process_pool_config() {
        let _setup = SetupDefaults::init();

        let transactions = "{}";

        // Only required field
        let config = json!({
            "genesis_transactions": transactions
        }).to_string();

        process_init_pool_config_string(&config.to_string()).unwrap();

        let networks = get_indy_pool_networks().unwrap();
        assert_eq!(networks.len(), 1);

        // With optional fields
        let config = json!({
            "genesis_transactions": transactions,
            "namespace_list": vec!["test"],
        }).to_string();

        process_init_pool_config_string(&config.to_string()).unwrap();

        let networks = get_indy_pool_networks().unwrap();
        assert_eq!(networks.len(), 1);

        // Empty
        let config = json!({}).to_string();
        assert_eq!(process_init_pool_config_string(&config.to_string()).unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
    }

    #[test]
    fn test_process_pool_config_for_multiple_networks() {
        let _setup = SetupDefaults::init();

        let genesis_transactions = "{}";
        let namespace_list = vec!["pool1"];

        let genesis_transactions_2 = "{}";
        let namespace_list_2 = vec!["pool2"];

        let config = json!(
            vec![
                json!({
                    "namespace_list": namespace_list,
                    "genesis_transactions": genesis_transactions
                }),
                json!({
                    "namespace_list": namespace_list_2,
                    "genesis_transactions": genesis_transactions_2
                })
            ]
        );

        process_init_pool_config_string(&config.to_string()).unwrap();

        let networks = get_indy_pool_networks().unwrap();
        assert_eq!(networks.len(), 2);
        assert_eq!(networks[0].genesis_transactions, genesis_transactions.to_string());
        assert_eq!(networks[1].genesis_transactions, genesis_transactions_2.to_string());
    }
}
