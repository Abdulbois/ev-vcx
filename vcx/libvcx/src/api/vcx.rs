use crate::utils::{version_constants, threadpool};
use libc::c_char;
use crate::utils::cstring::CStringUtils;
use crate::utils::libindy::{wallet, vdr};
use crate::utils::error;
use crate::settings;
use std::ffi::CString;
use crate::utils::threadpool::spawn;
use crate::error::prelude::*;
use crate::indy::{INVALID_WALLET_HANDLE, CommandHandle};
use crate::utils::libindy::vdr::init_vdr;
use std::thread;

/// Initializes VCX with config settings
///
/// example configuration is in libvcx/sample_config/config.json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// config: config as json.
/// The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
///
/// cb: Callback that provides error status of initialization
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_init_with_config(command_handle: CommandHandle,
                                   config: *const c_char,
                                   cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_init_with_config >>>");

    check_useful_c_str!(config,VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_init(command_handle: {}, config: {:?})",
           command_handle, secret!(config));

    if config == "ENABLE_TEST_MODE" {
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        settings::set_defaults();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "1.0");
    } else {
        match settings::process_config_string(&config, true) {
            Err(e) => {
                error!("Cannot initialize with given config.");
                return e.into();
            }
            Ok(_) => (),
        }
    };

    _finish_init(command_handle, cb)
}

/// Initializes VCX with config file
///
/// An example file is at libvcx/sample_config/config.json
/// The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// config_path: path to a config file to populate config attributes
///
/// cb: Callback that provides error status of initialization
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_init(command_handle: CommandHandle,
                       config_path: *const c_char,
                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_init >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_init(command_handle: {}, config_path: {:?})",
           command_handle, secret!(config_path));


    if !config_path.is_null() {
        check_useful_c_str!(config_path,VcxErrorKind::InvalidOption);

        if config_path == "ENABLE_TEST_MODE" {
            settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
            settings::set_defaults();
            settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "1.0");
        } else {
            match settings::process_config_file(&config_path) {
                Ok(_) => (),
                Err(err) => {
                    error!("Cannot initialize with given config path.");
                    return err.into();
                }
            };
        }
    } else {
        error!("Cannot initialize with given config path: config path is null.");
        return VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "Cannot initialize with given config path: config path is null.").into();
    }

    _finish_init(command_handle, cb)
}

