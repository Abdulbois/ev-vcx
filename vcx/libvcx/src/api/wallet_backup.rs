use libc::c_char;
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use crate::utils::threadpool::spawn;
use crate::error::prelude::*;
use crate::wallet_backup::{WalletBackup, create_wallet_backup, from_string, restore_wallet};
use crate::utils::object_cache::Handle;
use crate::agent::messages::get_message::Message;
use std::ptr;
use vdrtools_sys::CommandHandle;

/// -> Create a Wallet Backup object that provides a Cloud wallet backup and provision's backup protocol with Agent
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: institution's personal identification for the user
///
/// wallet_encryption_key: String representing the User's Key for securing (encrypting) the exported Wallet.
///
/// cb: Callback that provides wallet_backup handle and error status of request
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_wallet_backup_create(command_handle: CommandHandle,
                                       source_id: *const c_char,
                                       wallet_encryption_key: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, wallet_backup_handle: Handle<WalletBackup>)>) -> u32 {
    info!("vcx_wallet_backup_create >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(wallet_encryption_key, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_backup_create(command_handle: {}, source_id: {}, wallet_backup_key: {})",
           command_handle, source_id, secret!(wallet_encryption_key));

    spawn(move || {
        match create_wallet_backup(&source_id, &wallet_encryption_key) {
            Ok(handle) => {
                trace!("vcx_wallet_backup_create(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_wallet_backup_create(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Wallet Backup to the Cloud
///
/// #Params:
/// command_handle: Handle for User's Reference only.
/// wallet_backup_handle: Wallet Backup handle that was provided during creation. Used to access object
/*
    Todo: path is needed because the only exposed libindy functionality for exporting
    an encrypted wallet, writes it to the file system. A possible better way is for libindy's export_wallet
    to optionally return an encrypted stream of bytes instead of writing it to the fs. This could also
    be done in a separate libindy api call if necessary.
 */
/// Todo: path will not be necessary when libindy functionality for wallet export functionality is expanded
/// Todo: path must be different than other exported wallets because this instance is deleted after its uploaded to the cloud
/// path: Path to export wallet to User's File System. (This instance of the export
/// cb: Callback that provides the success/failure of the api call.
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
#[no_mangle]
pub extern fn vcx_wallet_backup_backup(command_handle: CommandHandle,
                                       wallet_backup_handle: Handle<WalletBackup>,
                                       path: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_wallet_backup_backup >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(path,  VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_backup_backup(command_handle: {}, wallet_backup_handle: {}, path: {})",
           command_handle, wallet_backup_handle, secret!(path));

    spawn(move || {
        trace!("vcx_wallet_backup_backup(command_handle: {}, wallet_backup_handle: {}, path: {})",
               command_handle, wallet_backup_handle, path);
        match wallet_backup_handle.backup_wallet(&path) {
            Ok(_) => {
                let return_code = error::SUCCESS.code_num;
                trace!("vcx_wallet_backup_backup(command_handle: {}, rc: {})", command_handle, return_code);
                cb(command_handle, return_code);
            }
            Err(e) => {
                warn!("vcx_wallet_backup_backup(command_handle: {}, rc: {})", command_handle, e);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Checks for any state change and updates the the state attribute
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// wallet_backup_handle: was provided during creation. Used to identify connection object
///
/// cb: Callback that provides most current state of the wallet_backup and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_backup_update_state(command_handle: CommandHandle,
                                             wallet_backup_handle: Handle<WalletBackup>,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_wallet_backup_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_backup_update_state(command_handle: {}, wallet_backup: {})",
           command_handle, wallet_backup_handle);

    spawn(move || {
        match wallet_backup_handle.update_state(None) {
            Ok(x) => {
                trace!("vcx_wallet_backup_update_state(command_handle: {}, rc: {}, wallet_backup_handle: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), wallet_backup_handle, wallet_backup_handle.get_state());
                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(x) => {
                warn!("vcx_wallet_backup_update_state(command_handle: {}, rc: {}, wallet_backup_handle: {}, state: {})",
                      command_handle, x, wallet_backup_handle, wallet_backup_handle.get_state());
                cb(command_handle, x.into(), 0);
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Checks the message any state change and updates the the state attribute
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// wallet_backup_handle: was provided during creation. Used to identify connection object
///
/// message: message to process
///
/// cb: Callback that provides most current state of the wallet_backup and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_backup_update_state_with_message(command_handle: CommandHandle,
                                                          wallet_backup_handle: Handle<WalletBackup>,
                                                          message: *const c_char,
                                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_wallet_backup_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_backup_update_state_with_message(command_handle: {}, wallet_backup: {}, message: {})",
           command_handle, wallet_backup_handle, secret!(message));

    let message: Message = match serde_json::from_str(&message) {
        Ok(x) => x,
        Err(_) => return VcxError::from(VcxErrorKind::InvalidJson).into(),
    };

    spawn(move || {
        match wallet_backup_handle.update_state(Some(message)) {
            Ok(x) => {
                trace!("vcx_wallet_backup_update_state_with_message(command_handle: {}, rc: {}, wallet_backup_handle: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), wallet_backup_handle, wallet_backup_handle.get_state());
                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(x) => {
                warn!("vcx_wallet_backup_update_state_with_message(command_handle: {}, rc: {}, wallet_backup_handle: {}, state: {})",
                      command_handle, x, wallet_backup_handle, wallet_backup_handle.get_state());
                cb(command_handle, x.into(), 0);
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes the wallet backup object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Wallet Backup handle that was provided during creation. Used to identify the wallet backup object
///
/// cb: Callback that provides json string of the wallet backup's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_backup_serialize(command_handle: CommandHandle,
                                          wallet_backup_handle: Handle<WalletBackup>,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>) -> u32 {
    info!("vcx_wallet_backup_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_backup_serialize(command_handle: {}, proof_handle: {})",
           command_handle, wallet_backup_handle);

    spawn(move || {
        match wallet_backup_handle.to_string() {
            Ok(x) => {
                trace!("vcx_wallet_backup_serialize_cb(command_handle: {}, rc: {}, data: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_wallet_backup_serialize_cb(command_handle: {}, rc: {}, data: {})",
                       command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing an wallet backup object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// data: json string representing a wallet backup object
///
///
/// cb: Callback that provides handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_backup_deserialize(command_handle: CommandHandle,
                                            wallet_backup_str: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: Handle<WalletBackup>)>) -> u32 {
    info!("vcx_wallet_backup_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(wallet_backup_str, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_backup_deserialize(command_handle: {}, proof_data: {})",
           command_handle, secret!(wallet_backup_str));

    spawn(move || {
        match from_string(&wallet_backup_str) {
            Ok(x) => {
                trace!("vcx_wallet_backup_deserialize_cb(command_handle: {}, rc: {}, wallet_backup_handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);

                cb(command_handle, 0, x);
            }
            Err(x) => {
                error!("vcx_wallet_backup_deserialize_cb(command_handle: {}, rc: {}, wallet_backup_handle: {})",
                       command_handle, x, 0);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Requests a recovery of a backup previously stored with a cloud agent
///
/// config: "{"wallet_name":"","wallet_key":"","exported_wallet_path":"","backup_key":"","key_derivation":""}"
/// backup_key: Key used when creating the backup of the wallet (For encryption/decrption)
/// cb: Callback that provides the success/failure of the api call.
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
#[no_mangle]
pub extern fn vcx_wallet_backup_restore(command_handle: u32,
                                        config: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_wallet_backup_recovery >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(config,  VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_backup_recovery(command_handle: {}, config: {})",
           command_handle, secret!(config));

    spawn(move || {
        match restore_wallet(&config) {
            Ok(_) => {
                trace!("vcx_wallet_backup_recovery(command_handle: {}, rc: {})", command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_wallet_backup_recovery(command_handle: {}, rc: {})", command_handle, e);
                cb(command_handle, e.into());
            }
        };
        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(all(test, feature = "wallet_backup"))]
mod tests {
    use super::*;
    use std::ffi::CString;
    use crate::utils::error;
    use std::time::Duration;
    use crate::api::return_types;
    use std::ptr;
    use serde_json::Value;
    use crate::wallet_backup;
    use crate::utils::devsetup::SetupMocks;

    const PW: *const c_char = "pass_phrae\0".as_ptr().cast();
    const TEST_CREATE: *const c_char = "test_create\0".as_ptr().cast();
    const ENCRYPTION_KEY: *const c_char = "encryption_key\0".as_ptr().cast();

    #[test]
    fn test_vcx_wallet_backup_create() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_wh();
        let rc = vcx_wallet_backup_create(h,
                                          TEST_CREATE,
                                          PW,
                                          Some(cb));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert!(r.recv_with(Duration::from_secs(10)).unwrap() > 0);
    }

    #[test]
    fn test_vcx_wallet_backup_create_fails() {
        let _setup = SetupMocks::init();

        let rc = vcx_wallet_backup_create(0,
                                          "test_create_fails\0".as_ptr().cast(),
                                          PW,
                                          None);
        assert_eq!(rc, error::INVALID_OPTION.code_num);
        let (h, cb, _r) = return_types::return_u32_wh();
        let rc = vcx_wallet_backup_create(h,
                                          ptr::null(),
                                          PW,
                                          Some(cb));
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[test]
    fn test_wallet_backup() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_wh();
        vcx_wallet_backup_create(h,
                                 TEST_CREATE,
                                 ENCRYPTION_KEY,
                                 Some(cb));
        let wallet_handle = r.recv_long().unwrap();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_backup_backup(h,
                                            wallet_handle,
                                            "path\0".as_ptr().cast(),
                                            Some(cb)), error::SUCCESS.code_num);
        r.recv_long().unwrap();
    }

    #[test]
    fn test_vcx_wallet_backup_serialize_and_deserialize() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_str();
        let handle = wallet_backup::create_wallet_backup("abc", "encryption_key").unwrap();
        assert_eq!(vcx_wallet_backup_serialize(h,
                                               handle,
                                               Some(cb)), error::SUCCESS.code_num);
        let s = r.recv_with(Duration::from_secs(2)).unwrap().unwrap();
        let j: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(j["version"], crate::utils::constants::DEFAULT_SERIALIZE_VERSION);

        let (h, cb, r) = return_types::return_u32_wh();
        let cstr = CString::new(s).unwrap();
        assert_eq!(vcx_wallet_backup_deserialize(h,
                                                 cstr.as_ptr(),
                                                 Some(cb)),
                   error::SUCCESS.code_num);

        let handle = r.recv_with(Duration::from_secs(2)).unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_wallet_backup_update_state() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_wh();
        vcx_wallet_backup_create(h,
                                 TEST_CREATE,
                                 ENCRYPTION_KEY,
                                 Some(cb));
        let wallet_handle = r.recv_long().unwrap();

        crate::utils::httpclient::AgencyMock::set_next_response(&[]);
        let (h, cb, r) = return_types::return_u32_u32();
        assert_eq!(vcx_wallet_backup_update_state(h,
                                                  wallet_handle,
                                                  Some(cb)), error::SUCCESS.code_num);
        let state = r.recv_long().unwrap();
        assert_eq!(state, crate::api::WalletBackupState::InitRequested as u32)
    }
}
