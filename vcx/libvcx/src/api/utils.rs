use serde_json;
use libc::c_char;
use crate::messages;
use std::ptr;
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use crate::utils::threadpool::spawn;
use crate::utils::libindy::{payments, wallet};
use std::thread;
use crate::error::prelude::*;
use indy_sys::CommandHandle;
use crate::utils::httpclient::AgencyMock;
use crate::utils::constants::*;
use crate::messages::agent_provisioning::types::ProvisioningConfig;
use crate::v3::handlers::connection::agent::AgentInfo;
use crate::messages::{agent_provisioning, update_agent};
use crate::messages::update_agent::ComMethod;
use crate::v3::messages::attachment::extract_attached_message;

/// Provision an agent in the agency, populate configuration and wallet for this agent.
///
/// #Params
/// config: Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
/// token: {
///          This can be a push notification endpoint to contact the sponsee or
///          an id that the sponsor uses to reference the sponsee in its backend system
///          "sponseeId": String,
///          "sponsorId": String, //Persistent Id of the Enterprise sponsoring the provisioning
///          "nonce": String,
///          "timestamp": String,
///          "sig": String, // Base64Encoded(sig(nonce + timestamp + id))
///          "sponsorVerKey": String,
///          "attestationAlgorithm": Optional<String>, // device attestation signature algorithm. Can be one of: SafetyNet | DeviceCheck
///          "attestationData": Optional<String>, // device attestation signature matching to specified algorithm
///        }
///
/// #Returns
/// Configuration (wallet also populated), on error returns NULL
#[no_mangle]
pub extern fn vcx_provision_agent_with_token(config: *const c_char, token: *const c_char) -> *mut c_char {
    info!("vcx_provision_agent >>>");

    let config = match CStringUtils::c_str_to_string(config) {
        Ok(Some(val)) => val,
        _ => {
            let _res: u32 = VcxError::from_msg(VcxErrorKind::InvalidOption, "Invalid pointer has been passed").into();
            return ptr::null_mut();
        }
    };

    let token = match CStringUtils::c_str_to_string(token) {
        Ok(Some(val)) => val,
        _ => {
            let _res: u32 = VcxError::from_msg(VcxErrorKind::InvalidOption, "Invalid pointer has been passed").into();
            return ptr::null_mut();
        }
    };

    trace!("vcx_provision_agent_with_token(config: {}, token: {})", secret!(config), secret!(token));

    match messages::agent_provisioning::agent_provisioning_v0_7::provision(&config, &token) {
        Err(e) => {
            error!("Provision Agent Error {}.", e);
            wallet::close_wallet().ok();
            let _res: u32 = e.into();
            ptr::null_mut()
        }
        Ok(s) => {
            debug!("Provision Agent Successful");
            let msg = CStringUtils::string_to_cstring(s);

            // TODO: this is a memory leak
            msg.into_raw()
        }
    }
}

