use crate::utils::object_cache::Handle;
use crate::credential_def::CredentialDef;
use libc::c_char;
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use std::ptr;
use crate::credential_def;
use crate::settings;
use crate::utils::threadpool::spawn;
use crate::error::prelude::*;
use indy_sys::CommandHandle;

/// Create a new CredentialDef object and publish correspondent record on the ledger
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Enterprise's personal identification for the user.
///
/// credentialdef_name: Name of credential definition
///
/// schema_id: The schema id given during the creation of the schema
///
/// issuer_did: did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
///
/// tag: way to create a unique credential def with the same schema and issuer did.
///
/// revocation details: type-specific configuration of credential definition revocation
///     TODO: Currently supports ISSUANCE BY DEFAULT, support for ISSUANCE ON DEMAND will be added as part of ticket: IS-1074
///     support_revocation: true|false - Optional, by default its false
///     tails_file: path to tails file - Optional if support_revocation is false
///     max_creds: size of tails file - Optional if support_revocation is false
/// # Examples config ->  "{}" | "{"support_revocation":false}" | "{"support_revocation":true, "tails_file": "/tmp/tailsfile.txt", "max_creds": 1}"
/// cb: Callback that provides CredentialDef handle and error status of request.
///
/// payment_handle: future use (currently uses any address in wallet)
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_create(command_handle: CommandHandle,
                                       source_id: *const c_char,
                                       credentialdef_name: *const c_char,
                                       schema_id: *const c_char,
                                       issuer_did: *const c_char,
                                       tag: *const c_char,
                                       revocation_details: *const c_char,
                                       _payment_handle: u32,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: Handle<CredentialDef>)>) -> u32 {
    info!("vcx_credentialdef_create >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credentialdef_name, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tag, VcxErrorKind::InvalidOption);
    check_useful_c_str!(revocation_details, VcxErrorKind::InvalidOption);

    let issuer_did: String = if !issuer_did.is_null() {
        check_useful_c_str!(issuer_did, VcxErrorKind::InvalidOption);
        issuer_did.to_owned()
    } else {
        match settings::get_config_value(settings::CONFIG_INSTITUTION_DID) {
            Ok(x) => x,
            Err(x) => return x.into(),
        }
    };

    trace!("vcx_credential_def_create(command_handle: {}, source_id: {}, credentialdef_name: {} schema_id: {}, issuer_did: {}, tag: {}, revocation_details: {:?})",
           command_handle,
           source_id,
           secret!(credentialdef_name),
           secret!(schema_id),
           secret!(issuer_did),
           secret!(tag),
           secret!(revocation_details));

    spawn(move || {
        let (rc, handle) = match credential_def::create_and_publish_credentialdef(source_id,
                                                                                  credentialdef_name,
                                                                                  issuer_did,
                                                                                  schema_id,
                                                                                  tag,
                                                                                  revocation_details) {
            Ok(x) => {
                trace!("vcx_credential_def_create_cb(command_handle: {}, rc: {}, credentialdef_handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_credential_def_create_cb(command_handle: {}, rc: {}, credentialdef_handle: {})",
                      command_handle, x, 0);
                (x.into(), Handle::dummy())
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}
/// Create a new CredentialDef object from a cred_def_id
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Enterprise's personal identification for the user.
///
/// cred_def_id: reference to already created cred def
///
/// issuer_did: did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
///
/// revocation_config: Information given during the initial create of the cred def if revocation was enabled
///  {
///     tails_file: Option<String>,  // Path to tails file
///     rev_reg_id: Option<String>,
///     rev_reg_def: Option<String>,
///     rev_reg_entry: Option<String>,
///  }
///
/// cb: Callback that provides CredentialDef handle and error status of request.
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_create_with_id(command_handle: CommandHandle,
                                               source_id: *const c_char,
                                               cred_def_id: *const c_char,
                                               issuer_did: *const c_char,
                                               revocation_config: *const c_char,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: Handle<CredentialDef>)>) -> u32 {
    info!("vcx_credentialdef_create_with_id >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(cred_def_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(revocation_config, VcxErrorKind::InvalidOption);

    let issuer_did: String = if !issuer_did.is_null() {
        check_useful_c_str!(issuer_did, VcxErrorKind::InvalidOption);
        issuer_did.to_owned()
    } else {
        match settings::get_config_value(settings::CONFIG_INSTITUTION_DID) {
            Ok(x) => x,
            Err(x) => return x.into(),
        }
    };

    trace!("vcx_credentialdef_create_with_id(command_handle: {}, source_id: {}, cred_def_id: {} issuer_did: {}, revocation_config: {:?})",
           command_handle,
           source_id,
           secret!(cred_def_id),
           secret!(issuer_did),
           secret!(revocation_config)
    );

    spawn(move|| {
        let ( rc, handle) = match credential_def::create_credentialdef_from_id(source_id,
                                                                               cred_def_id,
                                                                               issuer_did,
                                                                               revocation_config ) {
            Ok(x) => {
                trace!("vcx_credentialdef_create_with_id_cb(command_handle: {}, rc: {}, credentialdef_handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_credentialdef_create_with_id(command_handle: {}, rc: {}, credentialdef_handle: {})",
                      command_handle, x, 0);
                (x.into(), Handle::dummy())
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a new CredentialDef object that will be published by Endorser later.
///
/// Note that CredentialDef can't be used for credential issuing until it will be published on the ledger.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Enterprise's personal identification for the user.
///
/// credentialdef_name: Name of credential definition
///
/// schema_id: The schema id given during the creation of the schema
///
/// issuer_did: did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
///
/// tag: way to create a unique credential def with the same schema and issuer did.
///
/// revocation details: type-specific configuration of credential definition revocation
///     TODO: Currently supports ISSUANCE BY DEFAULT, support for ISSUANCE ON DEMAND will be added as part of ticket: IS-1074
///     support_revocation: true|false - Optional, by default its false
///     tails_file: path to tails file - Optional if support_revocation is false
///     max_creds: size of tails file - Optional if support_revocation is false
///
/// endorser: DID of the Endorser that will submit the transaction.
///
/// # Examples config ->  "{}" | "{"support_revocation":false}" | "{"support_revocation":true, "tails_file": "/tmp/tailsfile.txt", "max_creds": 1}"
/// cb: Callback that provides CredentialDef handle, transactions (CredentialDef, Option<RevocRegDef>, Option<RevocRegEntry>) that should be passed to Endorser for publishing.
///
/// payment_handle: future use (currently uses any address in wallet)
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_prepare_for_endorser(command_handle: CommandHandle,
                                                     source_id: *const c_char,
                                                     credentialdef_name: *const c_char,
                                                     schema_id: *const c_char,
                                                     issuer_did: *const c_char,
                                                     tag: *const c_char,
                                                     revocation_details: *const c_char,
                                                     endorser: *const c_char,
                                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32,
                                                                          credentialdef_handle: Handle<CredentialDef>,
                                                                          credentialdef_transaction: *const c_char,
                                                                          rev_reg_def_transaction: *const c_char,
                                                                          rev_reg_entry_transaction: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_prepare_for_endorser >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credentialdef_name, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(schema_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tag, VcxErrorKind::InvalidOption);
    check_useful_c_str!(endorser, VcxErrorKind::InvalidOption);
    check_useful_c_str!(revocation_details, VcxErrorKind::InvalidOption);

    let issuer_did: String = if !issuer_did.is_null() {
        check_useful_c_str!(issuer_did, VcxErrorKind::InvalidOption);
        issuer_did.to_owned()
    } else {
        match settings::get_config_value(settings::CONFIG_INSTITUTION_DID) {
            Ok(x) => x,
            Err(x) => return x.into(),
        }
    };

    trace!("vcx_credentialdef_prepare_for_endorser(command_handle: {}, source_id: {}, credentialdef_name: {} schema_id: {}, issuer_did: {}, tag: {}, revocation_details: {:?}, endorser: {:?})",
           command_handle,
           source_id,
           secret!(credentialdef_name),
           secret!(schema_id),
           secret!(issuer_did),
           secret!(tag),
           secret!(revocation_details),
           secret!(endorser));

    spawn(move || {
        match credential_def::prepare_credentialdef_for_endorser(source_id,
                                                                 credentialdef_name,
                                                                 issuer_did,
                                                                 schema_id,
                                                                 tag,
                                                                 revocation_details,
                                                                 endorser) {
            Ok((handle, cred_def_req, rev_reg_def_req, rev_reg_entry_req)) => {
                trace!(target: "vcx", "vcx_credentialdef_prepare_for_endorser(command_handle: {}, rc: {}, handle: {}, cred_def_req: {}, rev_reg_def_req: {:?}, rev_reg_entry_req: {:?})",
                       command_handle, error::SUCCESS.as_str(), handle, secret!(cred_def_req), secret!(rev_reg_def_req), secret!(rev_reg_entry_req));
                let cred_def_req = CStringUtils::string_to_cstring(cred_def_req);
                let rev_reg_def_req = rev_reg_def_req.map(CStringUtils::string_to_cstring);
                let rev_reg_entry_req = rev_reg_entry_req.map(CStringUtils::string_to_cstring);

                cb(command_handle, error::SUCCESS.code_num, handle, cred_def_req.as_ptr(),
                   rev_reg_def_req.as_ref().map(|def| def.as_ptr()).unwrap_or(ptr::null()),
                   rev_reg_entry_req.as_ref().map(|entry| entry.as_ptr()).unwrap_or(ptr::null()));
            }
            Err(x) => {
                warn!("vcx_credentialdef_prepare_for_endorser(command_handle: {}, rc: {}, handle: {}, cred_def_req: {}, cred_def_req: {:?}, cred_def_req: {:?}) source_id: {}",
                      command_handle, x, 0, "", "", "", "");
                cb(command_handle, x.into(), Handle::dummy(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes the credentialdef object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credentialdef_handle: Credentialdef handle that was provided during creation. Used to access credentialdef object
///
/// cb: Callback that provides json string of the credentialdef's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_serialize(command_handle: CommandHandle,
                                          credentialdef_handle: Handle<CredentialDef>,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_state: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credentialdef_serialize(command_handle: {}, credentialdef_handle: {})",
           command_handle, credentialdef_handle);

    spawn(move || {
        match credentialdef_handle.to_string() {
            Ok(x) => {
                trace!("vcx_credentialdef_serialize_cb(command_handle: {}, credentialdef_handle: {}, rc: {}, state: {})",
                       command_handle, credentialdef_handle, error::SUCCESS.as_str(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_credentialdef_serialize_cb(command_handle: {}, credentialdef_handle: {}, rc: {}, state: {})",
                      command_handle, credentialdef_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            },
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing a credentialdef object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credentialdef_data: json string representing a credentialdef object
///
/// cb: Callback that provides credentialdef handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_deserialize(command_handle: CommandHandle,
                                            credentialdef_data: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credentialdef_handle: Handle<CredentialDef>)>) -> u32 {
    info!("vcx_credentialdef_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credentialdef_data, VcxErrorKind::InvalidOption);

    trace!("vcx_credentialdef_deserialize(command_handle: {}, credentialdef_data: {})",
           command_handle, secret!(credentialdef_data));

    spawn(move || {
        let (rc, handle) = match credential_def::from_string(&credentialdef_data) {
            Ok(x) => {
                trace!("vcx_credentialdef_deserialize_cb(command_handle: {}, rc: {}, handle: {}),",
                       command_handle, error::SUCCESS.as_str(), x);
                (error::SUCCESS.code_num, x)
            },
            Err(e) => {
                warn!("vcx_credentialdef_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, e, 0);
                (e.into(), Handle::dummy())
            },
        };
        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves credential definition's id
///
/// #Params
/// cred_def_handle: CredDef handle that was provided during creation. Used to access proof object
///
/// cb: Callback that provides credential definition id and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_get_cred_def_id(command_handle: CommandHandle,
                                                cred_def_handle: Handle<CredentialDef>,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, cred_def_id: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_get_cred_def_id >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {})", command_handle, cred_def_handle);

    spawn(move || {
        match cred_def_handle.get_cred_def_id() {
            Ok(x) => {
                trace!("vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {}, rc: {}, cred_def_id: {})",
                       command_handle, cred_def_handle, error::SUCCESS.as_str(),  secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_credentialdef_get_cred_def_id(command_handle: {}, cred_def_handle: {}, rc: {}, cred_def_id: {})",
                      command_handle, cred_def_handle, x, "");
                cb(command_handle, x.into(), ptr::null_mut());
            },
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the payment transaction information generated when paying the ledger fee
///
/// #param
/// handle: credential_def handle that was provided during creation.  Used to access credential_def object.
///
/// #Callback returns
/// PaymentTxn json
/// example: {
///         "amount":25,
///         "inputs":[
///             "pay:null:1_3FvPC7dzFbQKzfG"
///         ],
///         "outputs":[
///             {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null}
///         ]
///     }
#[no_mangle]
pub extern fn vcx_credentialdef_get_payment_txn(command_handle: CommandHandle,
                                                handle: Handle<CredentialDef>,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, txn: *const c_char)>) -> u32 {
    info!("vcx_credentialdef_get_payment_txn >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credentialdef_get_payment_txn(command_handle: {})", command_handle);

    spawn(move || {
        match handle.get_cred_def_payment_txn() {
            Ok(x) => {
                match serde_json::to_string(&x) {
                    Ok(x) => {
                        trace!("vcx_credentialdef_get_payment_txn_cb(command_handle: {}, rc: {}, : {})",
                               command_handle, error::SUCCESS.as_str(), secret!(x));

                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, 0, msg.as_ptr());
                    }
                    Err(e) => {
                        let err = VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize payment txn. Err: {:?}", e));
                        error!("vcx_credentialdef_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {})",
                               command_handle, err, "null");
                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                }
            },
            Err(x) => {
                error!("vcx_credentialdef_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {})",
                       command_handle, x, "null");
                cb(command_handle, x.into(), ptr::null());
            },
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the credentialdef object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access credential object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_credentialdef_release(credentialdef_handle: Handle<CredentialDef>) -> u32 {
    info!("vcx_credentialdef_release >>>");

    spawn(move || {
        match credentialdef_handle.release() {
            Ok(()) => {
                trace!("vcx_credentialdef_release(credentialdef_handle: {}, rc: {})",
                       credentialdef_handle, error::SUCCESS.as_str());
            }
            Err(_e) => {
                // FIXME logging here results in panic while python tests
                // warn!("vcx_credentialdef_release(credentialdef_handle: {}), rc: {})",
                //       credentialdef_handle, e);
            }
        };
        Ok(())
    });
    error::SUCCESS.code_num
}

/// Checks if credential definition is published on the Ledger and updates the state if it is.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credentialdef_handle: Credentialdef handle that was provided during creation. Used to access credentialdef object
///
/// cb: Callback that provides most current state of the credential definition and error status of request
///     States:
///         0 = Built
///         1 = Published
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_update_state(command_handle: CommandHandle,
                                             credentialdef_handle: Handle<CredentialDef>,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credentialdef_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credentialdef_update_state(command_handle: {}, credentialdef_handle: {})",
           command_handle, credentialdef_handle);

    spawn(move || {
        match credentialdef_handle.update_state() {
            Ok(state) => {
                trace!("vcx_credentialdef_update_state(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), state);
                cb(command_handle, error::SUCCESS.code_num, state);
            }
            Err(x) => {
                warn!("vcx_credentialdef_update_state(command_handle: {}, rc: {}, state: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the current state of the credential definition object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credentialdef_handle: Credentialdef handle that was provided during creation. Used to access credentialdef object
///
/// cb: Callback that provides most current state of the credential definition and error status of request
///     States:
///         0 = Built
///         1 = Published
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credentialdef_get_state(command_handle: CommandHandle,
                                          credentialdef_handle: Handle<CredentialDef>,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credentialdef_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credentialdef_get_state(command_handle: {}, credentialdef_handle: {})",
           command_handle, credentialdef_handle);

    spawn(move || {
        match credentialdef_handle.get_state() {
            Ok(state) => {
                trace!("vcx_credentialdef_get_state(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), state);
                cb(command_handle, error::SUCCESS.code_num, state);
            }
            Err(x) => {
                warn!("vcx_credentialdef_get_state(command_handle: {}, rc: {}, state: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use crate::settings;
    use crate::api::return_types;
    use crate::utils::constants::{SCHEMA_ID, SCHEMA_ID_CSTR};
    use crate::utils::devsetup::*;

    const TEST_SOURCE_ID: *const c_char = "Test Source ID\0".as_ptr().cast();
    const TEST_CRED_DEF: *const c_char = "Test Credential Def\0".as_ptr().cast();
    const EMPTY_JSON: *const c_char = "{}\0".as_ptr().cast();
    const TAG: *const c_char = "tag\0".as_ptr().cast();
    const ISSUER_DID: *const c_char = "6vkhW3L28AophhA68SSzRS\0".as_ptr().cast();

    #[test]
    fn test_vcx_create_credentialdef_success() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_cdh();
        assert_eq!(vcx_credentialdef_create(h,
                                            TEST_SOURCE_ID,
                                            TEST_CRED_DEF,
                                            SCHEMA_ID_CSTR,
                                            ISSUER_DID,
                                            TAG,
                                            EMPTY_JSON,
                                            0,
                                            Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_create_credentialdef_fails() {
        let _setup = SetupLibraryWallet::init();

        settings::set_defaults();
        let (h, cb, r) = return_types::return_u32_cdh();
        assert_eq!(vcx_credentialdef_create(h,
                                            TEST_SOURCE_ID,
                                            TEST_CRED_DEF,
                                            SCHEMA_ID_CSTR,
                                            ptr::null(),
                                            TAG,
                                            EMPTY_JSON,
                                            0,
                                            Some(cb)), error::SUCCESS.code_num);
        assert!(r.recv_medium().is_err());
    }
    #[test]
    fn test_vcx_create_credentialdef_from_id_success() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_cdh();
        assert_eq!(vcx_credentialdef_create_with_id(h,
                                            TEST_SOURCE_ID,
                                            TEST_CRED_DEF,
                                            ISSUER_DID,
                                            ptr::null(),
                                            Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_credentialdef_serialize() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_cdh();
        assert_eq!(vcx_credentialdef_create(h,
                                            TEST_SOURCE_ID,
                                            TEST_CRED_DEF,
                                            SCHEMA_ID_CSTR,
                                            ptr::null(),
                                            TAG,
                                            EMPTY_JSON ,
                                            0,
                                            Some(cb)), error::SUCCESS.code_num);

        let handle = r.recv_medium().unwrap();
        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_credentialdef_serialize(h, handle, Some(cb)), error::SUCCESS.code_num);
        let cred = r.recv_medium().unwrap();
        assert!(cred.is_some());
    }

    #[test]
    fn test_vcx_credentialdef_deserialize_succeeds() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_cdh();

        let original = concat!(r#"{"version":"1.0", "data": {"id":"2hoqvcwupRTUNkXn6ArYzs:3:CL:1697","issuer_did":"2hoqvcwupRTUNkXn6ArYzs","tag":"tag","name":"Test Credential Definition","rev_ref_def":null,"rev_reg_entry":null,"rev_reg_id":null,"source_id":"SourceId"}}"#, "\0").as_ptr().cast();
        assert_eq!(vcx_credentialdef_deserialize(h,
                                                 original,
                                                 Some(cb)), error::SUCCESS.code_num);

        let handle = r.recv_short().unwrap();
        assert!(handle > 0);

    }

    #[test]
    fn test_vcx_credentialdef_deserialize_succeeds_with_old_data() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_cdh();

        let original = concat!(r#"{"data":{"id":"V4SGRU86Z58d6TV7PBUe6f:3:CL:912:tag1","name":"color","payment_txn":null,"source_id":"1","tag":"tag1"},"version":"1.0"}"#, "\0")
            .as_ptr()
            .cast();
        assert_eq!(vcx_credentialdef_deserialize(h,
                                                 original,
                                                 Some(cb)), error::SUCCESS.code_num);

        let handle = r.recv_short().unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_credentialdef_release() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_cdh();
        assert_eq!(vcx_credentialdef_create(h,
                                            "Test Source ID Release Test\0".as_ptr().cast(),
                                            "Test Credential Def Release\0".as_ptr().cast(),
                                            SCHEMA_ID_CSTR,
                                            ptr::null(),
                                            TAG,
                                            EMPTY_JSON,
                                            0,
                                            Some(cb)), error::SUCCESS.code_num);

        r.recv_medium().unwrap();
    }


    #[test]
    fn test_vcx_creddef_get_id() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_cdh();
        assert_eq!(vcx_credentialdef_create(h,
                                            TEST_SOURCE_ID,
                                            TEST_CRED_DEF,
                                            SCHEMA_ID_CSTR,
                                            ISSUER_DID,
                                            TAG,
                                            EMPTY_JSON,
                                            0,
                                            Some(cb)), error::SUCCESS.code_num);
        let handle = r.recv_medium().unwrap();
        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_credentialdef_get_cred_def_id(h, handle, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_prepare_cred_def_success() {
        let _setup = SetupMocks::init();

        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        let (h, cb, r) = return_types::return_u32_cdh_str_str_str();
        assert_eq!(vcx_credentialdef_prepare_for_endorser(h,
                                            TEST_SOURCE_ID,
                                            TEST_CRED_DEF,
                                            SCHEMA_ID_CSTR,
                                            ISSUER_DID,
                                            TAG,
                                            EMPTY_JSON,
                                            "V4SGRU86Z58d6TV7PBUe6f\0".as_ptr().cast(),
                                            Some(cb)), error::SUCCESS.code_num);
        let (_handle, cred_def_transaction, rev_reg_def_transaction, rev_reg_delta_transaction) = r.recv_short().unwrap();
        let cred_def_transaction = cred_def_transaction.unwrap();
        let cred_def_transaction: serde_json::Value = serde_json::from_str(&cred_def_transaction).unwrap();
        let expected_cred_def_transaction: serde_json::Value = serde_json::from_str(crate::utils::constants::REQUEST_WITH_ENDORSER).unwrap();
        assert_eq!(expected_cred_def_transaction, cred_def_transaction);
        assert!(rev_reg_def_transaction.is_none());
        assert!(rev_reg_delta_transaction.is_none());
    }

    #[test]
    fn test_vcx_prepare_cred_def_with_revocation_success() {
        let _setup = SetupMocks::init();

        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        let (h, cb, r) = return_types::return_u32_cdh_str_str_str();
        let details = CString::new(credential_def::tests::revocation_details(true).to_string()).unwrap();
        assert_eq!(vcx_credentialdef_prepare_for_endorser(h,
                                            TEST_SOURCE_ID,
                                            TEST_CRED_DEF,
                                            SCHEMA_ID_CSTR,
                                            ISSUER_DID,
                                            TAG,
                                            details.as_ptr(),
                                            "V4SGRU86Z58d6TV7PBUe6f\0".as_ptr().cast(),
                                            Some(cb)), error::SUCCESS.code_num);
        let (_handle, cred_def_transaction, rev_reg_def_transaction, rev_reg_delta_transaction) = r.recv_short().unwrap();
        let cred_def_transaction = cred_def_transaction.unwrap();
        let cred_def_transaction: serde_json::Value = serde_json::from_str(&cred_def_transaction).unwrap();
        let expected_cred_def_transaction: serde_json::Value = serde_json::from_str(crate::utils::constants::REQUEST_WITH_ENDORSER).unwrap();
        assert_eq!(expected_cred_def_transaction, cred_def_transaction);
        assert!(rev_reg_def_transaction.is_some());
        assert!(rev_reg_delta_transaction.is_some());
    }

    #[test]
    fn test_vcx_cred_def_get_state() {
        let _setup = SetupMocks::init();

        let (handle, _, _, _) = credential_def::prepare_credentialdef_for_endorser("testid".to_string(),
                                                                                   "Test Credential Def".to_string(),
                                                                                   "6vkhW3L28AophhA68SSzRS".to_string(),
                                                                                   SCHEMA_ID.to_string(),
                                                                                   "tag".to_string(),
                                                                                   "{}".to_string(),
                                                                                   "V4SGRU86Z58d6TV7PBUe6f".to_string()).unwrap();
        {
            let (h, cb, r) = return_types::return_u32_u32();
            let _rc = vcx_credentialdef_get_state(h, handle, Some(cb));
            assert_eq!(r.recv_medium().unwrap(), crate::api::PublicEntityStateType::Built as u32)
        }
        {
            let (h, cb, r) = return_types::return_u32_u32();
            let _rc = vcx_credentialdef_update_state(h, handle, Some(cb));
            assert_eq!(r.recv_medium().unwrap(), crate::api::PublicEntityStateType::Published as u32);
        }
        {
            let (h, cb, r) = return_types::return_u32_u32();
            let _rc = vcx_credentialdef_get_state(h, handle, Some(cb));
            assert_eq!(r.recv_medium().unwrap(), crate::api::PublicEntityStateType::Published as u32)
        }
    }
}
