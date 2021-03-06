use libc::c_char;
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use crate::utils::libindy::payments::{pay_a_payee, get_wallet_token_info, create_address, sign_with_address, verify_with_address};
use crate::utils::libindy::wallet::{export, import, get_wallet_handle};
use crate::utils::libindy::wallet;
use crate::utils::threadpool::spawn;
use std::{thread, ptr};
use std::ptr::null;
use crate::error::prelude::*;
use crate::indy::{CommandHandle, SearchHandle};

const EMPTY_CSTR: *const c_char = "\0".as_ptr().cast();

/// Get the total balance from all addresses contained in the configured wallet
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// payment_handle: for future use
///
/// cb: Callback that provides error status of function execution and wallet balance
///
/// # Example info -> "{"balance":6,"balance_str":"6","addresses":[{"address":"pay:null:9UFgyjuJxi1i1HD","balance":3,"utxo":[{"source":"pay:null:1","paymentAddress":"pay:null:zR3GN9lfbCVtHjp","amount":1,"extra":"yqeiv5SisTeUGkw"}]}]}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_get_token_info(command_handle: CommandHandle,
                                        payment_handle: u32,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, *const c_char)>) -> u32 {
    info!("vcx_wallet_get_token_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!("vcx_wallet_get_token_info(command_handle: {}, payment_handle: {})",
           command_handle, payment_handle);

    spawn(move || {
        match get_wallet_token_info() {
            Ok(x) => {
                trace!("vcx_wallet_get_token_info_cb(command_handle: {}, rc: {}, info: {})",
                       command_handle, 0, secret!(x));

                let msg = CStringUtils::string_to_cstring(x.to_string());
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_wallet_get_token_info_cb(command_handle: {}, rc: {}, info: {})",
                      command_handle, x, "null");

                cb(command_handle, x.into(), EMPTY_CSTR);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Add a payment address to the wallet
///
/// #params
///
/// cb: Callback that provides error status of function execution and payment address info
///
/// # Example payment_address -> "pay:null:9UFgyjuJxi1i1HD"
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_wallet_create_payment_address(command_handle: CommandHandle,
                                                seed: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, address: *const c_char)>) -> u32 {
    info!("vcx_wallet_create_payment_address >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    let seed = if !seed.is_null() {
        check_useful_opt_c_str!(seed, VcxErrorKind::InvalidOption);
        seed
    } else {
        None
    };

    trace!("vcx_wallet_create_payment_address(command_handle: {}, seed: {:?})",
           command_handle, secret!(seed));

    spawn(move || {
        match create_address(seed) {
            Ok(x) => {
                trace!("vcx_wallet_create_payment_address_cb(command_handle: {}, rc: {}, address: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(x));

                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_wallet_create_payment_address_cb(command_handle: {}, rc: {}, address: {})",
                      command_handle, x, "null");

                cb(command_handle, x.into(), EMPTY_CSTR);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Signs a message with a payment address.
///
/// # Params:
/// command_handle: command handle to map callback to user context.
/// payment_address: payment address of message signer. The key must be created by calling vcx_wallet_create_address
/// message_raw: a pointer to first byte of message to be signed
/// message_len: a message length
/// cb: Callback that provides error status of function execution and generated signature
///
/// # Returns:
/// a signature string
#[no_mangle]
pub extern fn vcx_wallet_sign_with_address(command_handle: CommandHandle,
                                           payment_address: *const c_char,
                                           message_raw: *const u8,
                                           message_len: u32,
                                           cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32,
                                                                signature: *const u8, signature_len: u32)>) -> u32 {
    info!("vcx_wallet_sign_with_address >>>");
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(payment_address, VcxErrorKind::InvalidOption);
    check_useful_c_byte_array!(message_raw, message_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_sign_with_address(command_handle: {}, payment_address: {}, message_raw: {:?})",
           command_handle, secret!(payment_address), secret!(message_raw));

    spawn(move || {
        match sign_with_address(&payment_address, message_raw.as_slice()) {
            Ok(signature) => {
                trace!("vcx_wallet_sign_with_address_cb(command_handle: {}, rc: {}, signature: {:?})",
                       command_handle, error::SUCCESS.as_str(), signature);

                let (signature_raw, signature_len) = crate::utils::cstring::vec_to_pointer(&signature);

                cb(command_handle, error::SUCCESS.code_num, signature_raw, signature_len);
            }
            Err(error) => {
                warn!("vcx_wallet_sign_with_address_cb(command_handle: {}, error: {})",
                      command_handle, error);

                cb(command_handle, error.into(), null(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Verify a signature with a payment address.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// payment_address: payment address of the message signer
/// message_raw: a pointer to first byte of message that has been signed
/// message_len: a message length
/// signature_raw: a pointer to first byte of signature to be verified
/// signature_len: a signature length
/// cb: Callback that provides error status of function execution and signature verification result
///
/// #Returns
/// valid: true - if signature is valid, false - otherwise
#[no_mangle]
pub extern fn vcx_wallet_verify_with_address(command_handle: CommandHandle,
                                             payment_address: *const c_char,
                                             message_raw: *const u8,
                                             message_len: u32,
                                             signature_raw: *const u8,
                                             signature_len: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32,
                                                                  result: bool)>) -> u32 {
    info!("vcx_wallet_sign_with_address >>>");
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(payment_address, VcxErrorKind::InvalidOption);
    check_useful_c_byte_array!(message_raw, message_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_byte_array!(signature_raw, signature_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_verify_with_address(command_handle: {}, payment_address: {}, message_raw: {:?}, signature_raw: {:?})",
           command_handle, secret!(payment_address), secret!(message_raw), signature_raw);

    spawn(move || {
        match verify_with_address(&payment_address, message_raw.as_slice(), signature_raw.as_slice()) {
            Ok(valid) => {
                trace!("vcx_wallet_verify_with_address_cb(command_handle: {}, rc: {}, valid: {})",
                       command_handle, error::SUCCESS.as_str(), valid);

                cb(command_handle, error::SUCCESS.code_num, valid);
            }
            Err(error) => {
                warn!("vcx_wallet_verify_with_address_cb(command_handle: {}, error: {})",
                      command_handle, error);

                cb(command_handle, error.into(), false);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Adds a record to the wallet
/// Assumes there is an open wallet.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// value: value of the record with the associated id.
///
/// tags_json: the record tags used for search and storing meta information as json:
///   {
///     "tagName1": <str>, // string tag (will be stored encrypted)
///     "tagName2": <int>, // int tag (will be stored encrypted)
///     "~tagName3": <str>, // string tag (will be stored un-encrypted)
///     "~tagName4": <int>, // int tag (will be stored un-encrypted)
///   }
///  The tags_json must be valid json, and if no tags are to be associated with the
/// record, then the empty '{}' json must be passed.
///
/// cb: Callback that provides error status of function execution
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
pub extern fn vcx_wallet_add_record(command_handle: CommandHandle,
                                    type_: *const c_char,
                                    id: *const c_char,
                                    value: *const c_char,
                                    tags_json: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_wallet_add_record >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(value, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tags_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_add_record(command_handle: {}, type_: {}, id: {}, value: {}, tags_json: {})",
           command_handle, secret!(&type_), secret!(&id), secret!(&value), secret!(&tags_json));

    spawn(move || {
        match wallet::add_record(&type_, &id, &value, Some(&tags_json)) {
            Ok(()) => {
                trace!("vcx_wallet_add_record(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                trace!("vcx_wallet_add_record(command_handle: {}, rc: {})",
                       command_handle, x);

                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Updates the value of a record already in the wallet.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// value: New value of the record with the associated id.
///
/// cb: Callback that provides error status of function execution
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
pub extern fn vcx_wallet_update_record_value(command_handle: CommandHandle,
                                             type_: *const c_char,
                                             id: *const c_char,
                                             value: *const c_char,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_wallet_update_record_value >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(value, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_update_record_value(command_handle: {}, type_: {}, id: {}, value: {})",
           command_handle, secret!(&type_), secret!(&id), secret!(&value));

    spawn(move || {
        match wallet::update_record_value(&type_, &id, &value) {
            Ok(()) => {
                trace!("vcx_wallet_update_record_value(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                trace!("vcx_wallet_update_record_value(command_handle: {}, rc: {})",
                       command_handle, x);

                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Updates tags of a record in the wallet.
/// Assumes there is an open wallet and that a record with specified type and id pair already exists.
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// tags: New tags for the record with the associated id and type.
///
/// cb: Callback that provides error status of function execution
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_update_record_tags(command_handle: CommandHandle,
                                            type_: *const c_char,
                                            id: *const c_char,
                                            tags: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_wallet_update_record_tags >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tags, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_update_record_tags(command_handle: {}, type_: {}, id: {}, tags: {})",
           command_handle, secret!(&type_), secret!(&id), secret!(&tags));

    spawn(move || {
        match wallet::update_record_tags(&type_, &id, &tags) {
            Ok(()) => {
                trace!("vcx_wallet_update_record_tags(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                trace!("vcx_wallet_update_record_tags(command_handle: {}, rc: {})",
                       command_handle, x);

                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Add tags to a record in the wallet.
/// Assumes there is an open wallet and that a record with specified type and id pair already exists.
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// tags: Tags for the record with the associated id and type.
///
/// cb: Callback that provides error status of function execution
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_add_record_tags(command_handle: CommandHandle,
                                         type_: *const c_char,
                                         id: *const c_char,
                                         tags: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_wallet_add_record_tags >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tags, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_add_record_tags(command_handle: {}, type_: {}, id: {}, tags: {})",
           command_handle, secret!(&type_), secret!(&id), secret!(&tags));


    spawn(move || {
        match wallet::add_record_tags(&type_, &id, &tags) {
            Ok(()) => {
                trace!("vcx_wallet_add_record_tags(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                trace!("vcx_wallet_add_record_tags(command_handle: {}, rc: {})",
                       command_handle, x);

                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Deletes tags from a record in the wallet.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// tags: Tags names as JSON array to remove from the record with the associated id and type.
///
/// cb: Callback that provides error status of function execution
///
/// #Returns
/// Error code as a u32
///
#[no_mangle]
pub extern fn vcx_wallet_delete_record_tags(command_handle: CommandHandle,
                                            type_: *const c_char,
                                            id: *const c_char,
                                            tags: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_wallet_delete_record_tags >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tags, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_delete_record_tags(command_handle: {}, type_: {}, id: {}, tags: {})",
           command_handle, secret!(&type_), secret!(&id), secret!(&tags));

    spawn(move || {
        match wallet::delete_record_tags(&type_, &id, &tags) {
            Ok(()) => {
                trace!("vcx_wallet_delete_record_tags(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                trace!("vcx_wallet_delete_record_tags(command_handle: {}, rc: {})",
                       command_handle, x);

                cb(command_handle, x.into());
            }
        };

        Ok(())
    });
    error::SUCCESS.code_num
}

/// Deletes an existing record.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// cb: Callback that provides error status of function execution and requested record
///
/// #Returns
/// Error code as a u32
/// Error will be a libindy error code
///
#[no_mangle]
pub extern fn vcx_wallet_get_record(command_handle: CommandHandle,
                                    type_: *const c_char,
                                    id: *const c_char,
                                    options_json: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, record_json: *const c_char)>) -> u32 {
    info!("vcx_wallet_get_record >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(options_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_get_record(command_handle: {}, type_: {}, id: {}, options: {})",
           command_handle, secret!(&type_), secret!(&id), options_json);

    spawn(move || {
        match wallet::get_record(&type_, &id, &options_json) {
            Ok(x) => {
                trace!("vcx_wallet_get_record(command_handle: {}, rc: {}, record_json: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(x));

                let msg = CStringUtils::string_to_cstring(x);

                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                trace!("vcx_wallet_get_record(command_handle: {}, rc: {}, record_json: {})",
                       command_handle, x, "null");

                cb(command_handle, x.into(), EMPTY_CSTR);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Deletes an existing record.
/// Assumes there is an open wallet and that a type and id pair already exists.
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// id: the id ("key") of the record.
///
/// cb: Callback that provides error status of function execution
///
/// #Returns
/// Error code as a u32
/// Error will be a libindy error code
///
#[no_mangle]
pub extern fn vcx_wallet_delete_record(command_handle: CommandHandle,
                                       type_: *const c_char,
                                       id: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_wallet_delete_record >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(id, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_delete_record(command_handle: {}, type_: {}, id: {})",
           command_handle, secret!(&type_), secret!(&id));

    spawn(move || {
        match wallet::delete_record(&type_, &id) {
            Ok(()) => {
                trace!("vcx_wallet_delete_record(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                trace!("vcx_wallet_delete_record(command_handle: {}, rc: {})",
                       command_handle, x);

                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Send tokens to a specific address
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// payment_handle: for future use (currently uses any address in the wallet)
///
/// tokens: number of tokens to send
///
/// recipient: address of recipient
///
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_send_tokens(command_handle: CommandHandle,
                                     payment_handle: u32,
                                     tokens: *const c_char,
                                     recipient: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, receipt: *const c_char)>) -> u32 {
    info!("vcx_wallet_send_tokens >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(recipient, VcxErrorKind::InvalidOption);
    check_useful_c_str!(tokens, VcxErrorKind::InvalidOption);

    let tokens: u64 = match tokens.parse::<u64>() {
        Ok(x) => x,
        Err(e) => return VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot parse tokens: {}", e)).into(),
    };
    trace!("vcx_wallet_send_tokens(command_handle: {}, payment_handle: {}, tokens: {}, recipient: {})",
           command_handle, payment_handle, secret!(tokens), secret!(recipient));

    spawn(move || {
        match pay_a_payee(tokens, &recipient) {
            Ok((_payment, msg)) => {
                trace!("vcx_wallet_send_tokens_cb(command_handle: {}, rc: {}, receipt: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                let msg = "Failed to send tokens";
                trace!("vcx_wallet_send_tokens_cb(command_handle: {}, rc: {}, receipt: {})", command_handle, e, msg);
                cb(command_handle, e.into(), EMPTY_CSTR);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Search for records in the wallet.
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// type_: type of record. (e.g. 'data', 'string', 'foobar', 'image')
///
/// query_json: MongoDB style query to wallet record tags:
///  {
///    "tagName": "tagValue",
///    $or: {
///      "tagName2": { $regex: 'pattern' },
///      "tagName3": { $gte: 123 },
///    },
///  }
/// options_json:
///  {
///    retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
///    retrieveTotalCount: (optional, false by default) Calculate total count,
///    retrieveType: (optional, false by default) Retrieve record type,
///    retrieveValue: (optional, true by default) Retrieve record value,
///    retrieveTags: (optional, true by default) Retrieve record tags,
///  }
/// cb: Callback that provides error status of function execution and handle pointing to the opened search
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub  extern fn vcx_wallet_open_search(command_handle: CommandHandle,
                                      type_: *const c_char,
                                      query_json: *const c_char,
                                      options_json: *const c_char,
                                      cb: Option<extern fn(command_handle_: CommandHandle, err: u32,
                                                           search_handle: SearchHandle)>) -> u32 {
    info!("vcx_wallet_open_search >>>");

    check_useful_c_str!(type_, VcxErrorKind::InvalidOption);
    check_useful_c_str!(query_json, VcxErrorKind::InvalidOption);
    check_useful_c_str!(options_json, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    spawn(move || {
        match wallet::open_search(&type_, &query_json, &options_json) {
            Ok(handle) => {
                trace!("vcx_wallet_open_search(command_handle: {}, rc: {}, handle: {}) ",
                       command_handle, error::SUCCESS.as_str(), handle);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_wallet_open_search(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });
    error::SUCCESS.code_num
}

/// Fetch next records for wallet search.
///
/// Not if there are no records this call returns WalletNoRecords Indy error.
///
/// #Params
/// wallet_search_handle: wallet search handle (returned by vcx_wallet_open_search)
///
/// count: Count of records to fetch
///
///
/// cb: Callback that provides error status of function execution and wallet records json:
/// {
///   totalCount: <int>, // present only if retrieveTotalCount set to true
///   records: [{ // present only if retrieveRecords set to true
///       id: "Some id",
///       type: "Some type", // present only if retrieveType set to true
///       value: "Some value", // present only if retrieveValue set to true
///       tags: <tags json>, // present only if retrieveTags set to true
///   }],
/// }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub  extern fn vcx_wallet_search_next_records(command_handle: CommandHandle,
                                              wallet_search_handle: i32,
                                              count: usize,
                                              cb: Option<extern fn(command_handle_: CommandHandle, err: u32,
                                                                   records_json: *const c_char)>) -> u32 {
    info!("vcx_wallet_search_next_records >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_search_next_records(command_handle: {}, wallet_search_handle: {}, count: {})",
           command_handle, wallet_search_handle, count);

    spawn(move || {
        match wallet::search_next_records(wallet_search_handle, count) {
            Ok(records) => {
                trace!("vcx_wallet_search_next_records(command_handle: {}, wallet_search_handle: {}, rc: {}, records: {})",
                       command_handle, wallet_search_handle, error::SUCCESS.as_str(), secret!(records));
                let records = CStringUtils::string_to_cstring(records);
                cb(command_handle, error::SUCCESS.code_num, records.as_ptr());
            }
            Err(x) => {
                warn!("vcx_wallet_search_next_records(command_handle: {}, wallet_search_handle: {}, rc: {})",
                      command_handle, wallet_search_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Close a search
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// search_handle: wallet search handle
///
/// cb: Callback that provides error status of function execution
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_wallet_close_search(command_handle: CommandHandle,
                                      search_handle: i32,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_wallet_close_search >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!("vcx_wallet_close_search(command_handle: {}, search_handle: {})",
           command_handle, search_handle);

    spawn(move || {
        match wallet::close_search(search_handle) {
            Ok(()) => {
                trace!("vcx_wallet_close_search(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                trace!("vcx_wallet_close_search(command_handle: {}, rc: {})",
                       command_handle, x);

                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Exports opened wallet
///
/// Note this endpoint is EXPERIMENTAL. Function signature and behaviour may change
/// in the future releases.
///
/// #Params:
/// command_handle: Handle for User's Reference only.
/// path: Path to export wallet to User's File System.
/// backup_key: String representing the User's Key for securing (encrypting) the exported Wallet.
/// cb: Callback that provides error status of function execution
///
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
#[no_mangle]
pub extern fn vcx_wallet_export(command_handle: CommandHandle,
                                path: *const c_char,
                                backup_key: *const c_char,
                                cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                     err: u32)>) -> u32 {
    info!("vcx_wallet_export >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(path,  VcxErrorKind::InvalidOption);
    check_useful_c_str!(backup_key, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_export(command_handle: {}, path: {}, backup_key: {})",
           command_handle, path, secret!(backup_key));


    spawn(move || {
        match export(get_wallet_handle(), &path, &backup_key) {
            Ok(()) => {
                let return_code = error::SUCCESS.code_num;
                trace!("vcx_wallet_export(command_handle: {}, rc: {})", command_handle, return_code);
                cb(command_handle, return_code);
            }
            Err(e) => {
                warn!("vcx_wallet_export(command_handle: {}, rc: {})", command_handle, e);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Creates a new secure wallet and then imports its content
/// according to fields provided in import_config
/// Cannot be used if wallet is already opened (Especially if vcx_init has already been used).
///
/// Note this endpoint is EXPERIMENTAL. Function signature and behaviour may change
/// in the future releases.
///
/// config: "{"wallet_name":"","wallet_key":"","exported_wallet_path":"","backup_key":"","key_derivation":""}"
/// exported_wallet_path: Path of the file that contains exported wallet content
/// backup_key: Key used when creating the backup of the wallet (For encryption/decrption)
/// Optional<key_derivation>: method of key derivation used by libindy. By default, libvcx uses ARGON2I_INT
/// cb: Callback that provides the success/failure of the api call.
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
#[no_mangle]
pub extern fn vcx_wallet_import(command_handle: CommandHandle,
                                config: *const c_char,
                                cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                     err: u32)>) -> u32 {
    info!("vcx_wallet_import >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(config,  VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_import(command_handle: {}, config: {})",
           command_handle, secret!(config));

    thread::spawn(move || {
        match import(&config) {
            Ok(()) => {
                trace!("vcx_wallet_import(command_handle: {}, rc: {})", command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_wallet_import(command_handle: {}, rc: {})", command_handle, e);
                cb(command_handle, e.into());
            }
        };
    });

    error::SUCCESS.code_num
}

// Functionality in Libindy for validating an address in NOT there yet
/// Validates a Payment address
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// payment_address: value to be validated as a payment address
///
/// cb: Callback that any errors
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub  extern fn vcx_wallet_validate_payment_address(command_handle: i32,
                                                   payment_address: *const c_char,
                                                   cb: Option<extern fn(command_handle_: i32, err: u32)>) -> u32 {
    info!("vcx_wallet_validate_payment_address >>>");

    check_useful_c_str!(payment_address,  VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_wallet_validate_payment_address(command_handle: {}, payment_address: {})",
           command_handle, secret!(payment_address));

    spawn(move || {
        cb(command_handle, error::SUCCESS.code_num);
        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::api::return_types;
    use std::ffi::CString;
    use crate::utils::libindy::wallet::{delete_wallet, init_wallet};
    use crate::utils::devsetup::*;
    use crate::settings;


    const XTYPE: *const c_char = "record_type\0".as_ptr().cast();
    const ID: *const c_char = "123\0".as_ptr().cast();
    const VALUE: *const c_char = "Record Value\0".as_ptr().cast();
    const EMPTY_TAGS: *const c_char = "{}\0".as_ptr().cast();
    const OPTIONS: *const c_char = concat!(r#"{
            "retrieveType": true,
            "retrieveValue": true,
            "retrieveTags": false
        }"#, "\0").as_ptr().cast();

        #[test]
    fn test_add_record() {
        let _setup = SetupLibraryWallet::init();

        // Valid add
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_add_record(h,
                                         XTYPE,
                                         ID,
                                         VALUE,
                                         EMPTY_TAGS,
                                         Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();

        // Failure because of duplicate
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_add_record(h,
                                         XTYPE,
                                         ID,
                                         VALUE,
                                         EMPTY_TAGS,
                                         Some(cb)),
                   error::SUCCESS.code_num);

        assert_eq!(r.recv_medium().err(), Some(error::DUPLICATE_WALLET_RECORD.code_num));
    }

    #[test]
    fn test_add_record_with_tag() {
        let _setup = SetupLibraryWallet::init();

        let tags = concat!(r#"{"tagName1":"tag1","tagName2":"tag2"}"#, "\0").as_ptr().cast();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_add_record(h,
                                         XTYPE,
                                         ID,
                                         VALUE,
                                         tags,
                                         Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_get_record_fails_with_no_value() {
        let _setup = SetupLibraryWallet::init();

        let options = json!({
            "retrieveType": true,
            "retrieveValue": true,
            "retrieveTags": false
        }).to_string();
        let options = CStringUtils::string_to_cstring(options);

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_wallet_get_record(h,
                                         XTYPE,
                                         ID,
                                         options.as_ptr(),
                                         Some(cb)),
                   error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().err(), Some(error::WALLET_RECORD_NOT_FOUND.code_num));
    }

    #[test]
    fn test_get_record_value_success() {
        let _setup = SetupLibraryWallet::init();

        // Valid add
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_add_record(h,
                                         XTYPE,
                                         ID,
                                         VALUE,
                                         EMPTY_TAGS,
                                         Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_wallet_get_record(h,
                                         XTYPE,
                                         ID,
                                         OPTIONS,
                                         Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_delete_record() {
        let _setup = SetupLibraryWallet::init();

        // Add record
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_add_record(h, 
                                         XTYPE,
                                         ID,
                                         VALUE,
                                         EMPTY_TAGS,
                                         Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();

        // Successful deletion
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_delete_record(h,
                                            XTYPE,
                                            ID,
                                            Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();

        // Fails with no record
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_delete_record(h,
                                            XTYPE,
                                            ID,
                                            Some(cb)),
                   error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().err(),
                   Some(error::WALLET_RECORD_NOT_FOUND.code_num));
    }

    #[test]
    fn test_update_record_value() {
        let _setup = SetupLibraryWallet::init();

        // Assert no record to update
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_update_record_value(h,
                                                  XTYPE,
                                                  ID,
                                                  OPTIONS,
                                                  Some(cb)),
                   error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().err(),
                   Some(error::WALLET_RECORD_NOT_FOUND.code_num));

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_add_record(h, 
                                         XTYPE,
                                         ID,
                                         VALUE,
                                         EMPTY_TAGS,
                                         Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();

        // Assert update works
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_wallet_update_record_value(h,
                                                  XTYPE,
                                                  ID,
                                                  OPTIONS,
                                                  Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_wallet_import_export() {
        let _setup = SetupDefaults::init();

        let wallet_name = "test_wallet_import_export";

        let export_file = TempFile::prepare_path(wallet_name);

        init_wallet(wallet_name, None, None, None).unwrap();

        let backup_key = settings::get_config_value(settings::CONFIG_WALLET_BACKUP_KEY).unwrap();
        let wallet_key = settings::get_config_value(settings::CONFIG_WALLET_KEY).unwrap();

        let (h, cb, r) = return_types::return_u32();
        let export_path_cstr = CString::new(export_file.path.as_bytes()).unwrap();
        let backup_key_cstr = CString::new(backup_key.as_bytes()).unwrap();
        assert_eq!(vcx_wallet_export(h,
                                     export_path_cstr.as_ptr(),
                                     backup_key_cstr.as_ptr(),
                                     Some(cb)), error::SUCCESS.code_num);
        r.recv_long().unwrap();

        delete_wallet(&wallet_name, None, None, None).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: wallet_name,
            settings::CONFIG_WALLET_KEY: wallet_key,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_file.path,
            settings::CONFIG_WALLET_BACKUP_KEY: backup_key,
        }).to_string();

        let (h, cb, r) = return_types::return_u32();
        let config_cstr = CString::new(import_config).unwrap();
        assert_eq!(vcx_wallet_import(h,
                                     config_cstr.as_ptr(),
                                     Some(cb)), error::SUCCESS.code_num);
        r.recv_long().unwrap();

        delete_wallet(&wallet_name, None, None, None).unwrap();
    }
}