/// Provision an agent in the agency, populate configuration and wallet for this agent.
///
/// #Params
/// config: Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
/// token: {
///          This can be a push notification endpoint to contact the sponsee or
///          an id that the sponsor uses to reference the sponsee in its backend system
///          "sponseeId": String,
///          "sponsorId": String, //Persistent Id of the Enterprise sponsoring the provisioning
///          "nonce": String,
///          "timestamp": String,
///          "sig": String, // Base64Encoded(sig(nonce + timestamp + id))
///          "sponsorVerKey": String,
///          "attestationAlgorithm": Optional<String>, // device attestation signature algorithm. Can be one of: SafetyNet | DeviceCheck
///          "attestationData": Optional<String>, // device attestation signature matching to specified algorithm
///        }
///
/// cb: Callback that provides configuration as JSON string or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_provision_agent_with_token_async(command_handle: CommandHandle,
                                                   config: *const c_char,
                                                   token: *const c_char,
                                                   cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, _config: *const c_char)>) -> u32 {
    info!("vcx_provision_agent_with_token_async >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(config, VcxErrorKind::InvalidOption);
    check_useful_c_str!(token, VcxErrorKind::InvalidOption);

    trace!("vcx_provision_agent_with_token_async(config: {}, token: {})", secret!(config), secret!(token));

    thread::spawn(move || {
        match messages::agent_provisioning::agent_provisioning_v0_7::provision(&config, &token) {
            Err(e) => {
                error!("vcx_provision_agent_with_token_async_cb(command_handle: {}, rc: {}, config: NULL", command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
            }
            Ok(config) => {
                trace!("vcx_provision_agent_with_token_async_cb(command_handle: {}, rc: {}, config: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(config));
                let cconfig = CStringUtils::string_to_cstring(config);
                cb(command_handle, 0, cconfig.as_ptr());
            }
        }
    });

    error::SUCCESS.code_num
}

/// Provision an agent in the agency, populate configuration and wallet for this agent.
/// NOTE: for asynchronous call use vcx_agent_provision_async
///
/// #Params
/// config: Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
///
/// #Returns
/// Configuration (wallet also populated), on error returns NULL
#[no_mangle]
pub extern fn vcx_provision_agent(config: *const c_char) -> *mut c_char {
    info!("vcx_provision_agent >>>");

    let config = match CStringUtils::c_str_to_string(config) {
        Ok(Some(val)) => val,
        _ => {
            let _res: u32 = VcxError::from_msg(VcxErrorKind::InvalidOption, "Invalid pointer has been passed").into();
            return ptr::null_mut();
        }
    };

    trace!("vcx_provision_agent(config: {})", secret!(config));

    match agent_provisioning::provision(&config) {
        Err(e) => {
            error!("Provision Agent Error {}.", e);
            wallet::close_wallet().ok();
            let _res: u32 = e.into();
            ptr::null_mut()
        }
        Ok(s) => {
            debug!("Provision Agent Successful");
            let msg = CStringUtils::string_to_cstring(s);

            // TODO: this is a memory leak
            msg.into_raw()
        }
    }
}

/// Provision an agent in the agency, populate configuration and wallet for this agent.
/// NOTE: for synchronous call use vcx_provision_agent
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// config: Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
///
/// cb: Callback that provides configuration or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_agent_provision_async(command_handle: CommandHandle,
                                        config: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, _config: *const c_char)>) -> u32 {
    info!("vcx_agent_provision_async >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(config, VcxErrorKind::InvalidOption);

    trace!("vcx_agent_provision_async(command_handle: {}, json: {})",
           command_handle, secret!(config));

    thread::spawn(move || {
        match agent_provisioning::provision(&config) {
            Err(e) => {
                error!("vcx_agent_provision_async_cb(command_handle: {}, rc: {}, config: NULL", command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
            }
            Ok(s) => {
                trace!("vcx_agent_provision_async_cb(command_handle: {}, rc: {}, config: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(s));
                let msg = CStringUtils::string_to_cstring(s);
                cb(command_handle, 0, msg.as_ptr());
            }
        }
    });

    error::SUCCESS.code_num
}

/// Get token which can be used for provisioning an agent
/// NOTE: Can be used only for Evernym's applications
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// config:
/// {
///     vcx_config: VcxConfig // Same config passed to agent provision
///                           // See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
///     sponsee_id: String,
///     sponsor_id: String,
/// }
///
/// cb: Callback that provides provision token or error status
///
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_get_provision_token(command_handle: CommandHandle,
                                      config: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, token: *const c_char)>) -> u32 {
    info!("vcx_get_provision_token >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(config, VcxErrorKind::InvalidOption);

    trace!("vcx_get_provision_token(command_handle: {}, config: {})",
           command_handle, secret!(config));

    let configs: serde_json::Value = match serde_json::from_str(&config) {
        Ok(x) => x,
        Err(e) => {
            return VcxError::from_msg(VcxErrorKind::InvalidConfiguration, format!("Cannot parse Config from JSON string. Err: {}", e)).into();
        }
    };

    let vcx_config: ProvisioningConfig = match serde_json::from_value(configs["vcx_config"].clone()) {
        Ok(x) => x,
        Err(_) => {
            return VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "missing vcx_config").into();
        }
    };

    let sponsee_id: String = match serde_json::from_value(configs["sponsee_id"].clone()) {
        Ok(x) => x,
        Err(_) => {
            return VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "missing sponsee_id").into();
        }
    };

    let sponsor_id: String = match serde_json::from_value(configs["sponsor_id"].clone()) {
        Ok(x) => x,
        Err(_) => {
            return VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "missing sponsor_id").into();
        }
    };

    spawn(move || {
        match messages::token_provisioning::token_provisioning::provision(vcx_config, &sponsee_id, &sponsor_id) {
            Ok(token) => {
                trace!("vcx_get_provision_token_cb(command_handle: {}, rc: {}, token: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(token));
                let token_ = CStringUtils::string_to_cstring(token);
                cb(command_handle, 0, token_.as_ptr());
            }
            Err(e) => {
                error!("vcx_get_provision_token_cb(command_handle: {}, rc: {}, token: NULL", command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Update information on the agent (ie, comm method and type)
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// json: updated configuration
///     {
///         "id": "string", 1 means push notifications, 4 means forward
///         "type": Optional(int), notifications type (1 is used by default).
///         "value": "string",
///     }
///
/// cb: Callback that provides configuration or error status
///
/// # Example json -> "{"id":"123","value":"value"}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_agent_update_info(command_handle: CommandHandle,
                                    json: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_agent_update_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(json, VcxErrorKind::InvalidOption);

    trace!("vcx_agent_update_info(command_handle: {}, json: {})",
           command_handle, secret!(json));

    let com_method: ComMethod = match serde_json::from_str(&json) {
        Ok(x) => x,
        Err(e) => {
            return VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse AgentInfo from JSON string. Err: {}", e)).into();
        }
    };

    spawn(move || {
        match update_agent::update_agent_info(com_method) {
            Ok(()) => {
                trace!("vcx_agent_update_info_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                error!("vcx_agent_update_info_cb(command_handle: {}, rc: {})",
                       command_handle, e);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get ledger fees from the network
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides the fee structure for the sovrin network
///
/// # Example fees -> "{ "txnType1": amount1, "txnType2": amount2, ..., "txnTypeN": amountN }"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_ledger_get_fees(command_handle: CommandHandle,
                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, fees: *const c_char)>) -> u32 {
    info!("vcx_ledger_get_fees >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!("vcx_ledger_get_fees(command_handle: {})",
           command_handle);

    spawn(move || {
        match crate::utils::libindy::payments::get_ledger_fees() {
            Ok(x) => {
                trace!("vcx_ledger_get_fees_cb(command_handle: {}, rc: {}, fees: {})",
                       command_handle, error::SUCCESS.as_str(), x);

                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                warn!("vcx_ledget_get_fees_cb(command_handle: {}, rc: {}, fees: {})",
                      command_handle, e, "null");

                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_set_next_agency_response(message_index: u32) {
    info!("vcx_set_next_agency_response >>>");

    let message = match message_index {
        1 => &CREATE_KEYS_RESPONSE[..],
        2 => &UPDATE_PROFILE_RESPONSE[..],
        3 => &GET_MESSAGES_RESPONSE[..],
        4 => &UPDATE_CREDENTIAL_RESPONSE[..],
        5 => &UPDATE_PROOF_RESPONSE[..],
        6 => &CREDENTIAL_REQ_RESPONSE[..],
        7 => &PROOF_RESPONSE[..],
        8 => &CREDENTIAL_RESPONSE[..],
        9 => &GET_MESSAGES_INVITE_ACCEPTED_RESPONSE[..],
        _ => &[],
    };

    AgencyMock::set_next_response(message);
}

/// Retrieve messages from the Cloud Agent
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// message_status: optional - query for messages with the specified status
///
/// uids: optional, comma separated - query for messages with the specified uids
///
/// cb: Callback that provides array of matching messages retrieved
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_download_agent_messages(command_handle: u32,
                                          message_status: *const c_char,
                                          uids: *const c_char,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32, messages: *const c_char)>) -> u32 {
    info!("vcx_download_agent_messages >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let message_status = if !message_status.is_null() {
        check_useful_c_str!(message_status, VcxErrorKind::InvalidOption);
        let v = message_status.split(',').map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v)
    } else {
        None
    };

    let uids = if !uids.is_null() {
        check_useful_c_str!(uids, VcxErrorKind::InvalidOption);
        let v = uids.split(',').map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v)
    } else {
        None
    };

    trace!("vcx_download_agent_messages(command_handle: {}, message_status: {:?}, uids: {:?})",
           command_handle, message_status, uids);

    spawn(move || {
        match crate::messages::get_message::download_agent_messages(message_status, uids) {
            Ok(x) => {
                match serde_json::to_string(&x) {
                    Ok(x) => {
                        trace!("vcx_download_agent_messages(command_handle: {}, rc: {}, messages: {})",
                               command_handle, error::SUCCESS.as_str(), secret!(x));

                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    Err(e) => {
                        let err = VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize downloaded messages as JSON. Error: {}", e));
                        warn!("vcx_download_agent_messages(command_handle: {}, rc: {}, messages: {})",
                              command_handle, err, "null");

                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                };
            }
            Err(e) => {
                warn!("vcx_download_agent_messages(command_handle: {}, rc: {}, messages: {})",
                      command_handle, e, "null");

                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieve messages from the agent
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// message_status: optional, comma separated -  - query for messages with the specified status.
///                            Statuses:
///                                 MS-101 - Created
///                                 MS-102 - Sent
///                                 MS-103 - Received
///                                 MS-104 - Accepted
///                                 MS-105 - Rejected
///                                 MS-106 - Reviewed
///
/// uids: optional, comma separated - query for messages with the specified uids
///
/// pw_dids: optional, comma separated - DID's pointing to specific connection
///
/// cb: Callback that provides array of matching messages retrieved
///
/// # Example message_status -> MS-103, MS-106
///
/// # Example uids -> s82g63, a2h587
///
/// # Example pw_dids -> did1, did2
///
/// # Example messages -> "[{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}]"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_messages_download(command_handle: CommandHandle,
                                    message_status: *const c_char,
                                    uids: *const c_char,
                                    pw_dids: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, messages: *const c_char)>) -> u32 {
    info!("vcx_messages_download >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let message_status = if !message_status.is_null() {
        check_useful_c_str!(message_status, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = message_status.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    let uids = if !uids.is_null() {
        check_useful_c_str!(uids, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = uids.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    let pw_dids = if !pw_dids.is_null() {
        check_useful_c_str!(pw_dids, VcxErrorKind::InvalidOption);
        let v: Vec<&str> = pw_dids.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v.to_owned())
    } else {
        None
    };

    trace!("vcx_messages_download(command_handle: {}, message_status: {:?}, uids: {:?}, pw_dids: {:?})",
           command_handle, message_status, uids, secret!(pw_dids));

    spawn(move || {
        match crate::messages::get_message::download_messages(pw_dids, message_status, uids) {
            Ok(x) => {
                match serde_json::to_string(&x) {
                    Ok(x) => {
                        trace!("vcx_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                               command_handle, error::SUCCESS.as_str(), secret!(x));

                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    Err(e) => {
                        let err = VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize downloaded messages as JSON. Error: {}", e));
                        warn!("vcx_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                              command_handle, err, "null");

                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                };
            }
            Err(e) => {
                warn!("vcx_messages_download_cb(command_handle: {}, rc: {}, messages: {})",
                      command_handle, e, "null");

                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves single message from the agency by the given uid.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// uid: id of the message to query.
///
/// cb: Callback that provides retrieved message
///
/// # Example message -> "{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_download_message(command_handle: CommandHandle,
                                   uid: *const c_char,
                                   cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                        err: u32,
                                                        message: *const c_char)>) -> u32 {
    info!("vcx_download_message >>>");

    check_useful_c_str!(uid, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_download_message(command_handle: {}, uid: {:?})",
           command_handle, uid);

    spawn(move || {
        match crate::messages::get_message::download_message(uid) {
            Ok(message) => {
                trace!("vcx_download_message_cb(command_handle: {}, rc: {}, message: {:?})",
                       command_handle, error::SUCCESS.as_str(), secret!(message));

                let message_json = json!(message).to_string();
                let msg = CStringUtils::string_to_cstring(message_json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                warn!("vcx_download_message_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Update the status of messages from the specified connection
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// message_status: target message status
///                 Statuses:
///                     MS-101 - Created
///                     MS-102 - Sent
///                     MS-103 - Received
///                     MS-104 - Accepted
///                     MS-105 - Rejected
///                     MS-106 - Reviewed
///
/// msg_json: messages to update: [{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]},...]
///
/// cb: Callback that provides success or failure of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_messages_update_status(command_handle: CommandHandle,
                                         message_status: *const c_char,
                                         msg_json: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_messages_update_status >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message_status, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_json, VcxErrorKind::InvalidOption);

    trace!("vcx_messages_set_status(command_handle: {}, message_status: {:?}, uids: {:?})",
           command_handle, message_status, secret!(msg_json));

    spawn(move || {
        match crate::messages::update_message::update_agency_messages(&message_status, &msg_json) {
            Ok(()) => {
                trace!("vcx_messages_set_status_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_messages_set_status_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Gets minimal request price for performing an action in case the requester can perform this action.
///
/// # Params
/// action_json: {
///     "auth_type": ledger transaction alias or associated value,
///     "auth_action": type of an action.,
///     "field": transaction field,
///     "old_value": (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action),
///     "new_value": (Optional) new value that can be used to fill the field,
/// }
/// requester_info_json: (Optional) {
///     "role": string - role of a user which can sign transaction.
///     "count": string - count of users.
///     "is_owner": bool - if user is an owner of transaction.
/// } otherwise context info will be used
///
/// # Return
/// "price": u64 - tokens amount required for action performing
#[no_mangle]
pub extern fn vcx_get_request_price(command_handle: CommandHandle,
                                    action_json: *const c_char,
                                    requester_info_json: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, price: u64)>) -> u32 {
    info!("vcx_get_request_price >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(action_json, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(requester_info_json, VcxErrorKind::InvalidOption);

    trace!(target: "vcx", "vcx_get_request_price(command_handle: {}, action_json: {}, requester_info_json: {:?})",
           command_handle, action_json, requester_info_json);

    spawn(move || {
        match payments::get_request_price(action_json, requester_info_json) {
            Ok(x) => {
                trace!(target: "vcx", "vcx_get_request_price(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);
                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(x) => {
                warn!("vcx_get_request_price(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Endorse transaction to the ledger preserving an original author
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// transaction: transaction to endorse
///
/// cb: Callback that provides success or failure of command
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_endorse_transaction(command_handle: CommandHandle,
                                      transaction: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_endorse_transaction >>>");

    check_useful_c_str!(transaction, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!("vcx_endorse_transaction(command_handle: {}, transaction: {})",
           command_handle, secret!(transaction));

    spawn(move || {
        match crate::utils::libindy::ledger::endorse_transaction(&transaction) {
            Ok(()) => {
                trace!("vcx_endorse_transaction(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_endorse_transaction(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Fetch and Cache public entities from the Ledger associated with stored in the wallet credentials.
/// This function performs two steps:
///     1) Retrieves the list of all credentials stored in the opened wallet.
///     2) Fetch and cache Schemas / Credential Definitions / Revocation Registry Definitions
///        correspondent to received credentials from the connected Ledger.
///
/// This helper function can be used, for instance as a background task, to refresh library cache.
/// This allows us to reduce the time taken for Proof generation by using already cached entities instead of queering the Ledger.
///
/// NOTE: Library must be already initialized (wallet and pool must be opened).
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides result code
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_fetch_public_entities(command_handle: CommandHandle,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                             err: u32)>) -> u32 {
    info!("vcx_fetch_public_entities >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    trace!("vcx_fetch_public_entities(command_handle: {})", command_handle);

    spawn(move || {
        match crate::utils::libindy::anoncreds::fetch_public_entities() {
            Ok(()) => {
                trace!("vcx_fetch_public_entities_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_fetch_public_entities_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// This function allows you to check the health of LibVCX and EAS/CAS instance.
/// It will return error in case of any problems on EAS or will resolve pretty long if VCX is thread-hungry.
/// WARNING: this call may take a lot of time returning answer in case of load, be careful.
/// NOTE: Library must be initialized, ENDPOINT_URL should be set
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// cb: Callback that provides result code
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_health_check(command_handle: CommandHandle,
                               cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                    err: u32)>) -> u32 {
    info!("vcx_health_check >>>");
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    spawn(move || {
        match crate::utils::health_check::health_check() {
            Ok(()) => {
                trace!("vcx_health_check_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_health_check_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };
        Ok(())
    });
    error::SUCCESS.code_num
}

/// Create pairwise agent which can be later used for connection establishing.
///
/// You can pass `agent_info` into `vcx_connection_connect` function as field of `connection_options` JSON parameter.
/// The passed Pairwise Agent will be used for connection establishing instead of creation a new one.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides agent info as JSON string:
///     {
///         "pw_did": string,
///         "pw_vk": string,
///         "agent_did": string,
///         "agent_vk": string,
///     }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_create_pairwise_agent(command_handle: CommandHandle,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                             err: u32,
                                                             agent_info: *const c_char)>) -> u32 {
    info!("vcx_create_pairwise_agent >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_create_pairwise_agent(command_handle: {})", command_handle);

    spawn(move || {
        match AgentInfo::create_agent() {
            Ok(agent_info) => {
                trace!("vcx_create_pairwise_agent_cb(command_handle: {}, rc: {}, message: {:?})",
                       command_handle, error::SUCCESS.as_str(), secret!(agent_info));

                let agent_info_json = json!(agent_info).to_string();
                let result = CStringUtils::string_to_cstring(agent_info_json);
                cb(command_handle, error::SUCCESS.code_num, result.as_ptr());
            }
            Err(e) => {
                warn!("vcx_create_pairwise_agent_cb(command_handle: {}, rc: {})",
                      command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Extract content of Aries message containing attachment decorator.
/// RFC: https://github.com/hyperledger/aries-rfcs/tree/main/features/0592-indy-attachments
///
/// #params
///
/// message: aries message containing attachment decorator
/// command_handle: command handle to map callback to user context.
///
///
/// cb: Callback that provides attached message
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_extract_attached_message(command_handle: CommandHandle,
                                           message: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                err: u32,
                                                                attachment_content: *const c_char)>) -> u32 {
    info!("vcx_extract_attached_message >>>");

    check_useful_c_str!(message, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_extract_attached_message(command_handle: {}, message: {:?})",
           command_handle, message);

    spawn(move || {
        match extract_attached_message(&message) {
            Ok(attachment_content) => {
                trace!("vcx_extract_attached_message_cb(command_handle: {}, rc: {}, message: {:?})",
                       command_handle, error::SUCCESS.as_str(), secret!(attachment_content));

                let attachment_content = CStringUtils::string_to_cstring(attachment_content);
                cb(command_handle, error::SUCCESS.code_num, attachment_content.as_ptr());
            }
            Err(e) => {
                warn!("vcx_extract_attached_message_cb(command_handle: {}, rc: {})",
                      command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
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
    use crate::api::return_types;
    use crate::utils::devsetup::*;
    use crate::utils::httpclient::AgencyMock;
    use crate::utils::constants::REGISTER_RESPONSE;

    static CONFIG: &'static str = r#"{"agency_url":"https://enym-eagency.pdev.evernym.com","agency_did":"Ab8TvZa3Q19VNkQVzAWVL7","agency_verkey":"5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf","wallet_name":"test_provision_agent","agent_seed":null,"enterprise_seed":null,"wallet_key":"key"}"#;
    const CONFIG_CSTR: *const c_char = concat!(r#"{"agency_url":"https://enym-eagency.pdev.evernym.com","agency_did":"Ab8TvZa3Q19VNkQVzAWVL7","agency_verkey":"5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf","wallet_name":"test_provision_agent","agent_seed":null,"enterprise_seed":null,"wallet_key":"key"}"#, "\0").as_ptr().cast();

    fn _vcx_agent_provision_async_c_closure(config: &str) -> Result<Option<String>, u32> {
        let (h, cb, r) = return_types::return_u32_str();
        let config = CString::new(config).unwrap();
        let rc = vcx_agent_provision_async(h,
                                           config.as_ptr(),
                                           Some(cb));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        r.recv_short()
    }

    #[test]
    fn test_provision_agent() {
        let _setup = SetupMocks::init();

        let result = vcx_provision_agent(CONFIG_CSTR);

        let result = CStringUtils::c_str_to_string(result).unwrap().unwrap();
        let _config: serde_json::Value = serde_json::from_str(&result).unwrap();
    }

    #[test]
    fn test_get_token_input_fails() {
        let _setup = SetupMocks::init();
        let vcx_config = serde_json::from_str::<serde_json::Value>(&CONFIG).unwrap();
        let config = json!({
            "vcx_config": vcx_config,
            "source_id": "123",
            "com_method": {"id":"123","value":"FCM:Value"}
        });

        let c_json = CString::new(config.to_string()).unwrap();

        let (h, cb, _r) = return_types::return_u32_str();
        let rc = vcx_get_provision_token(h, c_json.as_ptr(), Some(cb));
        assert_eq!(rc, error::INVALID_CONFIGURATION.code_num)
    }

    #[test]
    #[ignore] // TODO: restore it
    fn test_get_token_success() {
        let _setup = SetupMocks::init();
        let vcx_config = serde_json::from_str::<serde_json::Value>(&CONFIG).unwrap();
        let config = json!({
            "vcx_config": vcx_config,
            "sponsee_id": "123",
            "sponsor_id": "123",
            "com_method": {"type": 1, "id":"123","value":"FCM:Value"}
        });

        let c_json = CString::new(config.to_string()).unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        let rc = vcx_get_provision_token(h, c_json.as_ptr(), Some(cb));
        assert_eq!(rc, error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_create_agent() {
        let _setup = SetupMocks::init();

        let result = _vcx_agent_provision_async_c_closure(CONFIG).unwrap();
        let _config: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    }

    #[test]
    fn test_create_agent_fails() {
        let _setup = SetupMocks::init();

        let config = r#"{"agency_url":"https://enym-eagency.pdev.evernym.com","agency_did":"Ab8TvZa3Q19VNkQVzAWVL7","agency_verkey":"5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf","wallet_name":"test_provision_agent","agent_seed":null,"enterprise_seed":null,"wallet_key":null}"#;

        let err = _vcx_agent_provision_async_c_closure(config).unwrap_err();
        assert_eq!(err, error::INVALID_CONFIGURATION.code_num);
    }

    #[test]
    fn test_create_agent_fails_for_unknown_wallet_type() {
        let _setup = SetupDefaults::init();

        let config = json!({
            "agency_url":"https://enym-eagency.pdev.evernym.com",
            "agency_did":"Ab8TvZa3Q19VNkQVzAWVL7",
            "agency_verkey":"5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf",
            "wallet_name":"test_provision_agent",
            "wallet_key":"key",
            "wallet_type":"UNKNOWN_WALLET_TYPE"
        }).to_string();

        let err = _vcx_agent_provision_async_c_closure(&config).unwrap_err();
        assert_eq!(err, error::INVALID_WALLET_CREATION.code_num);
    }

    #[test]
    fn test_update_agent_info() {
        let _setup = SetupMocks::init();

        let c_json = concat!(r#"{"id":"123","value":"value"}"#, "\0").as_ptr().cast();

        let (h, cb, r) = return_types::return_u32();
        let _result = vcx_agent_update_info(h, c_json, Some(cb));
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_update_agent_info_with_type() {
        let _setup = SetupMocks::init();

        let c_json = concat!(r#"{"id":"123","value":"value", "type":1}"#, "\0").as_ptr().cast();

        let (h, cb, r) = return_types::return_u32();
        let _result = vcx_agent_update_info(h, c_json, Some(cb));
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_update_agent_fails() {
        let _setup = SetupMocks::init();

        AgencyMock::set_next_response(REGISTER_RESPONSE); //set response garbage

        let c_json = concat!(r#"{"id":"123"}"#, "\0").as_ptr().cast();

        let (h, cb, _r) = return_types::return_u32();
        assert_eq!(vcx_agent_update_info(h,
                                         c_json,
                                         Some(cb)),
                   error::INVALID_JSON.code_num);
    }

    #[test]
    fn test_get_ledger_fees() {
        let _setup = SetupMocks::init();

        let (h, cb, _r) = return_types::return_u32_str();
        assert_eq!(vcx_ledger_get_fees(h,
                                       Some(cb)),
                   error::SUCCESS.code_num);
    }

    #[test]
    fn test_messages_download() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_messages_download(h, ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_messages_update_status() {
        let _setup = SetupMocks::init();

        let status = "MS-103\0".as_ptr().cast();
        let json = concat!(r#"[{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]}]"#, "\0").as_ptr().cast();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_messages_update_status(h,
                                              status,
                                              json,
                                              Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    fn test_health_check() {
        let _setup = SetupLibraryAgencyV2ZeroFeesNewProvisioning::init();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(
            vcx_health_check(h, Some(cb)),
            error::SUCCESS.code_num
        );
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_health_check_failure() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(
            vcx_health_check(h, Some(cb)),
            error::SUCCESS.code_num
        );
        r.recv_medium().unwrap_err();
    }
}

