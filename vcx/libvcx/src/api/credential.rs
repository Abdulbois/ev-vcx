use libc::c_char;
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use crate::credential;
use std::ptr;
use crate::utils::threadpool::spawn;
use crate::error::prelude::*;
use vdrtools_sys::CommandHandle;

use crate::connection::Connections;
use crate::credential::Credentials;
use crate::aries::messages::issuance::credential_offer::CredentialOffer;
use crate::utils::object_cache::Handle;

/*
    The API represents a Holder side in credential issuance process.
    Assumes that pairwise connection between Issuer and Holder is already established.

    # State

    The set of object states, agent and transitions depends on the communication method is used.
    There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
    The communication method can be specified as a config option on one of *_init functions.

    proprietary:
        VcxStateType::VcxStateRequestReceived - once `vcx_credential_create_with_offer` (create Credential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `CRED_REQ` message) is called.

        VcxStateType::VcxStateAccepted - once `CRED` agent is received.
                                         use `vcx_credential_update_state` or `vcx_credential_update_state_with_message` functions for state updates.

    aries:
        VcxStateType::VcxStateRequestReceived - once `vcx_credential_create_with_offer` (create Credential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `CredentialRequest` message) is called.

        VcxStateType::VcxStateAccepted - once `Credential` agent is received.

        VcxStateType::VcxStateRejected - 1) once `ProblemReport` agent is received.
                                            use `vcx_credential_update_state` or `vcx_credential_update_state_with_message` functions for state updates.
                                         2) once `vcx_credential_reject` is called.

    # Transitions

    proprietary:
        VcxStateType::None - `vcx_credential_create_with_offer` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_credential_send_request` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `CRED` - VcxStateType::VcxStateAccepted

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
        VcxStateType::None - `vcx_credential_create_with_offer` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent
        VcxStateType::VcxStateRequestReceived - `vcx_credential_reject` - VcxStateType::None

        VcxStateType::VcxStateOfferSent - received `Credential` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::VcxStateRejected
        VcxStateType::VcxStateOfferSent - `vcx_credential_reject` - VcxStateType::VcxStateRejected

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

/// Retrieve Payment Transaction Information for this Credential. Typically this will include
/// how much payment is requried by the issuer, which needs to be provided by the prover, before the issuer will
/// issue the credential to the prover. Ideally a prover would want to know how much payment is being asked before
/// submitting the credential request (which triggers the payment to be made).
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides Payment Info of a Credential
///
/// # Example:
/// payment_info ->
///     {
///         "payment_required":"one-time",
///         "payment_addr":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j",
///         "price":1
///     }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_get_payment_info(command_handle: CommandHandle,
                                              credential_handle: Handle<Credentials>,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, *const c_char)>) -> u32 {
    info!("vcx_credential_get_payment_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    spawn(move || {
        match credential_handle.get_payment_information() {
            Ok(p) => {
                match p {
                    Some(p) => {
                        let info = p.to_string().unwrap_or("{}".to_string());
                        trace!("vcx_credential_get_payment_info(command_handle: {}, rc: {}, msg: {})", command_handle, error::SUCCESS.code_num, secret!(info));
                        let msg = CStringUtils::string_to_cstring(info);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    None => {
                        let msg = CStringUtils::string_to_cstring(format!("{{}}"));
                        trace!("vcx_credential_get_payment_info(command_handle: {}, rc: {}, msg: {})", command_handle, error::SUCCESS.code_num, "{}");
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                }
            }
            Err(e) => {
                warn!("vcx_credential_get_payment_info(command_handle: {}, rc: {}, msg: {})",
                      command_handle, e, "{}");
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Parse an Aries Credential Offer message
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// offer: received credential offer message
///
/// # Example
/// offer -> {"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential", "@id":"<uuid-of-offer-message>", "comment":"somecomment", "credential_preview":<json-ldobject>, "offers~attach":[{"@id":"libindy-cred-offer-0", "mime-type":"application/json", "data":{"base64":"<bytesforbase64>"}}]}
///
/// cb: Callback that provides credential offer info or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_parse_offer(command_handle: CommandHandle,
                                         offer: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                              err: u32,
                                                              info: *const c_char)>) -> u32 {
    info!("vcx_credential_parse_offer >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(offer, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_parse_offer(command_handle: {}, offer: {})",
           command_handle, secret!(&offer));

    spawn(move || {
        match CredentialOffer::parse(&offer) {
            Ok(info) => {
                trace!("vcx_credential_parse_offer_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.as_str(), info);
                let info = CStringUtils::string_to_cstring(info);
                cb(command_handle, error::SUCCESS.code_num, info.as_ptr())
            }
            Err(x) => {
                warn!("vcx_credential_parse_offer_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Credential object that requests and receives a credential for an institution
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the credential, should be unique.
///
/// offer: credential offer received via "vcx_credential_get_offers"
///
/// # Example
/// offer -> depends on communication method:
///     proprietary:
///         [{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]
///     aries:
///         {"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential", "@id":"<uuid-of-offer-message>", "comment":"somecomment", "credential_preview":<json-ldobject>, "offers~attach":[{"@id":"libindy-cred-offer-0", "mime-type":"application/json", "data":{"base64":"<bytesforbase64>"}}]}
///
/// cb: Callback that provides credential handle or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_create_with_offer(command_handle: CommandHandle,
                                               source_id: *const c_char,
                                               offer: *const c_char,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_handle: Handle<Credentials>)>) -> u32 {
    info!("vcx_credential_create_with_offer >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(offer, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_create_with_offer(command_handle: {}, source_id: {}, offer: {})",
           command_handle, source_id, secret!(&offer));

    spawn(move || {
        match credential::credential_create_with_offer(&source_id, &offer) {
            Ok(x) => {
                trace!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {})",
                       command_handle, source_id, error::SUCCESS.as_str(), x);
                cb(command_handle, error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {})",
                      command_handle, source_id, x, 0);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Accept credential for the given offer.
///
/// This function performs the following actions:
/// 1. Creates Credential state object that requests and receives a credential for an institution.
///     (equal to `vcx_credential_create_with_offer` function).
/// 2. Prepares Credential Request and replies to the issuer.
///     (equal to `vcx_credential_send_request` function).
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: institution's personal identification for the credential, should be unique.
///
/// offer: credential offer received from the issuer.
///
/// connection_handle: handle that identifies pairwise connection object with the issuer.
///
/// # Example
/// offer -> depends on communication method:
///     proprietary:
///         [
///             {
///                 "msg_type":"CREDENTIAL_OFFER",
///                 "version":"0.1",
///                 "to_did":"...",
///                 "from_did":"...",
///                 "credential":{
///                     "account_num":[
///                         "...."
///                     ],
///                     "name_on_account":[
///                         "Alice"
///                      ]
///                 },
///                 "schema_seq_no":48,
///                 "issuer_did":"...",
///                 "credential_name":"Account Certificate",
///                 "credential_id":"3675417066",
///                 "msg_ref_id":"ymy5nth"
///             }
///         ]
///     aries:
///         {
///             "@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential",
///             "@id":"<uuid-of-offer-message>",
///             "comment":"somecomment",
///             "credential_preview":"<json-ldobject>",
///             "offers~attach":[
///                 {
///                     "@id":"libindy-cred-offer-0",
///                     "mime-type":"application/json",
///                     "data":{
///                         "base64":"<bytesforbase64>"
///                     }
///                 }
///             ]
///         }
///
/// cb: Callback that provides credential handle or error status
///
/// # Returns
/// err: the result code as a u32
/// credential_handle: the handle associated with the created Credential state object.
/// credential_serialized: the json string representing the created Credential state object.
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_accept_credential_offer(command_handle: CommandHandle,
                                                     source_id: *const c_char,
                                                     offer: *const c_char,
                                                     connection_handle: Handle<Connections>,
                                                     cb: Option<extern fn(
                                                         xcommand_handle: CommandHandle,
                                                         err: u32,
                                                         credential_handle: Handle<Credentials>,
                                                         credential_serialized: *const c_char)>) -> u32 {
    info!("vcx_credential_accept_credential_offer >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(offer, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_accept_credential_offer(command_handle: {}, source_id: {}, offer: {}, connection_handle: {:?})",
           command_handle, source_id, secret!(offer), connection_handle);

    spawn(move || {
        match credential::accept_credential_offer(&source_id, &offer, connection_handle) {
            Ok((credential_handle, credential_serialized)) => {
                trace!("vcx_credential_accept_credential_offer(command_handle: {}, rc: {}, credential_handle: {}, credential_serialized: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), credential_handle, secret!(credential_serialized), source_id);
                let credential_serialized_ = CStringUtils::string_to_cstring(credential_serialized);
                cb(command_handle, error::SUCCESS.code_num, credential_handle, credential_serialized_.as_ptr());
            }
            Err(x) => {
                warn!("vcx_credential_accept_credential_offer(command_handle: {}, rc: {}) source_id: {}",
                      command_handle, x, source_id);
                cb(command_handle, x.into(), Handle::dummy(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieve information about a stored credential in user's wallet, including credential id and the credential itself.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides error status of api call, or returns the credential in json format of "{uuid:credential}".
///
/// # Example
/// credential -> depends on communication method:
///     proprietary:
///         {"credential_id":"cred_id", "credential": {"libindy_cred":"{....}","rev_reg_def_json":"","cred_def_id":"cred_def_id","msg_type":"CLAIM","claim_offer_id":"1234","version":"0.1","from_did":"did"}}
///     aries:
///         https://github.com/hyperledger/aries-rfcs/tree/master/features/0036-issue-credential#issue-credential
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_get_credential(command_handle: CommandHandle,
                                 credential_handle: Handle<Credentials>,
                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential: *const c_char)>) -> u32 {
    info!("vcx_get_credential >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_get_credential(command_handle: {}, credential_handle: {}))",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.get_credential() {
            Ok(s) => {
                trace!("vcx_get_credential_cb(commmand_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.code_num, secret!(s));
                let msg = CStringUtils::string_to_cstring(s);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                error!("vcx_get_credential_cb(commmand_handle: {}, rc: {}, msg: {})",
                       command_handle, e, "".to_string());
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Delete a Credential associated with the state object from the Wallet and release handle of the state object.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: handle pointing to credential state object to delete.
///
/// cb: Callback that provides error status of delete credential request
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_delete_credential(command_handle: CommandHandle,
                                    credential_handle: Handle<Credentials>,
                                    cb: Option<extern fn(
                                        xcommand_handle: CommandHandle,
                                        err: u32)>) -> u32 {
    info!("vcx_delete_credential >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_delete_credential(command_handle: {}, credential_handle: {}))",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.delete_credential() {
            Ok(_) => {
                trace!("vcx_delete_credential_cb(command_handle: {}, rc: {}), credential_handle: {})",
                       command_handle, error::SUCCESS.as_str(), credential_handle);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                trace!("vcx_delete_credential_cb(command_handle: {}, rc: {}), credential_handle: {})",
                       command_handle, e, credential_handle);
                cb(command_handle, e.into());
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Credential object based off of a known message id for a given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the credential, should be unique.
///
/// connection_handle: connection to query for credential offer
///
/// msg_id: msg_id that contains the credential offer
///
/// cb: Callback that provides credential handle or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_credential_create_with_msgid(command_handle: CommandHandle,
                                               source_id: *const c_char,
                                               connection_handle: Handle<Connections>,
                                               msg_id: *const c_char,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_handle: Handle<Credentials>, offer: *const c_char)>) -> u32 {
    info!("vcx_credential_create_with_msgid >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_id, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_create_with_msgid(command_handle: {}, source_id: {}, connection_handle: {}, msg_id: {})",
           command_handle, source_id, connection_handle, msg_id);

    spawn(move || {
        match credential::credential_create_with_msgid(&source_id, connection_handle, &msg_id) {
            Ok((handle, offer_string)) => {
                let offer_string = match handle.get_credential_offer() {
                    Ok(x) => x,
                    Err(_) => offer_string,
                };
                trace!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {}, offer_string: {:?}) source_id: {}",
                       command_handle, source_id, error::SUCCESS.as_str(), handle, secret!(offer_string), source_id);
                let c_offer = CStringUtils::string_to_cstring(offer_string);
                cb(command_handle, error::SUCCESS.code_num, handle, c_offer.as_ptr())
            }
            Err(e) => {
                warn!("vcx_credential_create_with_offer_cb(command_handle: {}, source_id: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, source_id, e, 0, source_id);
                cb(command_handle, e.into(), Handle::dummy(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Approves the credential offer and submits a credential request. The result will be a credential stored in the prover's wallet.
///
/// #params
/// command_handle: command handle to map callback to user context
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// connection_handle: Connection handle that identifies pairwise connection.
///                    Pass `0` in order to reply on ephemeral/connectionless credential offer
///                    Ephemeral/Connectionless Credential Offer contains `~server` decorator
///
/// cb: Callback that provides error status of credential request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_send_request(command_handle: CommandHandle,
                                          credential_handle: Handle<Credentials>,
                                          connection_handle: Handle<Connections>,
                                          _payment_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_credential_send_request >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_send_request(command_handle: {}, credential_handle: {}, connection_handle: {})",
           command_handle, credential_handle, connection_handle);

    spawn(move || {
        match credential_handle.send_credential_request(connection_handle) {
            Ok(x) => {
                trace!("vcx_credential_send_request_cb(command_handle: {}, rc: {})",
                       command_handle, x.to_string());
                cb(command_handle, x);
            }
            Err(e) => {
                warn!("vcx_credential_send_request_cb(command_handle: {}, rc: {})",
                      command_handle, e);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Approves the credential offer and gets the credential request message that can be sent to the specified connection
///
/// #params
/// command_handle: command handle to map callback to user context
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// my_pw_did: Use Connection api (vcx_connection_get_pw_did) with specified connection_handle to retrieve your pw_did
///
/// their_pw_did: Use Connection api (vcx_connection_get_their_pw_did) with specified connection_handle to retrieve theri pw_did
///
/// cb: Callback that provides error status of credential request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_get_request_msg(command_handle: CommandHandle,
                                             credential_handle: Handle<Credentials>,
                                             my_pw_did: *const c_char,
                                             their_pw_did: *const c_char,
                                             _payment_handle: u32,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_credential_get_request_msg >>>");

    check_useful_c_str!(my_pw_did, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(their_pw_did, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_get_request_msg(command_handle: {}, credential_handle: {}, my_pw_did: {}, their_pw_did: {:?})",
           command_handle, credential_handle, secret!(my_pw_did), secret!(their_pw_did));

    spawn(move || {
        match credential_handle.generate_credential_request_msg(&my_pw_did, &their_pw_did.unwrap_or_default()) {
            Ok(msg) => {
                trace!("vcx_credential_get_request_msg_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                warn!("vcx_credential_get_request_msg_cb(command_handle: {}, rc: {})",
                      command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Queries agency for credential offers from the given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection to query for credential offers.
///
/// cb: Callback that provides any credential offers and error status of query
///
/// # Example offers -> "[[{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]]"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_get_offers(command_handle: CommandHandle,
                                        connection_handle: Handle<Connections>,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, credential_offers: *const c_char)>) -> u32 {
    info!("vcx_credential_get_offers >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_get_offers(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match credential::get_credential_offer_messages(connection_handle) {
            Ok(x) => {
                trace!("vcx_credential_get_offers_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, x.to_string(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_credential_get_offers_cb(command_handle: {}, rc: {}, msg: null)",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Query the agency for the received agent.
/// Checks for any agent changing state in the credential object and updates the state attribute.
/// If it detects a credential it will store the credential in the wallet.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides most current state of the credential and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_update_state(command_handle: CommandHandle,
                                          credential_handle: Handle<Credentials>,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_update_state(command_handle: {}, credential_handle: {})",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.update_state(None) {
            Ok(state) => {
                trace!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), state);
                cb(command_handle, error::SUCCESS.code_num, state)
            }
            Err(e) => {
                error!("vcx_credential_update_state_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, e, 0);
                cb(command_handle, e.into(), 0)
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
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_update_state_with_message(command_handle: CommandHandle,
                                                       credential_handle: Handle<Credentials>,
                                                       message: *const c_char,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_update_state_with_message(command_handle: {}, credential_handle: {}, message: {})",
           command_handle, credential_handle, secret!(message));

    spawn(move || {
        match credential_handle.update_state(Some(message)) {
            Ok(state) => {
                trace!("vcx_credential_update_state_with_message_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), state);
                cb(command_handle, error::SUCCESS.code_num, state)
            }
            Err(e) => {
                error!("vcx_credential_update_state_with_message_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, e, 0);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the current state of the credential object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Credential handle that was provided during creation.
///
/// cb: Callback that provides most current state of the credential and error status of request
///     Credential statuses:
///         2 - Request Sent
///         3 - Request Received
///         4 - Accepted
///
/// #Returns
#[no_mangle]
pub extern fn vcx_credential_get_state(command_handle: CommandHandle,
                                       handle: Handle<Credentials>,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_credential_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_get_state(command_handle: {}, credential_handle: {})",
           command_handle, handle);

    spawn(move || {
        match handle.get_state() {
            Ok(s) => {
                trace!("vcx_credential_get_state_cb(command_handle: {}, rc: {}, state: {}),",
                       command_handle, error::SUCCESS.as_str(), s);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_credential_get_state_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, e, 0);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Takes the credential object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides json string of the credential's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_serialize(command_handle: CommandHandle,
                                       handle: Handle<Credentials>,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>) -> u32 {
    info!("vcx_credential_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_serialize(command_handle: {}, credential_handle: {})",
           command_handle, handle);

    spawn(move || {
        match handle.to_string() {
            Ok(x) => {
                trace!("vcx_credential_serialize_cb(command_handle: {}, rc: {}, data: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_credential_serialize_cb(command_handle: {}, rc: {}, data: {})",
                       command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing an credential object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_data: json string representing a credential object
///
///
/// cb: Callback that provides credential handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_deserialize(command_handle: CommandHandle,
                                         credential_data: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: Handle<Credentials>)>) -> u32 {
    info!("vcx_credential_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(credential_data, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_deserialize(command_handle: {}, credential_data: {})",
           command_handle, secret!(credential_data));

    spawn(move || {
        match credential::from_string(&credential_data) {
            Ok(x) => {
                trace!("vcx_credential_deserialize_cb(command_handle: {}, rc: {}, credential_handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);

                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(x) => {
                error!("vcx_credential_deserialize_cb(command_handle: {}, rc: {}, credential_handle: {})",
                       command_handle, x, 0);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the credential object by de-allocating memory
///
/// #Params
/// handle: Credential handle that was provided during creation. Used to access credential object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_credential_release(handle: Handle<Credentials>) -> u32 {
    info!("vcx_credential_release >>>");

    spawn(move || {
        match handle.release() {
            Ok(()) => {
                trace!("vcx_credential_release(handle: {}, rc: {})",
                       handle, error::SUCCESS.as_str());
            }
            Err(_e) => {
                // FIXME logging here results in panic while python tests
                // error!("vcx_credential_release(handle: {}, rc: {})",
                //        handle, e);
            }
        };
        Ok(())
    });
    error::SUCCESS.code_num
}


/// Retrieve the payment transaction associated with this credential. This can be used to get the txn that
/// was used to pay the issuer from the prover.  This could be considered a receipt of payment from the payer to
/// the issuer.
///
/// #param
/// handle: credential handle that was provided during creation.  Used to access credential object.
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
pub extern fn vcx_credential_get_payment_txn(command_handle: CommandHandle,
                                             handle: Handle<Credentials>,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, txn: *const c_char)>) -> u32 {
    info!("vcx_credential_get_payment_txn >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_get_payment_txn(command_handle: {})", command_handle);

    spawn(move || {
        match handle.get_payment_txn() {
            Ok(x) => {
                match serde_json::to_string(&x) {
                    Ok(x) => {
                        trace!("vcx_credential_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {})",
                               command_handle, error::SUCCESS.as_str(), secret!(x));

                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, 0, msg.as_ptr());
                    }
                    Err(e) => {
                        let err = VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize payment txn as JSON. Error: {:?}", e));
                        error!("vcx_credential_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {})",
                               command_handle, err, "null");
                        cb(command_handle, err.into(), ptr::null_mut());
                    }
                }
            }
            Err(x) => {
                error!("vcx_credential_get_payment_txn_cb(command_handle: {}, rc: {}, txn: {})",
                       command_handle, x, "null");
                cb(command_handle, x.into(), ptr::null());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a Credential rejection to the connection.
/// It can be called once Credential Offer or Credential agent are received.
///
/// Note that this function can be used for `aries` communication protocol.
/// In other cases it returns ActionNotSupported error.
///
/// #params
/// command_handle: command handle to map callback to user context
///
/// credential_handle: handle pointing to created Credential object.
///
/// connection_handle:  handle pointing to Connection object identifying pairwise connection.
///
/// comment: (Optional) human-friendly message to insert into Reject message.
///
/// cb: Callback that provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_reject(command_handle: CommandHandle,
                                    credential_handle: Handle<Credentials>,
                                    connection_handle: Handle<Connections>,
                                    comment: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_credential_reject >>>");

    check_useful_opt_c_str!(comment, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_reject(command_handle: {}, credential_handle: {}, connection_handle: {}, comment: {:?})",
           command_handle, credential_handle, connection_handle, secret!(comment));

    spawn(move || {
        match credential_handle.reject(connection_handle, comment) {
            Ok(()) => {
                trace!("vcx_credential_reject_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.code_num);
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_credential_reject_cb(command_handle: {}, rc: {})",
                      command_handle, e);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Build Presentation Proposal message for revealing Credential data.
///
/// Presentation Proposal is an optional message that can be sent by the Prover to the Verifier to
/// initiate a Presentation Proof process.
///
/// Presentation Proposal Format: https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof#propose-presentation
///
/// EXPERIMENTAL
///
/// #params
/// command_handle: command handle to map callback to user context
///
/// credential_handle: handle pointing to Credential to use for Presentation Proposal message building
///
/// cb: Callback that provides Presentation Proposal as json string and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_get_presentation_proposal_msg(command_handle: CommandHandle,
                                                           credential_handle: Handle<Credentials>,
                                                           cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                                err: u32,
                                                                                msg: *const c_char)>) -> u32 {
    info!("vcx_credential_get_presentation_proposal_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_get_presentation_proposal_msg(command_handle: {}, credential_handle: {})",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.get_presentation_proposal_msg() {
            Ok(msg) => {
                trace!("vcx_credential_get_presentation_proposal_msg_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                warn!("vcx_credential_get_presentation_proposal_msg_cb(command_handle: {}, rc: {})",
                      command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get Problem Report message for Credential object in Failed or Rejected state.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: handle pointing to Credential state object.
///
/// cb: Callback that returns Problem Report as JSON string or null
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_credential_get_problem_report(command_handle: CommandHandle,
                                                credential_handle: Handle<Credentials>,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                     err: u32,
                                                                     message: *const c_char)>) -> u32 {
    info!("vcx_credential_get_problem_report >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_credential_get_problem_report(command_handle: {}, credential_handle: {})",
           command_handle, credential_handle);

    spawn(move || {
        match credential_handle.get_problem_report_message() {
            Ok(message) => {
                trace!("vcx_credential_get_problem_report_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(message));
                let message = CStringUtils::string_to_cstring(message);
                cb(command_handle, error::SUCCESS.code_num, message.as_ptr());
            }
            Err(x) => {
                error!("vcx_credential_get_problem_report_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieve information about a stored credential.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// credential_handle: credential handle that was provided during creation. Used to identify credential object
///
/// cb: Callback that provides error status of api call, or returns the credential information in json format.
/// {
///     "referent": string, // cred_id in the wallet
///     "attrs": {"key1":"raw_value1", "key2":"raw_value2"},
///     "schema_id": string,
///     "cred_def_id": string,
///     "rev_reg_id": Optional<string>,
///     "cred_rev_id": Optional<string>
/// }
///
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
#[no_mangle]
pub extern fn vcx_credential_get_info(command_handle: CommandHandle,
                                      credential_handle: Handle<Credentials>,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, *const c_char)>) -> u32 {
    info!("vcx_credential_get_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    spawn(move || {
        match credential_handle.get_info() {
            Ok(info) => {
                trace!("vcx_credential_get_info_cb(command_handle: {}, rc: {}, info: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(info));
                let info = CStringUtils::string_to_cstring(info);
                cb(command_handle, error::SUCCESS.code_num, info.as_ptr());
            }
            Err(e) => {
                warn!("vcx_credential_get_info_cb(command_handle: {}, rc: {})",
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
    use crate::connection;
    use crate::api::VcxStateType;
    use crate::api::return_types;
    use serde_json::Value;
    use crate::utils::constants::{DEFAULT_SERIALIZED_CREDENTIAL, FULL_CREDENTIAL_SERIALIZED, PENDING_OBJECT_SERIALIZE_VERSION};
    use crate::utils::devsetup::*;
    use crate::utils::httpclient::AgencyMock;

    use crate::credential::tests::BAD_CREDENTIAL_OFFER;
    use crate::utils::constants;
    use crate::legacy::messages::issuance::credential_request::CredentialRequest;

    fn _vcx_credential_create_with_offer_c_closure(offer: &str) -> Result<Handle<Credentials>, u32> {
        let (h, cb, r) = return_types::return_u32_crdh();
        let offer = CString::new(offer).unwrap();
        let rc = vcx_credential_create_with_offer(h,
                                                  "test_create\0".as_ptr().cast(),
                                                  offer.as_ptr(),
                                                  Some(cb));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }

        let handle = r.recv_medium();
        handle
    }

    #[test]
    fn test_vcx_credential_create_with_offer_success() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(constants::CREDENTIAL_OFFER_JSON).unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_credential_create_with_offer_fails() {
        let _setup = SetupMocks::init();

        let err = _vcx_credential_create_with_offer_c_closure(BAD_CREDENTIAL_OFFER).unwrap_err();
        assert_eq!(err, error::INVALID_CREDENTIAL_OFFER.code_num);
    }

    #[test]
    fn test_vcx_credential_serialize_and_deserialize() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(constants::CREDENTIAL_OFFER_JSON).unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_credential_serialize(h,
                                            handle,
                                            Some(cb)), error::SUCCESS.code_num);
        let credential_json = r.recv_short().unwrap().unwrap();

        let object: Value = serde_json::from_str(&credential_json).unwrap();
        assert_eq!(object["version"], PENDING_OBJECT_SERIALIZE_VERSION);

        let (h, cb, r) = return_types::return_u32_crdh();
        let cred_json_cstr = CString::new(credential_json).unwrap();
        assert_eq!(vcx_credential_deserialize(h,
                                              cred_json_cstr.as_ptr(),
                                              Some(cb)), error::SUCCESS.code_num);
        let handle = r.recv_short().unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_credential_send_request() {
        let _setup = SetupMocks::init();

        let handle = credential::credential_create_with_offer("test_send_request", crate::utils::constants::CREDENTIAL_OFFER_JSON).unwrap();
        assert_eq!(handle.get_state().unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let connection_handle = connection::tests::build_test_connection();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_credential_send_request(h, handle, connection_handle, 0, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_credential_get_new_offers() {
        let _setup = SetupMocks::init();

        let cxn = crate::connection::tests::build_test_connection();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_credential_get_offers(h,
                                             cxn,
                                             Some(cb)),
                   error::SUCCESS.code_num as u32);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_credential_create() {
        let _setup = SetupMocks::init();

        let cxn = crate::connection::tests::build_test_connection();

        let (h, cb, r) = return_types::return_u32_crdh_str();
        assert_eq!(vcx_credential_create_with_msgid(h,
                                                    "test_vcx_credential_create\0".as_ptr().cast(),
                                                    cxn,
                                                    "123\0".as_ptr().cast(),
                                                    Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_credential_get_state() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(constants::CREDENTIAL_OFFER_JSON).unwrap();

        let (h, cb, r) = return_types::return_u32_u32();
        assert_eq!(vcx_credential_get_state(h, handle, Some(cb)), error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().unwrap(), VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_vcx_credential_update_state() {
        let _setup = SetupMocks::init();

        let cxn = crate::connection::tests::build_test_connection();

        let handle = credential::from_string(DEFAULT_SERIALIZED_CREDENTIAL).unwrap();

        AgencyMock::set_next_response(crate::utils::constants::NEW_CREDENTIAL_OFFER_RESPONSE);

        let (h, cb, r) = return_types::return_u32_u32();
        assert_eq!(vcx_credential_update_state(h, handle, Some(cb)), error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_credential_send_request(h, handle, cxn, 0, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_credential_get_request_msg() {
        let _setup = SetupMocks::init();

        let cxn = crate::connection::tests::build_test_connection();

        let my_pw_did = CString::new(cxn.get_pw_did().unwrap()).unwrap();
        let their_pw_did = CString::new(cxn.get_their_pw_did().unwrap()).unwrap();

        let handle = credential::from_string(DEFAULT_SERIALIZED_CREDENTIAL).unwrap();

        AgencyMock::set_next_response(crate::utils::constants::NEW_CREDENTIAL_OFFER_RESPONSE);

        let (h, cb, r) = return_types::return_u32_u32();
        assert_eq!(vcx_credential_update_state(h, handle, Some(cb)), error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_credential_get_request_msg(h, handle, my_pw_did.as_ptr(), their_pw_did.as_ptr(), 0, Some(cb)), error::SUCCESS.code_num);
        let msg = r.recv_medium().unwrap().unwrap();

        ::serde_json::from_str::<CredentialRequest>(&msg).unwrap();
    }

    #[test]
    fn test_get_credential() {
        let _setup = SetupMocks::init();

        let handle = credential::from_string(FULL_CREDENTIAL_SERIALIZED).unwrap();
        let bad_handle = Handle::from(1123);

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_get_credential(h, handle, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap().unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        vcx_get_credential(h, bad_handle, Some(cb));
        let rc = r.recv_medium().unwrap_err();
        assert_eq!(rc, error::INVALID_CREDENTIAL_HANDLE.code_num);

        let handle = credential::from_string(DEFAULT_SERIALIZED_CREDENTIAL).unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_get_credential(h, handle, Some(cb)), error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().err(), Some(error::NOT_READY.code_num));
    }

    #[test]
    fn test_vcx_credential_release() {
        let _setup = SetupMocks::init();

        let handle = _vcx_credential_create_with_offer_c_closure(constants::CREDENTIAL_OFFER_JSON).unwrap();
        assert_eq!(vcx_credential_release(handle), error::SUCCESS.code_num);
    }
}
