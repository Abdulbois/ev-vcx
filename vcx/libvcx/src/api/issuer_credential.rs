use libc::c_char;
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use crate::connection::Connections;
use crate::credential_def::CredentialDef;
use crate::issuer_credential::IssuerCredentials;
use crate::settings;
use crate::issuer_credential;
use std::ptr;
use crate::utils::threadpool::spawn;
use crate::error::prelude::*;
use vdrtools_sys::CommandHandle;
use crate::utils::object_cache::Handle;

/*
    The API represents an Issuer side in credential issuance process.
    Assumes that pairwise connection between Issuer and Holder is already established.

    # State

    The set of object states, agent and transitions depends on the communication method is used.
    There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
    The communication method can be specified as a config option on one of *_init functions.

    proprietary:
        VcxStateType::VcxStateInitialized - once `vcx_issuer_create_credential` (create IssuerCredential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_issuer_send_credential_offer` (send `CRED_OFFER` message) is called.

        VcxStateType::VcxStateRequestReceived - once `CRED_REQ` agent is received.
                                                use `vcx_issuer_credential_update_state` or `vcx_issuer_credential_update_state_with_message` functions for state updates.
        VcxStateType::VcxStateAccepted - once `vcx_issuer_send_credential` (send `CRED` message) is called.

    aries:
        VcxStateType::VcxStateInitialized - once `vcx_issuer_create_credential` (create IssuerCredential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_issuer_send_credential_offer` (send `CredentialOffer` message) is called.

        VcxStateType::VcxStateRequestReceived - once `CredentialRequest` agent is received.
        VcxStateType::None - once error occurred
                             use `vcx_issuer_credential_update_state` or `vcx_issuer_credential_update_state_with_message` functions for state updates.
        VcxStateType::VcxStateRejected - once `ProblemReport` agent is received.

        VcxStateType::VcxStateAccepted - once `vcx_issuer_send_credential` (send `Credential` message) is called.

    # Transitions

    proprietary:
        VcxStateType::None - `vcx_issuer_create_credential` - VcxStateType::VcxStateInitialized

        VcxStateType::VcxStateInitialized - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `CRED_REQ` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_issuer_send_credential` - VcxStateType::VcxStateAccepted

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
        VcxStateType::None - `vcx_issuer_create_credential` - VcxStateType::VcxStateInitialized

        VcxStateType::VcxStateInitialized - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `CredentialRequest` - VcxStateType::VcxStateRequestReceived
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::VcxStateRejected

        VcxStateType::VcxStateRequestReceived - vcx_issuer_send_credential` - VcxStateType::VcxStateAccepted

        VcxStateType::VcxStateAccepted - received `Ack` - VcxStateType::VcxStateAccepted

    # Messages

    proprietary:
        CredentialOffer (`CRED_OFFER`)
        CredentialRequest (`CRED_REQ`)
        Credential (`CRED`)

    aries:
        CredentialProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#propose-credential
        CredentialOffer - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#offer-credential
        CredentialRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#request-credential
        Credential - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#issue-credential
        ProblemReport - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0035-report-problem#the-problem-report-message-type
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
*/