fn _finish_init(command_handle: CommandHandle, cb: extern fn(xcommand_handle: CommandHandle, err: u32)) -> u32 {
    threadpool::init();

    settings::log_settings();

    if wallet::get_wallet_handle() != INVALID_WALLET_HANDLE {
        error!("Library was already initialized");
        return VcxError::from_msg(VcxErrorKind::AlreadyInitialized, "Library was already initialized").into();
    }
    // Wallet name was already validated
    let wallet_name = match settings::get_config_value(settings::CONFIG_WALLET_NAME) {
        Ok(x) => x,
        Err(_) => {
            debug!("No `wallet_name` parameter specified in the config JSON. Using default: {}", settings::DEFAULT_WALLET_NAME.to_string());
            settings::set_config_value(settings::CONFIG_WALLET_NAME, settings::DEFAULT_WALLET_NAME);
            settings::DEFAULT_WALLET_NAME.to_string()
        }
    };

    let wallet_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();
    let storage_config = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CONFIG).ok();
    let storage_creds = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CREDS).ok();

    trace!("libvcx version: {}{}", version_constants::VERSION, version_constants::REVISION);

    spawn(move || {
        let pool_open_thread = thread::spawn(|| {
            if settings::pool::get_indy_pool_networks().is_err() {
                info!("Skipping connection to Pool Ledger Network as no configs passed");
                return Ok(());
            }

            init_vdr()
                .map(|res| {
                    info!("Init Pool Successful.");
                    res
                })
        });

        match wallet::open_wallet(&wallet_name,
                                  wallet_type.as_ref().map(String::as_str),
                                  storage_config.as_ref().map(String::as_str),
                                  storage_creds.as_ref().map(String::as_str)) {
            Ok(_) => {
                info!("Init Wallet Successful.");
            }
            Err(e) => {
                error!("Init Wallet Error {}..", e);
                cb(command_handle, e.into());
                return Ok(());
            }
        }

        match pool_open_thread.join() {
            Ok(Ok(())) => {
                cb(command_handle, error::SUCCESS.code_num);
            }
            Ok(Err(e)) => {
                error!("Init Pool Error {}.", e);
                cb(command_handle, e.into());
            }
            Err(e) => {
                error!("Init Pool Error {:?}.", e);
                let error = VcxError::from_msg(VcxErrorKind::IOError, format!("Could not join thread: {:?}.", e));
                cb(command_handle, error.into());
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Connect to a Pool Ledger
///
/// You can deffer connecting to the Pool Ledger during library initialization (vcx_init or vcx_init_with_config)
/// to decrease the taken time by omitting `genesis_path` field in config JSON.
/// Next, you can use this function (for instance as a background task) to perform a connection to the Pool Ledger.
///
/// Note: Pool must be already initialized before sending any request to the Ledger.
///
/// EXPERIMENTAL
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// pool_config: string - the configuration JSON containing pool related settings.
///                 {
///                     genesis_path: string - path to pool ledger genesis transactions,
///                     pool_name: Optional[string] - name of the pool ledger configuration will be created.
///                                                   If no value specified, the default pool name pool_name will be used.
///                     pool_config: Optional[string] - runtime pool configuration json:
///                             {
///                                 "timeout": int (optional), timeout for network request (in sec).
///                                 "extended_timeout": int (optional), extended timeout for network request (in sec).
///                                 "preordered_nodes": array<string> -  (optional), names of nodes which will have a priority during request sending:
///                                         ["name_of_1st_prior_node",  "name_of_2nd_prior_node", .... ]
///                                         This can be useful if a user prefers querying specific nodes.
///                                         Assume that `Node1` and `Node2` nodes reply faster.
///                                         If you pass them Libindy always sends a read request to these nodes first and only then (if not enough) to others.
///                                         Note: Nodes not specified will be placed randomly.
///                                 "number_read_nodes": int (optional) - the number of nodes to send read requests (2 by default)
///                                         By default Libindy sends a read requests to 2 nodes in the pool.
///                     }
///                     network: Optional[string] - Network identifier used for fully-qualified DIDs.
///                 }
///                 Note: You can also pass a list of network configs.
///                       In this case library will connect to multiple ledger networks and will look up public data in each of them.
///                     [{ "genesis_path": string, "pool_name": string, ... }]
///
/// cb: Callback that provides no value
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_init_pool(command_handle: CommandHandle,
                            pool_config: *const c_char,
                            cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                 err: u32)>) -> u32 {
    info!("vcx_init_pool >>>");

    check_useful_c_str!(pool_config, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_init_pool(command_handle: {}, pool_config: {:?})",
           command_handle, pool_config);

    match settings::process_init_pool_config_string(&pool_config) {
        Err(e) => {
            error!("Invalid pool configuration specified: {}", e);
            return e.into();
        }
        Ok(_) => (),
    }

    spawn(move || {
        match init_vdr() {
            Ok(()) => {
                trace!("vcx_init_pool_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                error!("vcx_init_pool_cb(command_handle: {}, rc: {})",
                       command_handle, e);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

lazy_static! {
    pub static ref VERSION_STRING: CString = CString::new(format!("{}{}", version_constants::VERSION, version_constants::REVISION)).unwrap();
}

#[no_mangle]
pub extern fn vcx_version() -> *const c_char {
    info!("vcx_version >>>");
    VERSION_STRING.as_ptr()
}

/// Reset libvcx to a pre-configured state, releasing/deleting any handles and freeing memory
///
/// libvcx will be inoperable and must be initialized again with vcx_init_with_config
///
/// #Params
/// delete: specify whether wallet/pool should be deleted
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_shutdown(delete: bool) -> u32 {
    info!("vcx_shutdown >>>");
    trace!("vcx_shutdown(delete: {})", delete);

    match wallet::close_wallet() {
        Ok(()) => {}
        Err(_) => {}
    };

    match vdr::close_vdr() {
        Ok(()) => {}
        Err(_) => {}
    };

    crate::schema::release_all();
    crate::connection::release_all();
    crate::issuer_credential::release_all();
    crate::credential_def::release_all();
    crate::proof::release_all();
    crate::disclosed_proof::release_all();
    crate::credential::release_all();

    if delete {
        let wallet_name = settings::get_config_value(settings::CONFIG_WALLET_NAME)
            .unwrap_or(settings::DEFAULT_WALLET_NAME.to_string());

        let wallet_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();

        match wallet::delete_wallet(&wallet_name, wallet_type.as_ref().map(String::as_str), None, None) {
            Ok(()) => (),
            Err(_) => (),
        };

        vdr::close_vdr().ok();
    }

    settings::clear_config();
    trace!("vcx_shutdown(delete: {})", delete);
    error::SUCCESS.code_num
}

/// Get the message corresponding to an error code
///
/// #Params
/// error_code: code of error
///
/// #Returns
/// Error message
#[no_mangle]
pub extern fn vcx_error_c_message(error_code: u32) -> *const c_char {
    info!("vcx_error_c_message >>>");
    trace!("vcx_error_message(error_code: {})", error_code);
    error::error_c_message(error_code).as_ptr()
}

/// Update setting to set new local institution information
///
/// #Params
/// name: institution name
/// logo_url: url containing institution logo
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_update_institution_info(name: *const c_char, logo_url: *const c_char) -> u32 {
    info!("vcx_update_institution_info >>>");

    check_useful_c_str!(name, VcxErrorKind::InvalidOption);
    check_useful_c_str!(logo_url, VcxErrorKind::InvalidOption);

    trace!("vcx_update_institution_info(name: {}, logo_url: {})", secret!(name), secret!(logo_url));

    settings::set_config_value(crate::settings::CONFIG_INSTITUTION_NAME, &name);
    settings::set_config_value(crate::settings::CONFIG_INSTITUTION_LOGO_URL, &logo_url);

    error::SUCCESS.code_num
}

/// Get details for last occurred error.
///
/// This function should be called in two places to handle both cases of error occurrence:
///     1) synchronous  - in the same application thread
///     2) asynchronous - inside of function callback
///
/// NOTE: Error is stored until the next one occurs in the same execution thread or until asynchronous callback finished.
///       Returning pointer has the same lifetime.
///
/// #Params
/// * `error_json_p` - Reference that will contain error details (if any error has occurred before)
///  in the format:
/// {
///     "backtrace": Optional<str> - error backtrace.
///         Collecting of backtrace can be enabled by setting environment variable `RUST_BACKTRACE=1`
///     "message": str - human-readable error description
/// }
///
#[no_mangle]
pub extern fn vcx_get_current_error(error_json_p: *mut *const c_char) {
    trace!("vcx_get_current_error >>> error_json_p: {:?}", error_json_p);

    let error = get_current_error_c_json();
    unsafe { *error_json_p = error };

    trace!("vcx_get_current_error: <<<");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use crate::utils::libindy::{
        wallet::{import, tests::export_test_wallet},
        vdr::get_vdr,
    };
    use crate::api::return_types;
    use crate::utils::devsetup::*;
    #[cfg(feature = "pool_tests")]
    use crate::utils::libindy::vdr::tests::{get_txns, delete_test_pool};

    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    fn config() -> String {
        json!({
           "wallet_name": settings::DEFAULT_WALLET_NAME,
           "wallet_key": settings::DEFAULT_WALLET_KEY,
           "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
           "agency_endpoint" : "https://agency.com",
           "agency_did" : "72x8p4HubxzUK1dwxcc5FU",
           "remote_to_sdk_did" : "UJGjM6Cea2YVixjWwHN9wq",
           "sdk_to_remote_did" : "AB3JM851T4EQmhh8CdagSP",
           "sdk_to_remote_verkey" : "888MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
           "institution_name" : "evernym enterprise",
           "agency_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
           "remote_to_sdk_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
           "genesis_transactions": get_txns(),
           "payment_method": "null",
           "pool_config": json!({"timeout":60}).to_string()
       }).to_string()
    }

    fn _vcx_init_c_closure(path: &str) -> Result<(), u32> {
        let (h, cb, r) = return_types::return_u32();
        let path = CString::new(path.as_bytes()).unwrap();
        let rc = vcx_init(h,
                          path.as_ptr(),
                          Some(cb));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        r.recv_long()
    }

    fn _vcx_init_with_config_c_closure(config: &str) -> Result<(), u32> {
        let (h, cb, r) = return_types::return_u32();
        let config = CString::new(config.as_bytes()).unwrap();
        let rc = vcx_init_with_config(h,
                                      config.as_ptr(),
                                      Some(cb));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        r.recv_medium()
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_with_file() {
        let _setup = SetupWallet::init();

        let config = TempFile::create_with_data("test_init.json", &config());

        _vcx_init_c_closure(&config.path).unwrap();

        // Assert wallet and pool was initialized
        get_vdr().unwrap();
    }

    #[test]
    fn test_init_with_file_no_payment_method() {
        let _setup = SetupWallet::init();

        let config = json!({
            "wallet_name": settings::DEFAULT_WALLET_NAME,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        let config = TempFile::create_with_data("test_init.json", &config);

        _vcx_init_c_closure(&config.path).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_with_config() {
        let _setup = SetupWallet::init();

        _vcx_init_with_config_c_closure(&config()).unwrap();

        // Assert pool was initialized
        get_vdr().unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_fails_when_open_pool_fails() {
        let _setup = SetupWallet::init();

        // Use invalid genesis transactions
        let config = json!({
           "agency_endpoint" : "https://agency.com",
           "agency_did" : "72x8p4HubxzUK1dwxcc5FU",
           "agency_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
           "genesis_transactions": "{'date':'ds'}",
       }).to_string();

        let err = _vcx_init_with_config_c_closure(&config).unwrap_err();
        assert_eq!(err, error::POOL_LEDGER_CONNECT.code_num);
        assert!(get_vdr().is_err());

        delete_test_pool();
    }

    #[test]
    fn test_init_can_be_called_with_no_pool_config() {
        let _setup = SetupWallet::init();

        let content = json!({
            "wallet_name": settings::DEFAULT_WALLET_NAME,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        _vcx_init_with_config_c_closure(&content).unwrap();

        // assert that pool was never initialized
        assert!(get_vdr().is_err());
    }

    #[test]
    fn test_init_fails_with_no_wallet_key() {
        let _setup = SetupEmpty::init();

        let content = json!({
            "wallet_name": settings::DEFAULT_WALLET_NAME,
        }).to_string();

        let rc = _vcx_init_with_config_c_closure(&content).unwrap_err();
        assert_eq!(rc, error::MISSING_WALLET_KEY.code_num);
    }

    #[test]
    fn test_config_with_no_wallet_uses_default() {
        let _setup = SetupEmpty::init();

        assert!(settings::get_config_value(settings::CONFIG_WALLET_NAME).is_err());

        let content = json!({
            "wallet_key": "key"
        }).to_string();

        _vcx_init_with_config_c_closure(&content).unwrap_err();

        // Assert default wallet name
        assert_eq!(settings::get_config_value(settings::CONFIG_WALLET_NAME).unwrap(), settings::DEFAULT_WALLET_NAME);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_with_default_values() {
        let _setup = SetupWallet::init();

        _vcx_init_with_config_c_closure("{}").unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_called_twice_fails() {
        let _setup = SetupWallet::init();

        _vcx_init_with_config_c_closure("{}").unwrap();

        // Repeat call
        let rc = _vcx_init_with_config_c_closure("{}").unwrap_err();
        assert_eq!(rc, error::ALREADY_INITIALIZED.code_num);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_called_twice_passes_after_shutdown() {
        for _ in 0..2 {
            let _setup = SetupDefaults::init();

            wallet::create_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();

            _vcx_init_with_config_c_closure("{}").unwrap();

            //Assert config values were set correctly
            assert_eq!(settings::get_config_value("wallet_name").unwrap(), settings::DEFAULT_WALLET_NAME);

            //Verify shutdown was successful
            vcx_shutdown(true);
            assert_eq!(settings::get_config_value("wallet_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        }
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_fails_with_open_wallet() {
        let _setup = SetupLibraryWallet::init();

        let config = TempFile::create_with_data("test_init.json", &config());

        let rc = _vcx_init_c_closure(&config.path).unwrap_err();
        assert_eq!(rc, error::ALREADY_INITIALIZED.code_num);
    }

    #[test]
    fn test_init_after_importing_wallet_success() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = export_test_wallet();

        wallet::delete_wallet(&wallet_name, None, None, None).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: &wallet_name,
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_WALLET_KEY_DERIVATION: settings::DEFAULT_WALLET_KEY_DERIVATION,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
        }).to_string();
        import(&import_config).unwrap();

        let content = json!({
            "wallet_name": &wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        _vcx_init_with_config_c_closure(&content).unwrap();

        vcx_shutdown(true);
    }

    #[test]
    fn test_init_with_imported_wallet_fails_with_different_params() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = export_test_wallet();

        wallet::delete_wallet(&wallet_name, None, None, None).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_WALLET_KEY_DERIVATION: settings::DEFAULT_WALLET_KEY_DERIVATION,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
        }).to_string();
        import(&import_config).unwrap();

        let content = json!({
            "wallet_name": "different_wallet_name",
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        let err = _vcx_init_with_config_c_closure(&content).unwrap_err();
        assert_eq!(err, error::WALLET_NOT_FOUND.code_num);

        wallet::delete_wallet(&wallet_name, None, None, None).unwrap();
    }

    #[test]
    fn test_import_after_init_fails() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = export_test_wallet();

        let content = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        _vcx_init_with_config_c_closure(&content).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
        }).to_string();
        assert_eq!(import(&import_config).unwrap_err().kind(), VcxErrorKind::DuplicationWallet);

        vcx_shutdown(true);
    }

    #[test]
    fn test_init_bad_path() {
        let _setup = SetupEmpty::init();

        let rc = _vcx_init_c_closure("").unwrap_err();
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[test]
    fn test_init_no_config_path() {
        let _setup = SetupEmpty::init();

        let (h, cb, _r) = return_types::return_u32();
        assert_eq!(vcx_init(h,
                            ptr::null(),
                            Some(cb)),
                   error::INVALID_CONFIGURATION.code_num);
    }

    #[test]
    fn test_shutdown_with_no_previous_config() {
        let _setup = SetupDefaults::init();

        vcx_shutdown(true);
        vcx_shutdown(false);
    }

    #[test]
    fn test_shutdown() {
        let _setup = SetupMocks::init();

        let data = r#"["name","male"]"#;
        let connection = crate::connection::tests::build_test_connection();
        let credentialdef = crate::credential_def::create_and_publish_credentialdef("SID".to_string(), "NAME".to_string(), "4fUDR9R7fjwELRvH9JT6HH".to_string(), "id".to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let issuer_credential = crate::issuer_credential::issuer_credential_create(credentialdef, "1".to_string(), "8XFh8yBzrpJQmNyZzgoTqB".to_owned(), "credential_name".to_string(), "{\"attr\":\"value\"}".to_owned(), 1).unwrap();
        let proof = crate::proof::create_proof("1".to_string(), "[]".to_string(), "[]".to_string(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let schema = crate::schema::create_and_publish_schema("5", "VsKV7grR1BUE29mG2Fm2kX".to_string(), "name".to_string(), "0.1".to_string(), data.to_string()).unwrap();
        let disclosed_proof = crate::disclosed_proof::create_proof("id", crate::utils::constants::PROOF_REQUEST_JSON).unwrap();
        let credential = crate::credential::credential_create_with_offer("name", crate::utils::constants::CREDENTIAL_OFFER_JSON).unwrap();

        vcx_shutdown(true);
        assert_eq!(connection.release().unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(issuer_credential.release().unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(schema.release().unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(proof.release().unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(credentialdef.release().unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(credential.release().unwrap_err().kind(), VcxErrorKind::InvalidCredentialHandle);
        assert_eq!(disclosed_proof.release().unwrap_err().kind(), VcxErrorKind::InvalidDisclosedProofHandle);
        assert_eq!(wallet::get_wallet_handle(), INVALID_WALLET_HANDLE);
    }

    #[test]
    fn test_error_c_message() {
        let _setup = SetupMocks::init();

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(0)).unwrap().unwrap();
        assert_eq!(c_message, error::SUCCESS.as_str());

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1001)).unwrap().unwrap();
        assert_eq!(c_message, error::UNKNOWN_ERROR.as_str());

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(100100)).unwrap().unwrap();
        assert_eq!(c_message, error::UNKNOWN_ERROR.as_str());

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1021)).unwrap().unwrap();
        assert_eq!(c_message, error::INVALID_ATTRIBUTES_STRUCTURE.as_str());
    }

    #[test]
    fn test_vcx_version() {
        let _setup = SetupDefaults::init();

        let return_version = CStringUtils::c_str_to_string(vcx_version()).unwrap().unwrap();
        assert!(return_version.len() > 5);
    }

    #[test]
    fn test_vcx_update_institution_info() {
        let _setup = SetupDefaults::init();

        let new_name_cstr = "new_name\0";
        let new_name = &new_name_cstr[..new_name_cstr.len() - 1];
        let new_url_cstr = "http://www.evernym.com\0";
        let new_url = &new_url_cstr[..new_url_cstr.len() - 1];
        assert_ne!(new_name, &settings::get_config_value(crate::settings::CONFIG_INSTITUTION_NAME).unwrap());
        assert_ne!(new_url, &settings::get_config_value(crate::settings::CONFIG_INSTITUTION_LOGO_URL).unwrap());

        assert_eq!(error::SUCCESS.code_num, vcx_update_institution_info(new_name_cstr.as_ptr().cast(), new_url_cstr.as_ptr().cast()));

        assert_eq!(new_name, &settings::get_config_value(crate::settings::CONFIG_INSTITUTION_NAME).unwrap());
        assert_eq!(new_url, &settings::get_config_value(crate::settings::CONFIG_INSTITUTION_LOGO_URL).unwrap());
    }

    #[test]
    fn get_current_error_works_for_no_error() {
        let _setup = SetupDefaults::init();

        crate::error::reset_current_error();

        let mut error_json_p: *const c_char = ptr::null();

        vcx_get_current_error(&mut error_json_p);
        assert_eq!(None, CStringUtils::c_str_to_string(error_json_p).unwrap());
    }

    #[test]
    fn get_current_error_works_for_sync_error() {
        let _setup = SetupDefaults::init();

        crate::api::utils::vcx_provision_agent(ptr::null());

        let mut error_json_p: *const c_char = ptr::null();
        vcx_get_current_error(&mut error_json_p);
        assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
    }

    #[test]
    fn get_current_error_works_for_async_error() {
        let _setup = SetupDefaults::init();

        extern fn cb(_storage_handle: i32,
                     _err: u32,
                     _config: *const c_char) {
            let mut error_json_p: *const c_char = ptr::null();
            vcx_get_current_error(&mut error_json_p);
            assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
        }

        let config = CString::new("{}").unwrap();
        crate::api::utils::vcx_agent_provision_async(0, config.as_ptr(), Some(cb));
        ::std::thread::sleep(::std::time::Duration::from_secs(1));
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_fails_with_not_found_pool_genesis_file() {
        let _setup = SetupWallet::init();

        let content = json!({
            "genesis_path": "invalid/txn/path",
            "wallet_name": settings::DEFAULT_WALLET_NAME,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        let rc = _vcx_init_with_config_c_closure(&content).unwrap_err();
        assert_eq!(rc, error::INVALID_GENESIS_TXN_PATH.code_num);
    }
}
