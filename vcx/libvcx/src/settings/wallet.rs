use error::prelude::*;

#[cfg(all(feature = "mysql"))]
use std::env;

#[cfg(all(feature = "mysql"))]
use std::sync::Once;
use settings;

#[cfg(all(feature = "mysql"))]
use utils::libindy::mysql_wallet::init_mysql_wallet as do_init_mysql_wallet;

#[cfg(all(feature = "mysql"))]
static START: Once = Once::new();

#[cfg(all(feature = "mysql"))]
pub fn init_mysql_wallet() {
    START.call_once(|| {
        let _ = do_init_mysql_wallet();
    });
}

#[cfg(all(not(feature = "mysql")))]
pub fn init_mysql_wallet() {
    //nothing is initiated without this feature
}

#[cfg(all(not(feature = "mysql")))]
pub fn get_wallet_type() -> String {
    settings::DEFAULT_DEFAULT.to_string()
}

#[cfg(all(feature = "mysql"))]
pub fn get_wallet_type() -> String {
    "mysql".to_string()
}

#[cfg(all(not(feature = "mysql")))]
pub fn get_wallet_storage_config() -> String {
    trace!("setting default storage confing");
    settings::DEFAULT_WALLET_STORAGE_CONFIG.to_string()
}

#[cfg(all(feature = "mysql"))]
pub fn get_wallet_storage_config() -> String {
    trace!("setting mysql storage config");
    json!({
        "db_name": "wallet",
        "port": get_port(),
        "write_host": get_write_host(),
        "read_host": get_read_host()
    }).to_string()
}

#[cfg(all(feature = "mysql"))]
fn get_write_host() -> String {
    env::var("DB_WRITE_HOST").unwrap_or("mysql".to_string())
}

#[cfg(all(feature = "mysql"))]
fn get_read_host() -> String {
    env::var("DB_WRITE_HOST").unwrap_or("mysql".to_string())
}

#[cfg(all(feature = "mysql"))]
fn get_port() -> i32 {
    let port_var = env::var("DB_PORT").and_then(|s| s.parse::<i32>().map_err(|_| env::VarError::NotPresent));
    if port_var.is_err() {
        warn!("Port is absent or is not int, using default 3306");
    }
    port_var.unwrap_or(3306)
}

#[cfg(all(not(feature = "mysql")))]
pub fn get_wallet_storage_credentials() -> String {
    settings::DEFAULT_WALLET_STORAGE_CREDENTIALS.to_string()
}

#[cfg(all(feature = "mysql"))]
pub fn get_wallet_storage_credentials() -> String {
    json!({
        "pass": get_pass(),
        "user": get_user(),
    }).to_string()
}

#[cfg(all(feature = "mysql"))]
fn get_user() -> String {
    env::var("DB_USER").unwrap_or("root".to_string())
}

#[cfg(all(feature = "mysql"))]
fn get_pass() -> String {
    env::var("DB_ROOT").unwrap_or("root".to_string())
}

pub fn get_wallet_config(wallet_name: &str, wallet_type: Option<&str>, _storage_config: Option<&str>) -> String { // TODO: _storage_config must be used
    trace!("get_wallet_config >>> wallet_name: {}, wallet_type: {:?}", wallet_name, secret!(wallet_type));

    let mut config = json!({
        "id": wallet_name,
    });

    let config_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();

    if let Some(_type) = wallet_type.map(str::to_string).or(config_type) {
        config["storage_type"] = serde_json::Value::String(_type);
    }

    if let Ok(_config) = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CONFIG) {
        config["storage_config"] = serde_json::from_str(&_config).unwrap();
    }

    trace!("get_wallet_config >>> config: {:?}", secret!(config));

    config.to_string()
}

pub fn get_wallet_credentials(_storage_creds: Option<&str>) -> String { // TODO: storage_creds must be used?
    trace!("get_wallet_credentials >>> ");

    let key = settings::get_config_value(settings::CONFIG_WALLET_KEY).unwrap_or(settings::UNINITIALIZED_WALLET_KEY.to_string());
    let mut credentials = json!({"key": key});

    let key_derivation = settings::get_config_value(settings::CONFIG_WALLET_KEY_DERIVATION).ok();
    if let Some(_key) = key_derivation { credentials["key_derivation_method"] = json!(_key); }

    let storage_creds = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CREDS).ok();
    if let Some(_creds) = storage_creds { credentials["storage_credentials"] = serde_json::from_str(&_creds).unwrap(); }

    trace!("get_wallet_credentials >>> credentials: {:?}", secret!(credentials));

    credentials.to_string()
}

pub fn get_wallet_name() -> VcxResult<String> {
    settings::get_config_value(settings::CONFIG_WALLET_NAME)
        .map_err(|_| VcxError::from(VcxErrorKind::MissingWalletKey))
}