/// Create a Issuer Credential object that provides a credential for an enterprise's user
/// Assumes a credential definition has been already written to the ledger.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Enterprise's personal identification for the credential, should be unique.
///
/// cred_def_id: id of credential definition given during creation of the credential definition
///
/// issuer_did: did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
///
/// credential_data: data attributes offered to person in the credential
///
/// credential_name: Name of the credential - ex. Drivers Licence
///
/// price: price of credential
///
/// cb: Callback that provides credential handle and error status of request
///
/// #Returns
/// Error code as a u32
///
/// # Example crendetial_data -> "{"state":"UT"}"
/// Note, that value can be empty: "{"middle_name":""}"
/// # Example credential_data -> "{"state":["UT"]}"  please note: this format is deprecated
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_issuer_create_credential(command_handle: CommandHandle,
                                           source_id: *const c_char,
                                           cred_def_handle: Handle<CredentialDef>,
                                           issuer_did: *const c_char,
                                           credential_data: *const c_char,
                                           credential_name: *const c_char,
                                           price: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_handle: Handle<IssuerCredentials>)>) -> u32 {
    info!("vcx_issuer_create_credential >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credential_data, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credential_name, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(price, VcxErrorKind::InvalidOption);

    let issuer_did: String = if !issuer_did.is_null() {
        check_useful_c_str!(issuer_did, VcxErrorKind::InvalidOption);
        issuer_did.to_owned()
    } else {
        match settings::get_config_value(settings::CONFIG_INSTITUTION_DID) {
            Ok(x) => x,
            Err(x) => return x.into()
        }
    };

    let price: u64 = match price.parse::<u64>() {
        Ok(x) => x,
        Err(err) => return VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot parse price: {}", err)).into(),
    };

    trace!("vcx_issuer_create_credential(command_handle: {}, source_id: {}, cred_def_handle: {}, issuer_did: {}, credential_data: {}, credential_name: {})",
           command_handle,
           source_id,
           cred_def_handle,
           secret!(issuer_did),
           secret!(&credential_data),
           secret!(credential_name));

    spawn(move || {
        let (rc, handle) = match issuer_credential::issuer_credential_create(cred_def_handle, source_id, issuer_did, credential_name, credential_data, price) {
            Ok(x) => {
                trace!("vcx_issuer_create_credential_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_issuer_create_credential_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                (x.into(), Handle::dummy())
            }
        };

        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a credential offer to user showing what will be included in the actual credential
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of credential offer
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_send_credential_offer(command_handle: CommandHandle,
                                               credential_handle: Handle<IssuerCredentials>,
                                               connection_handle: Handle<Connections>,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_issuer_send_credential_offer >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_send_credential_offer(command_handle: {}, credential_handle: {}, connection_handle: {})",
           command_handle, credential_handle, connection_handle);

    spawn(move || {
        let err = match credential_handle.send_credential_offer(connection_handle) {
            Ok(x) => {
                trace!("vcx_issuer_send_credential_cb(command_handle: {}, credential_handle: {}, rc: {})",
                       command_handle, credential_handle, error::SUCCESS.as_str());
                x
            }
            Err(x) => {
                warn!("vcx_issuer_send_credential_cb(command_handle: {}, credential_handle: {}, rc: {}))",
                      command_handle, credential_handle, x);
                x.into()
            }
        };

        cb(command_handle, err);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Gets the offer message that can be sent to the specified connection
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides error status of credential offer
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_get_credential_offer_msg(command_handle: CommandHandle,
                                                  credential_handle: Handle<IssuerCredentials>,
                                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_issuer_get_credential_offer_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_get_credential_offer_msg(command_handle: {}, credential_handle: {})",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.generate_credential_offer_msg() {
            Ok(msg) => {
                trace!("vcx_issuer_get_credential_offer_msg_cb(command_handle: {}, credential_handle: {}, rc: {}, msg: {})",
                       command_handle, credential_handle, error::SUCCESS.as_str(), secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_issuer_get_credential_offer_msg_cb(command_handle: {}, credential_handle: {}, rc: {}))",
                      command_handle, credential_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Query the agency for the received agent.
/// Checks for any agent changing state in the object and updates the state attribute.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides most current state of the credential and error status of request
///     States:
///         1 - Initialized
///         2 - Offer Sent
///         3 - Request Received
///         4 - Issued
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_credential_update_state(command_handle: CommandHandle,
                                                 credential_handle: Handle<IssuerCredentials>,
                                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_issuer_credential_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_credential_update_state(command_handle: {}, credential_handle: {})",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.update_state(None) {
            Ok(state) => {
                trace!("vcx_issuer_credential_update_state_cb(command_handle: {}, credential_handle: {}, rc: {}, state: {})",
                       command_handle, credential_handle, error::SUCCESS.as_str(), state);
                cb(command_handle, error::SUCCESS.code_num, state);
            }
            Err(x) => {
                warn!("vcx_issuer_credential_update_state_cb(command_handle: {}, credential_handle: {}, rc: {}, state: {})",
                      command_handle, credential_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Update the state of the credential based on the given message.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// message: message to process for state changes
///
/// cb: Callback that provides most current state of the credential and error status of request
///     States:
///         1 - Initialized
///         2 - Offer Sent
///         3 - Request Received
///         4 - Issued
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_credential_update_state_with_message(command_handle: CommandHandle,
                                                              credential_handle: Handle<IssuerCredentials>,
                                                              message: *const c_char,
                                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_issuer_credential_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_credential_update_state_with_message(command_handle: {}, credential_handle: {}, message: {})",
           command_handle, credential_handle, secret!(message));

    spawn(move || {
        match credential_handle.update_state(Some(message)) {
            Ok(x) => {
                trace!("vcx_issuer_credential_update_state_with_message_cb(command_handle: {}, credential_handle: {}, rc: {}, state: {})",
                       command_handle, credential_handle, error::SUCCESS.as_str(), x);
                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(x) => {
                warn!("vcx_issuer_credential_update_state_with_message_cb(command_handle: {}, credential_handle: {}, rc: {}, state: {})",
                      command_handle, credential_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the current state of the issuer credential object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Issuer Credential handle that was provided during creation.
///
/// cb: Callback that provides most current state of the issuer credential and error status of request
///     States:
///         1 - Initialized
///         2 - Offer Sent
///         3 - Request Received
///         4 - Issued
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_credential_get_state(command_handle: CommandHandle,
                                              credential_handle: Handle<IssuerCredentials>,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_issuer_credential_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_credential_get_state(command_handle: {}, credential_handle: {})",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.get_state() {
            Ok(x) => {
                trace!("vcx_issuer_credential_get_state_cb(command_handle: {}, credential_handle: {}, rc: {}, state: {})",
                       command_handle, credential_handle, error::SUCCESS.as_str(), x);
                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(x) => {
                warn!("vcx_issuer_credential_get_state_cb(command_handle: {}, credential_handle: {}, rc: {}, state: {})",
                      command_handle, credential_handle, x, 0);
                cb(command_handle, x.into(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[allow(unused_variables, unused_mut)]
pub extern fn vcx_issuer_get_credential_request(credential_handle: Handle<IssuerCredentials>, credential_request: *mut c_char) -> u32 {
    info!("vcx_issuer_get_credential_request >>>");
    error::SUCCESS.code_num
}

#[allow(unused_variables, unused_mut)]
pub extern fn vcx_issuer_accept_credential(credential_handle: Handle<IssuerCredentials>) -> u32 {
    info!("vcx_issuer_accept_credential >>>");
    error::SUCCESS.code_num
}

/// Sends the credential to the end user (holder).
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of sending the credential
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_send_credential(command_handle: CommandHandle,
                                         credential_handle: Handle<IssuerCredentials>,
                                         connection_handle: Handle<Connections>,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_issuer_send_credential >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_send_credential(command_handle: {}, credential_handle: {}, connection_handle: {})",
           command_handle, credential_handle, connection_handle);

    spawn(move || {
        let err = match credential_handle.send_credential(connection_handle) {
            Ok(x) => {
                trace!("vcx_issuer_send_credential_cb(command_handle: {}, credential_handle: {}, rc: {})",
                       command_handle, credential_handle, error::SUCCESS.as_str());
                x
            }
            Err(x) => {
                warn!("vcx_issuer_send_credential_cb(command_handle: {}, credential_handle: {}, rc: {})",
                      command_handle, credential_handle, x);
                x.into()
            }
        };

        cb(command_handle, err);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Gets the credential message that can be sent to the user
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// my_pw_did: Use Connection api (vcx_connection_get_pw_did) with specified connection_handle to retrieve your pw_did
///
/// cb:  Callback that provides any error status of the credential
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_get_credential_msg(command_handle: CommandHandle,
                                            credential_handle: Handle<IssuerCredentials>,
                                            my_pw_did: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_issuer_get_credential_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(my_pw_did, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_get_credential_msg(command_handle: {}, credential_handle: {}, my_pw_did: {})",
           command_handle, credential_handle,  secret!(my_pw_did));

    spawn(move || {
        match credential_handle.generate_credential_msg(&my_pw_did) {
            Ok(msg) => {
                trace!("vcx_issuer_get_credential_msg_cb(command_handle: {}, credential_handle: {}, rc: {}, msg: {})",
                       command_handle, credential_handle, error::SUCCESS.as_str(), secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_issuer_get_credential_msg_cb(command_handle: {}, credential_handle: {}, rc: {})",
                      command_handle, credential_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[allow(unused_variables)]
pub extern fn vcx_issuer_terminate_credential(credential_handle: Handle<IssuerCredentials>, termination_type: u32, msg: *const c_char) -> u32 {
    info!("vcx_issuer_terminate_credential >>>");
    error::SUCCESS.code_num
}

/// Takes the credential object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides json string of the credential's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_credential_serialize(command_handle: CommandHandle,
                                              credential_handle: Handle<IssuerCredentials>,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_state: *const c_char)>) -> u32 {
    info!("vcx_issuer_credential_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_credential_serialize(credential_serialize(command_handle: {}, credential_handle: {})",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.to_string() {
            Ok(x) => {
                trace!("vcx_issuer_credential_serialize_cb(command_handle: {}, credential_handle: {}, rc: {}, state: {})",
                       command_handle, credential_handle, error::SUCCESS.as_str(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                trace!("vcx_issuer_credential_serialize_cb(command_handle: {}, credential_handle: {}, rc: {}, state: {}))",
                       command_handle, credential_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing an issuer credential object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_data: json string representing a credential object
///
/// cb: Callback that provides credential handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_credential_deserialize(command_handle: CommandHandle,
                                                credential_data: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_handle: Handle<IssuerCredentials>)>) -> u32 {
    info!("vcx_issuer_credential_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credential_data, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_credential_deserialize(command_handle: {}, credential_data: {})",
           command_handle, secret!(credential_data));

    spawn(move || {
        let (rc, handle) = match issuer_credential::from_string(&credential_data) {
            Ok(x) => {
                trace!("vcx_issuer_credential_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_issuer_credential_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                (x.into(), Handle::dummy())
            }
        };

        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the issuer credential object by deallocating memory
///
/// #Params
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_issuer_credential_release(credential_handle: Handle<IssuerCredentials>) -> u32 {
    info!("vcx_issuer_credential_release >>>");

    spawn(move || {
        match credential_handle.release() {
            Ok(()) => {
                trace!("vcx_issuer_credential_release(credential_handle: {}, rc: {})",
                       credential_handle, error::SUCCESS.as_str());
            }
            Err(_e) => {
                // FIXME logging here results in panic while python tests
                // warn!("vcx_issuer_credential_release(credential_handle: {}), rc: {})",
                //       credential_handle, e);
            }
        };
        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieve the payment transaction associated with this credential. This can be used to get the txn that
/// was used to pay the issuer from the holder.
/// This could be considered a receipt of payment from the payer to the issuer.
///
/// #param
/// handle: issuer_credential handle that was provided during creation.  Used to access issuer_credential object.
///
/// #Callback returns
/// PaymentTxn json
/// example: {
///         "amount":25,
///         "inputs":[
///             "pay:null:1_3FvPC7dzFbQKzfG",
///             "pay:null:1_lWVGKc07Pyc40m6"
///         ],
///         "outputs":[
///             {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null},
///             {"recipient":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j","amount":25,"extra":null}
///         ]
///     }
#[no_mangle]
pub extern fn vcx_issuer_credential_get_payment_txn(command_handle: CommandHandle,
                                                    handle: Handle<IssuerCredentials>,
                                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, txn: *const c_char)>) -> u32 {
    info!("vcx_issuer_credential_get_payment_txn >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_credential_get_payment_txn(command_handle: {})", command_handle);

    spawn(move || {
        match handle.get_payment_txn() {
            Ok(x) => {
                match serde_json::to_string(&x) {
                    Ok(x) => {
                        trace!("vcx_issuer_credential_get_payment_txn_cb(command_handle: {}, rc: {}, : {})",
                               command_handle, error::SUCCESS.as_str(), secret!(x));

                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, 0, msg.as_ptr());
                    }
                    Err(e) => {
                        let err = VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize payment txn as JSON. Error: {}", e));
                        error!("vcx_issuer_credential_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {})",
                               command_handle, err, "null");
                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                }
            }
            Err(x) => {
                error!("vcx_issuer_credential_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {})",
                       command_handle, x, "null");
                cb(command_handle, x.into(), ptr::null());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Revoke Credential
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides error status of revoking the credential
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_revoke_credential(command_handle: CommandHandle,
                                           credential_handle: Handle<IssuerCredentials>,
                                           cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    info!("vcx_issuer_revoke_credential(command_handle: {}, credential_handle: {})",
          command_handle, credential_handle);

    spawn(move || {
        let err = match credential_handle.revoke_credential() {
            Ok(()) => {
                info!("vcx_issuer_revoke_credential_cb(command_handle: {}, credential_handle: {}, rc: {})",
                      command_handle, credential_handle, error::SUCCESS.as_str());
                error::SUCCESS.code_num
            }
            Err(x) => {
                warn!("vcx_issuer_revoke_credential_cb(command_handle: {}, credential_handle: {}, rc: {})",
                      command_handle, credential_handle, x);
                x.into()
            }
        };

        cb(command_handle, err);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get Problem Report message for Issuer Credential object in Failed or Rejected state.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: handle pointing to Issuer Credential state object.
///
/// cb: Callback that returns Problem Report as JSON string or null
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_issuer_credential_get_problem_report(command_handle: CommandHandle,
                                                       credential_handle: Handle<IssuerCredentials>,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                            err: u32,
                                                                            message: *const c_char)>) -> u32 {
    info!("vcx_issuer_credential_get_problem_report >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_issuer_credential_get_problem_report(command_handle: {}, credential_handle: {})",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.get_problem_report_message() {
            Ok(message) => {
                trace!("vcx_issuer_credential_get_problem_report_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(message));
                let message = CStringUtils::string_to_cstring(message);
                cb(command_handle, error::SUCCESS.code_num, message.as_ptr());
            }
            Err(x) => {
                error!("vcx_issuer_credential_get_problem_report_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;
    use crate::settings;
    use crate::utils::{
        constants::CREDENTIAL_REQ_RESPONSE_STR,
        get_temp_dir_path
    };
    use crate::api::{return_types, VcxStateType};
    use crate::utils::devsetup::*;

    const DEFAULT_CREDENTIAL_NAME_CSTR: *const c_char = "Credential Name Default\0".as_ptr().cast();
    const DEFAULT_DID_CSTR: *const c_char = "8XFh8yBzrpJQmNyZzgoTqB\0".as_ptr().cast();
    static DEFAULT_DID: &str = "8XFh8yBzrpJQmNyZzgoTqB";
    const DEFAULT_ATTR_CSTR: *const c_char = "{\"attr\":\"value\"}\0".as_ptr().cast();

    pub fn issuer_credential_state_accepted() -> String {
        json!({
            "version": "1.0",
            "data": {
                "cred_def_handle":1,
                "tails_file": get_temp_dir_path("tails").to_str().unwrap(),
                "rev_reg_id": "123",
                "cred_rev_id": "456",
                "source_id": "standard_credential",
                "credential_attributes": "{\"address2\":[\"101 Wilson Lane\"],\n        \"zip\":[\"87121\"],\n        \"state\":[\"UT\"],\n        \"city\":[\"SLC\"],\n        \"address1\":[\"101 Tela Lane\"]\n        }",
                "msg_uid": "1234",
                "schema_seq_no": 32,
                "issuer_did": "QTrbV4raAcND4DWWzBmdsh",
                "state": 3,
                "credential_request": {
                    "libindy_cred_req": "{\"prover_did\":\"2hoqvcwupRTUNkXn6ArYzs\",\"cred_def_id\":\"2hoqvcwupRTUNkXn6ArYzs:3:CL:1766\",\"blinded_ms\":{\"u\":\"8732071602357015307810566138808197234658312581785137109788113302982640059349967050965447489217593298616209988826723701562661343443517589847218013366407845073616266391756009264980040238952349445643778936575656535779015458023493903785780518101975701982901383514030208868847307622362696880263163343848494510595690307613204277848599695882210459126941797459019913953592724097855109613611647709745072773427626720401442235193011557232562555622244156336806151662441234847773393387649719209243455960347563274791229126202016215550120934775060992031280966045894859557271641817491943416048075445449722000591059568013176905304195\",\"ur\":null},\"blinded_ms_correctness_proof\":{\"c\":\"26530740026507431379491385424781000855170637402280225419270466226736067904512\",\"v_dash_cap\":\"143142764256221649591394190756594263575252787336888260277569702754606119430149731374696604981582865909586330696038557351486556018124278706293019764236792379930773289730781387402321307275066512629558473696520197393762713894449968058415758200647216768004242460019909604733610794104180629190082978779757591726666340720737832809779281945323437475154340615798778337960748836468199407007775031657682302038533398039806427675709453395148841959462470861915712789403465722659960342165041260269463103782446132475688821810775202828210979373826636650138063942962121467854349698464501455098258293105554402435773328031261630390919907379686173528652481917022556931483089035786146580024468924714494948737711000361399753716101561779590\",\"ms_cap\":\"6713785684292289748157544902063599004332363811033155861083956757033688921010462943169460951559595511857618896433311745591610892377735569122165958960965808330552472093346163460366\"},\"nonce\":\"1154549882365416803296713\"}",
                    "libindy_cred_req_meta": "{\"master_secret_blinding_data\":{\"v_prime\":\"5395355128172250143169068089431956784792642542761864362402228480600989694874966075941384260155648520933482583695015613159862636260075389615716222159662546164168786411292929058350829109114076583253317335067228793239648602609298582418017531463540043998240957993320093249294158252626231822371040785324638542033761124918129739329505169470758613520824786030494489920230941474441127178440612550463476183902911947132651422614577934309909240587823495239211344374406789215531181787691051240041033304085509402896936138071991158258582839272399829973882057207073602788766808713962858580770439194397272070900372124998541828707590819468056588985228490934\",\"vr_prime\":null},\"nonce\":\"1154549882365416803296713\",\"master_secret_name\":\"main\"}",
                    "cred_def_id": "2hoqvcwupRTUNkXn6ArYzs:3:CL:1766",
                    "tid": "cCanHnpFAD",
                    "to_did": "BnRXf8yDMUwGyZVDkSENeq",
                    "from_did": "GxtnGN6ypZYgEqcftSQFnC",
                    "version": "0.1",
                    "mid": "",
                    "msg_ref_id": "12345"
                },
                "credential_offer": {
                    "msg_type": "CRED_OFFER",
                    "version": "0.1",
                    "to_did": "8XFh8yBzrpJQmNyZzgoTqB",
                    "from_did": "8XFh8yBzrpJQmNyZzgoTqB",
                    "libindy_offer": "{\"schema_id\":\"2hoqvcwupRTUNkXn6ArYzs:2:schema_name:0.0.11\",\"cred_def_id\":\"2hoqvcwupRTUNkXn6ArYzs:3:CL:1766\",\"key_correctness_proof\":{\"c\":\"81455034389059130581506970475392033040313255495112570189348030990050944959723\",\"xz_cap\":\"313645697267968767252234073635675430449902008059550004460259716107399731378591839990019486954341409015811398444145390509019258403747288031702507727573872041899321045924287139508392740014051146807378366748171039375722083582850094590251566094137198468729226768809401256609008814847622114541957109991869490323195581928533376835343922482073783968747913611549869005687592623346914265913612170394649557294382253996246104002213172081216651539025706643350612557508228429410997102814965307308636524874409734625285377555470610010065029649043789306111101285927931757335536116856245613021564584847709796772325323716389295248332887528840195072737364278387101996545501723112970168561425282691953586374723401\",\"xr_cap\":{\"age\":\"882754630824080045376337358848444600715931719237593270810742883245639461185815851876695993155364347227577960272007297643455666310248109151421699898719086697252758726897984721300131927517824869533193272729923436764134176057310403382007926964744387461941410106739551156849252510593074993038770740497381973934250838808938096281745915721201706218145129356389886319652075267352853728443472451999347485331725183791798330085570375973775830893185375873153450320600510970851511952771344003741169784422212142610068911032856394030732377780807267819554991221318614567131747542069695452212861957610989952712388162117309870024706736915145245688230386906705817571265829695877232812698581971245658766976413035\",\"height\":\"987637616420540109240639213457114631238834322455397854134075974962516028070241761486895351636137675737583463907200584608953198912009428606796987435233170230262246507002244616435810064614719873830573727071246389627645604379157359983051337498205555868770767724876429776832782322071025598605854225056296405802351270140259313942108556513054492873024197036931111152136704979025907027537437514085689067466225661223523070057146052814725207863140129032189711026590245299845102901392525049014890473357388530510591717159458757929233202259332009161834669583439224425159885860519286698297401104830776447810193871233628235105641793685350321428066559473844839135685992587694149460959649026855973744322255314\",\"name\":\"1546639434545851623074023662485597065284112939224695559955181790271051962463722945049040324831863838273446566781589598791986646525127962031342679728936610678403807319789934638790962870799709103831307094501191346766422178361730723105585107221227683700136793784629414737866344469139276697568820727798174438114746109084012381033673759358527018948810066386903378176283974585934466197449653414224049202874335628877153172622300824161652402616917051692229112366954543190460604470158025596786552965425465904108943932508335616457348969058666355825158659883154681844070175331759147881082936624886840666700175491257446990494466033687900546604556189308597860524376648979247121908124398665458633017197827236\",\"sex\":\"716474787042335984121980741678479956610893721743783933016481046646620232719875607171626872246169633453851120125820240948330986140162546620706675695953306343625792456607323180362022779776451183315417053730047607706403536921566872327898942782065882640264019040337889347226013768331343768976174940163847488834059250858062959921604207705933170308295671034308248661208253191415678118624962846251281290296191433330052514696549137940098226268222146864337521249047457556625050919427268119508782974114298993324181252788789806496387982332099887944556949042187369539832351477275159404450154234059063271817130338030393531532967222197942953924825232879558249711884940237537025210406407183892784259089230597\"}},\"nonce\":\"161126724054910446992163\"}",
                    "cred_def_id": "2hoqvcwupRTUNkXn6ArYzs:3:CL:1766",
                    "credential_attrs": {
                        "address1":["101 Tela Lane"],
                        "address2":["101 Wilson Lane"],
                        "city":["SLC"],
                        "state":["UT"],
                        "zip":["87121"]
                    },
                    "schema_seq_no":1487,
                    "claim_name":"Credential",
                    "claim_id":"defaultCredentialId",
                    "msg_ref_id":"abcd"
                },
                "credential_name":"Credential",
                "credential_id":"defaultCredentialId",
                "cred_def_id":"2hoqvcwupRTUNkXn6ArYzs:3:CL:1766",
                "price":0,
                "ref_msg_id":"null",
                 "agent_info":{
                    "connection_handle":0,
                    "my_pw_did":"8XFh8yBzrpJQmNyZzgoTqB",
                    "my_pw_vk":"8XFh8yBzrpJQmNyZzgoTqB",
                    "their_pw_did":"8XFh8yBzrpJQmNyZzgoTqB",
                    "their_pw_vk":"8XFh8yBzrpJQmNyZzgoTqB",
                    "agent_did":"8XFh8yBzrpJQmNyZzgoTqB",
                    "agent_vk":"8XFh8yBzrpJQmNyZzgoTqB",
                    "agency_did":"8XFh8yBzrpJQmNyZzgoTqB",
                    "agency_vk":"8XFh8yBzrpJQmNyZzgoTqB"
                 },
            }
        }).to_string()
    }

    fn _vcx_issuer_create_credential_c_closure() -> Result<Handle<IssuerCredentials>, u32> {
        let (h, cb, r) = return_types::return_u32_ih();
        let rc = vcx_issuer_create_credential(h,
                                              DEFAULT_CREDENTIAL_NAME_CSTR,
                                              crate::credential_def::tests::create_cred_def_fake(),
                                              DEFAULT_DID_CSTR,
                                              DEFAULT_ATTR_CSTR,
                                              DEFAULT_CREDENTIAL_NAME_CSTR,
                                              "1\0".as_ptr().cast(),
                                              Some(cb));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        r.recv_short()
    }

    #[test]
    fn test_vcx_issuer_create_credential_success() {
        let _setup = SetupMocks::init();

        let handle = _vcx_issuer_create_credential_c_closure().unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_issuer_create_credential_fails() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_ih();
        assert_eq!(vcx_issuer_create_credential(h,
                                                DEFAULT_CREDENTIAL_NAME_CSTR,
                                                crate::credential_def::tests::create_cred_def_fake(),
                                                ptr::null(),
                                                ptr::null(),
                                                DEFAULT_CREDENTIAL_NAME_CSTR,
                                                "1\0".as_ptr().cast(),
                                                Some(cb)),
                   error::INVALID_OPTION.code_num);
        let _ = r.recv_medium().is_err();
    }

    #[test]
    fn test_vcx_issuer_credential_serialize_deserialize() {
        let _setup = SetupMocks::init();

        let handle = _vcx_issuer_create_credential_c_closure().unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_issuer_credential_serialize(h,
                                                   handle,
                                                   Some(cb)),
                   error::SUCCESS.code_num);
        let credential_json = r.recv_short().unwrap().unwrap();

        let (h, cb, r) = return_types::return_u32_ih();
        let cstr = CString::new(credential_json).unwrap();
        assert_eq!(vcx_issuer_credential_deserialize(h,
                                                     cstr.as_ptr(),
                                                     Some(cb)),
                   error::SUCCESS.code_num);
        let handle_2 = r.recv_short().unwrap();
        assert!(handle_2 > 0);

        assert_ne!(handle, handle_2);
    }

    #[test]
    fn test_vcx_issuer_send_credential_offer() {
        let _setup = SetupMocks::init();

        let connection_handle = crate::connection::tests::build_test_connection();

        let handle = _vcx_issuer_create_credential_c_closure().unwrap();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_issuer_send_credential_offer(h,
                                                    handle,
                                                    connection_handle,
                                                    Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();

        let (h, cb, r) = return_types::return_u32_u32();
        let cstr = CString::new(CREDENTIAL_REQ_RESPONSE_STR).unwrap();
        assert_eq!(vcx_issuer_credential_update_state_with_message(h,
                                                                   handle,
                                                                   cstr.as_ptr(),
                                                                   Some(cb)), error::SUCCESS.code_num);
        let state = r.recv_medium().unwrap();
        assert_eq!(state, VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_vcx_issuer_get_credential_offer_msg() {
        let _setup = SetupMocks::init();

        let handle = _vcx_issuer_create_credential_c_closure().unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_issuer_get_credential_offer_msg(h,
                                                       handle,
                                                       Some(cb)),
                   error::SUCCESS.code_num);
        let _msg = r.recv_medium().unwrap().unwrap();
    }

    #[test]
    fn test_vcx_issuer_send_a_credential() {
        let _setup = SetupMocks::init();

        // create connection
        let connection_handle = crate::connection::tests::build_test_connection();

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, DEFAULT_DID);
        let handle = issuer_credential::from_string(&issuer_credential_state_accepted()).unwrap();

        // send the credential
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_issuer_send_credential(h,
                                              handle,
                                              connection_handle,
                                              Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_issuer_get_credential_msg() {
        let _setup = SetupMocks::init();

        let handle = issuer_credential::from_string(&issuer_credential_state_accepted()).unwrap();

        // send the credential
        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_issuer_get_credential_msg(h,
                                                 handle,
                                                 DEFAULT_DID_CSTR,
                                                 Some(cb)),
                   error::SUCCESS.code_num);
        let _msg = r.recv_medium().unwrap().unwrap();
    }

    #[test]
    fn test_create_credential_arguments_correct() {
        let _setup = SetupMocks::init();

        let handle = _vcx_issuer_create_credential_c_closure().unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_issuer_credential_serialize(h,
                                                   handle,
                                                   Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_issuer_credential_get_state() {
        let _setup = SetupMocks::init();

        let handle = _vcx_issuer_create_credential_c_closure().unwrap();

        let (h, cb, r) = return_types::return_u32_u32();
        assert_eq!(vcx_issuer_credential_get_state(h,
                                                   handle,
                                                   Some(cb)),
                   error::SUCCESS.code_num);
        let state = r.recv_medium().unwrap();
        assert_eq!(state, VcxStateType::VcxStateInitialized as u32);
    }

    #[test]
    fn test_vcx_issuer_revoke_credential() {
        let _setup = SetupMocks::init();

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, DEFAULT_DID);
        let handle = issuer_credential::from_string(&issuer_credential_state_accepted()).unwrap();

        // send the credential
        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_issuer_revoke_credential(h,
                                                handle,
                                                Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_issuer_credential_release() {
        let _setup = SetupMocks::init();

        let handle = _vcx_issuer_create_credential_c_closure().unwrap();
        assert_eq!(vcx_issuer_credential_release(handle), error::SUCCESS.code_num);
    }
}
