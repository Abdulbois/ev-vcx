use libc::c_char;
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use std::ptr;
use crate::utils::threadpool::spawn;
use crate::error::prelude::*;
use vdrtools_sys::CommandHandle;
use crate::aries::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::disclosed_proof::{self, DisclosedProofs};
use crate::utils::object_cache::Handle;

use crate::connection::Connections;

/*
    APIs in this module are called by a prover throughout the request-proof-and-verify process.
    Assumes that pairwise connection between Verifier and Prover is already established.

    # State

    The set of object states, agent and transitions depends on the communication method is used.
    There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
    The communication method can be specified as a config option on one of *_init functions.

    proprietary:
        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_create_with_request` (create DisclosedProof object) is called.

        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_generate_proof` is called.

        VcxStateType::VcxStateAccepted - once `vcx_disclosed_proof_send_proof` (send `PROOF` message) is called.

    aries:
        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_create_with_request` (create DisclosedProof object) is called.

        VcxStateType::VcxStateRequestReceived - once `vcx_disclosed_proof_generate_proof` is called.

        VcxStateType::VcxStateOfferSent - once `vcx_disclosed_proof_send_proof` (send `Presentation` message) is called.
        VcxStateType::None - once `vcx_disclosed_proof_decline_presentation_request` (send `PresentationReject` or `PresentationProposal` message) is called.

        VcxStateType::VcxStateAccepted - once `Ack` agent is received.
        VcxStateType::None - once `ProblemReport` agent is received.

    # Transitions

    proprietary:
        VcxStateType::None - `vcx_disclosed_proof_create_with_request` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_generate_proof` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_send_proof` - VcxStateType::VcxStateAccepted

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        VcxStateType::None - `vcx_disclosed_proof_create_with_request` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_generate_proof` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_send_proof` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateRequestReceived - `vcx_disclosed_proof_decline_presentation_request` - VcxStateType::None

        VcxStateType::VcxStateOfferSent - received `Ack` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None

    # Messages

    proprietary:
        ProofRequest (`PROOF_REQ`)
        Proof (`PROOF`)

    aries:
        PresentationRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#request-presentation
        Presentation - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#presentation
        PresentationProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
*/

/// Parse aa Aries Proof Request message
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_request: received proof request message
///
/// cb: Callback that provides proof request info or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_disclosed_proof_parse_request(command_handle: CommandHandle,
                                                request: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                     err: u32,
                                                                     request_info: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_parse_request >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(request, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_parse_request(command_handle: {}, offer: {})",
           command_handle, secret!(&request));

    spawn(move || {
        match PresentationRequest::parse(&request) {
            Ok(info) => {
                trace!("vcx_disclosed_proof_parse_request_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.as_str(), info);
                let info = CStringUtils::string_to_cstring(info);
                cb(command_handle, error::SUCCESS.code_num, info.as_ptr())
            }
            Err(x) => {
                warn!("vcx_disclosed_proof_parse_request_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Proof object for fulfilling a corresponding proof request
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's identification for the proof, should be unique.
///
/// req: proof request received via "vcx_get_proof_requests"
///
/// cb: Callback that provides proof handle or error status
///
/// # Example proof_req -> "{"@topic":{"mid":9,"tid":1},"@type":{"name":"PROOF_REQUEST","version":"1.0"},"msg_ref_id":"ymy5nth","proof_request_data":{"name":"AccountCertificate","nonce":"838186471541979035208225","requested_attributes":{"business_2":{"name":"business"},"email_1":{"name":"email"},"name_0":{"name":"name"}},"requested_predicates":{},"version":"0.1"}}"
///
/// #Returns
/// Error code as u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_disclosed_proof_create_with_request(command_handle: CommandHandle,
                                                      source_id: *const c_char,
                                                      proof_req: *const c_char,
                                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: Handle<DisclosedProofs>)>) -> u32 {
    info!("vcx_disclosed_proof_create_with_request >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(proof_req, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_create_with_request(command_handle: {}, source_id: {}, proof_req: {})",
           command_handle, source_id, secret!(proof_req));

    spawn(move || {
        match disclosed_proof::create_proof(&source_id, &proof_req) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_create_with_request_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), x, source_id);
                cb(command_handle, 0, x);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_create_with_request_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, x, 0, source_id);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Create a proof based off of a known message id for a given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's personal identification for the proof, should be unique.
///
/// connection: connection to query for proof request
///
/// msg_id:  id of the message that contains the proof request
///
/// cb: Callback that provides proof handle and proof request or error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_disclosed_proof_create_with_msgid(command_handle: CommandHandle,
                                                    source_id: *const c_char,
                                                    connection_handle: Handle<Connections>,
                                                    msg_id: *const c_char,
                                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_handle: Handle<DisclosedProofs>, proof_req: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_create_with_msgid >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg_id, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_create_with_msgid(command_handle: {}, source_id: {}, connection_handle: {}, msg_id: {})",
           command_handle, source_id, connection_handle, msg_id);

    spawn(move || {
        match disclosed_proof::create_proof_with_msgid(&source_id, connection_handle, &msg_id) {
            Ok((handle, request)) => {
                trace!("vcx_disclosed_proof_create_with_msgid_cb(command_handle: {}, rc: {}, handle: {}, proof_req: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), handle,  secret!(request), source_id);
                let msg = CStringUtils::string_to_cstring(request);
                cb(command_handle, error::SUCCESS.code_num, handle, msg.as_ptr())
            }
            Err(e) => {
                cb(command_handle, e.into(), Handle::dummy(), ptr::null());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Proof object for fulfilling a corresponding proof proposal
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Institution's identification for the proof, should be unique.
///
/// proposal: the proposed format of presentation request
/// (see https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof#presentation-preview for details)
/// {
///    "attributes": [
///        {
///            "name": "<attribute_name>",
///            "cred_def_id": Optional("<cred_def_id>"),
///            "mime-type": Optional("<type>"),
///            "value": Optional("<value>")
///        },
///        // more attributes
///    ],
///    "predicates": [
///        {
///            "name": "<attribute_name>",
///            "cred_def_id": Optional("<cred_def_id>"),
///            "predicate": "<predicate>", - one of "<", "<=", ">=", ">"
///            "threshold": <threshold>
///        },
///        // more predicates
///    ]
/// }
///   An attribute specification must specify a value, a cred_def_id, or both:
///     if value is present and cred_def_id is absent, the preview proposes a self-attested attribute;
///     if value and cred_def_id are both present, the preview proposes a verifiable claim to reveal in the presentation;
///     if value is absent and cred_def_id is present, the preview proposes a verifiable claim not to reveal in the presentation.
///
/// # Example
///  proposal ->
///     {
///          "attributes": [
///              {
///                  "name": "first name"
///              }
///          ],
///          "predicates": [
///              {
///                  "name": "age",
///                  "predicate": ">",
///                  "threshold": 18
///              }
///          ]
///      }
///
/// comment: Comment sent with proposal.
///
/// cb: Callback that provides proof handle or error status
///
/// #Returns
/// Error code as u32
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_disclosed_proof_create_proposal(command_handle: CommandHandle,
                                                  source_id: *const c_char,
                                                  proposal: *const c_char,
                                                  comment: *const c_char,
                                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_handle: Handle<DisclosedProofs>)>) -> u32 {
    info!("vcx_disclosed_proof_create_proposal >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(proposal, VcxErrorKind::InvalidOption);
    check_useful_c_str!(comment, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_create_proposal(command_handle: {}, source_id: {}, proposal: {}, comment: {})",
           command_handle, source_id, secret!(proposal), secret!(comment));

    spawn(move || {
        let (rc, handle) = match disclosed_proof::create_proposal(&source_id, proposal, comment) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_create_proposal_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), x, x.get_source_id().unwrap_or_default());
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_disclosed_proof_create_proposal_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, x);
                (x.into(), Handle::dummy())
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a proof to the connection, called after having received a proof request
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// connection_handle: Connection handle that identifies pairwise connection.
///                    Pass `0` in order to reply on ephemeral/connectionless proof request
///                    Ephemeral/Connectionless Proof Request contains `~server` decorator
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_send_proof(command_handle: CommandHandle,
                                             proof_handle: Handle<DisclosedProofs>,
                                             connection_handle: Handle<Connections>,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_send_proof >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_send_proof(command_handle: {}, proof_handle: {}, connection_handle: {})",
           command_handle, proof_handle, connection_handle);

    spawn(move || {
        match proof_handle.send_proof(connection_handle) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_send_proof_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_send_proof_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a proof proposal to the connection, called after prepared a proof proposal
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_send_proposal(command_handle: CommandHandle,
                                                proof_handle: Handle<DisclosedProofs>,
                                                connection_handle: Handle<Connections>,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_send_proposal >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_send_proof(command_handle: {}, proof_handle: {}, connection_handle: {})",
           command_handle, proof_handle, connection_handle);

    spawn(move || {
        match proof_handle.send_proposal(connection_handle) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_send_proposal_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_send_proposal_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a proof rejection to the connection, called after having received a proof request
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_reject_proof(command_handle: CommandHandle,
                                               proof_handle: Handle<DisclosedProofs>,
                                               connection_handle: Handle<Connections>,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_reject_proof >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_reject_proof(command_handle: {}, proof_handle: {}, connection_handle: {})",
           command_handle, proof_handle, connection_handle);

    spawn(move || {
        match proof_handle.reject_proof(connection_handle) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_reject_proof_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_reject_proof_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the proof message for sending.
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_proof_msg(command_handle: CommandHandle,
                                                proof_handle: Handle<DisclosedProofs>,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_get_proof_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_get_proof_msg(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move || {
        match proof_handle.generate_proof_msg() {
            Ok(msg) => {
                trace!("vcx_disclosed_proof_get_proof_msg_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_get_proof_msg_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the reject proof message for sending.
///
/// #params
/// command_handle: command handle to map callback to API user context.
///
/// proof_handle: proof handle that was provided duration creation.  Used to identify proof object.
///
/// cb: Callback that provides error status of proof send request
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_reject_msg(command_handle: CommandHandle,
                                                 proof_handle: Handle<DisclosedProofs>,
                                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_get_reject_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_get_reject_msg(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move || {
        match proof_handle.generate_reject_proof_msg() {
            Ok(msg) => {
                trace!("vcx_disclosed_proof_get_reject_msg_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_get_reject_msg_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Queries agency for all pending proof requests from the given connection.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection to query for proof requests.
///
/// cb: Callback that provides any proof requests and error status of query
/// # Example requests -> "[{'@topic': {'tid': 0, 'mid': 0}, '@type': {'version': '1.0', 'name': 'PROOF_REQUEST'}, 'proof_request_data': {'name': 'proof_req', 'nonce': '118065925949165739229152', 'version': '0.1', 'requested_predicates': {}, 'non_revoked': None, 'requested_attributes': {'attribute_0': {'name': 'name', 'restrictions': {'$or': [{'issuer_did': 'did'}]}}}, 'ver': '1.0'}, 'thread_id': '40bdb5b2'}]"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_requests(command_handle: CommandHandle,
                                               connection_handle: Handle<Connections>,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, requests: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_get_requests >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_get_requests(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match disclosed_proof::get_proof_request_messages(connection_handle, None) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_get_requests_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_get_requests_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the current state of the disclosed proof object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access disclosed proof object
///
/// cb: Callback that provides most current state of the disclosed proof and error status of request
///     States:
///         3 - Request Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_state(command_handle: CommandHandle,
                                            proof_handle: Handle<DisclosedProofs>,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_disclosed_proof_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_get_state(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move || {
        match proof_handle.get_state() {
            Ok(s) => {
                trace!("vcx_disclosed_proof_get_state_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), s);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_disclosed_proof_get_state_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, e, 0);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Checks for any state change in the disclosed proof and updates the state attribute
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Credential handle that was provided during creation. Used to identify disclosed proof object
///
/// cb: Callback that provides most current state of the disclosed proof and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_update_state(command_handle: CommandHandle,
                                               proof_handle: Handle<DisclosedProofs>,
                                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_disclosed_proof_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_update_state(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move || {
        match proof_handle.update_state(None) {
            Ok(s) => {
                trace!("vcx_disclosed_proof_update_state_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), s);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_disclosed_proof_update_state_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, e, 0);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Checks for any state change from the given message and updates the state attribute
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Credential handle that was provided during creation. Used to identify disclosed proof object
///
/// message: message to process for state changes
///
/// cb: Callback that provides most current state of the disclosed proof and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_update_state_with_message(command_handle: CommandHandle,
                                                            proof_handle: Handle<DisclosedProofs>,
                                                            message: *const c_char,
                                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_disclosed_proof_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_update_state_with_message(command_handle: {}, proof_handle: {}, message: {})",
           command_handle, proof_handle, secret!(message));

    spawn(move || {
        match proof_handle.update_state(Some(message)) {
            Ok(s) => {
                trace!("vcx_disclosed_proof_update_state__with_message_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), s);
                cb(command_handle, error::SUCCESS.code_num, s)
            }
            Err(e) => {
                error!("vcx_disclosed_proof_update_state_with_message_cb(command_handle: {}, rc: {}, state: {})",
                       command_handle, e, 0);
                cb(command_handle, e.into(), 0)
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes the disclosed proof object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// cb: Callback that provides json string of the disclosed proof's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_serialize(command_handle: CommandHandle,
                                            proof_handle: Handle<DisclosedProofs>,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_serialize(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move || {
        match proof_handle.to_string() {
            Ok(x) => {
                trace!("vcx_disclosed_proof_serialize_cb(command_handle: {}, rc: {}, data: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_serialize_cb(command_handle: {}, rc: {}, data: {})",
                       command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing an disclosed proof object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// data: json string representing a disclosed proof object
///
///
/// cb: Callback that provides handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_deserialize(command_handle: CommandHandle,
                                              proof_data: *const c_char,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, handle: Handle<DisclosedProofs>)>) -> u32 {
    info!("vcx_disclosed_proof_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(proof_data, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_deserialize(command_handle: {}, proof_data: {})",
           command_handle, secret!(proof_data));

    spawn(move || {
        match disclosed_proof::from_string(&proof_data) {
            Ok(x) => {
                trace!("vcx_disclosed_proof_deserialize_cb(command_handle: {}, rc: {}, proof_handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);

                cb(command_handle, 0, x);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_deserialize_cb(command_handle: {}, rc: {}, proof_handle: {})",
                       command_handle, x, 0);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get credentials from wallet matching to the proof request associated with proof object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// cb: Callback that provides json string of the credentials in wallet associated with proof request
///
/// # Example
/// credentials -> "{'attrs': {'attribute_0': [{'cred_info': {'schema_id': 'id', 'cred_def_id': 'id', 'attrs': {'attr_name': 'attr_value', ...}, 'referent': '914c7e11'}}]}}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_retrieve_credentials(command_handle: CommandHandle,
                                                       proof_handle: Handle<DisclosedProofs>,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, data: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_retrieve_credentials >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_retrieve_credentials(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move || {
        match proof_handle.retrieve_credentials() {
            Ok(x) => {
                trace!("vcx_disclosed_proof_retrieve_credentials(command_handle: {}, rc: {}, data: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_retrieve_credentials(command_handle: {}, rc: {}, data: {})",
                       command_handle, x, 0);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Accept proof request associated with proof object and generates a proof from the selected credentials and self attested attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
///
/// handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// selected_credentials: a json string with a credential for each proof request attribute.
///     List of possible credentials for each attribute is returned from vcx_disclosed_proof_retrieve_credentials,
///         (user needs to select specific credential to use from list of credentials)
///         {
///             "attrs":{
///                 String:{// Attribute key: This may not be the same as the attr name ex. "age_1" where attribute name is "age"
///                     "credential": {
///                         "cred_info":{
///                             "referent":String,
///                             "attrs":{ String: String }, // ex. {"age": "111", "name": "Bob"}
///                             "schema_id": String,
///                             "cred_def_id": String,
///                             "rev_reg_id":Option<String>,
///                             "cred_rev_id":Option<String>,
///                             },
///                         "interval":Option<{to: Option<u64>, from:: Option<u64>}>
///                     }, // This is the exact credential information selected from list of
///                        // credentials returned from vcx_disclosed_proof_retrieve_credentials
///                     "tails_file": Option<"String">, // Path to tails file for this credential
///                 },
///            },
///           "predicates":{ TODO: will be implemented as part of IS-1095 ticket. }
///        }
///     // selected_credentials can be empty "{}" if the proof only contains self_attested_attrs
///
/// self_attested_attrs: a json string with attributes self attested by user
/// # Examples
/// self_attested_attrs -> "{"self_attested_attr_0":"attested_val"}" | "{}"
/// selected_credentials -> "{'attrs': {'attribute_0': {'credential': {'cred_info': {'cred_def_id': 'od', 'schema_id': 'id', 'referent': '0c212108-9433-4199-a21f-336a44164f38', 'attrs': {'attr_name': 'attr_value', ...}}}}}}"
/// cb: Callback that returns error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_generate_proof(command_handle: CommandHandle,
                                                 proof_handle: Handle<DisclosedProofs>,
                                                 selected_credentials: *const c_char,
                                                 self_attested_attrs: *const c_char,
                                                 cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_generate_proof >>>");

    check_useful_c_str!(selected_credentials, VcxErrorKind::InvalidOption);
    check_useful_c_str!(self_attested_attrs, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_generate_proof(command_handle: {}, proof_handle: {}, selected_credentials: {}, self_attested_attrs: {})",
           command_handle, proof_handle, json!(selected_credentials), json!(self_attested_attrs));

    spawn(move || {
        match proof_handle.generate_proof(selected_credentials, self_attested_attrs) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_generate_proof(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_generate_proof(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Declines presentation request.
/// There are two ways of following interaction:
///     - Prover wants to propose using a different presentation - pass `proposal` parameter.
///     - Prover doesn't want to continue interaction - pass `reason` parameter.
/// Note that only one of these parameters can be passed.
///
/// Note that proposing of different presentation is supported for `aries` protocol only.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to identify the disclosed proof object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// reason: (Optional) human-readable string that explain the reason of decline
///
/// proposal: (Optional) the proposed format of presentation request
/// (see https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof#presentation-preview for details)
/// {
///    "attributes": [
///        {
///            "name": "<attribute_name>",
///            "cred_def_id": Optional("<cred_def_id>"),
///            "mime-type": Optional("<type>"),
///            "value": Optional("<value>")
///        },
///        // more attributes
///    ],
///    "predicates": [
///        {
///            "name": "<attribute_name>",
///            "cred_def_id": Optional("<cred_def_id>"),
///            "predicate": "<predicate>", - one of "<", "<=", ">=", ">"
///            "threshold": <threshold>
///        },
///        // more predicates
///    ]
/// }
///
/// # Example
///  proposal ->
///     {
///          "attributes": [
///              {
///                  "name": "first name"
///              }
///          ],
///          "predicates": [
///              {
///                  "name": "age",
///                  "predicate": ">",
///                  "threshold": 18
///              }
///          ]
///      }
///
/// cb: Callback that returns error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_decline_presentation_request(command_handle: u32,
                                                               proof_handle: Handle<DisclosedProofs>,
                                                               connection_handle: Handle<Connections>,
                                                               reason: *const c_char,
                                                               proposal: *const c_char,
                                                               cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_disclosed_proof_decline_presentation_request >>>");

    check_useful_opt_c_str!(reason, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(proposal, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_decline_presentation_request(command_handle: {}, proof_handle: {}, connection_handle: {}, reason: {:?}, proposal: {:?})",
           command_handle, proof_handle, connection_handle, secret!(reason), secret!(proposal));

    spawn(move || {
        match proof_handle.decline_presentation_request(connection_handle, reason, proposal) {
            Ok(_) => {
                trace!("vcx_disclosed_proof_decline_presentation_request(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(x) => {
                error!("vcx_disclosed_proof_decline_presentation_request(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get Problem Report message for Disclosed Proof object in Failed or Rejected state.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: handle pointing to Disclosed Proof state object.
///
/// cb: Callback that returns Problem Report as JSON string or null
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_get_problem_report(command_handle: CommandHandle,
                                                     proof_handle: Handle<DisclosedProofs>,
                                                     cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                          err: u32,
                                                                          message: *const c_char)>) -> u32 {
    info!("vcx_disclosed_proof_get_problem_report >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_disclosed_proof_get_problem_report(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move || {
        match proof_handle.get_problem_report_message() {
            Ok(message) => {
                trace!("vcx_disclosed_proof_get_problem_report_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(message));
                let message = CStringUtils::string_to_cstring(message);
                cb(command_handle, error::SUCCESS.code_num, message.as_ptr());
            }
            Err(x) => {
                error!("vcx_disclosed_proof_get_problem_report_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the disclosed proof object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_disclosed_proof_release(handle: Handle<DisclosedProofs>) -> u32 {
    info!("vcx_disclosed_proof_release >>>");

    spawn(move || {
        match handle.release() {
            Ok(()) => {
                trace!("vcx_disclosed_proof_release(handle: {}, rc: {})",
                       handle, error::SUCCESS.as_str());
            }
            Err(_e) => {
                // FIXME logging here results in panic while python tests
                // warn!("vcx_disclosed_proof_release(handle: {}), rc: {})",
                //       handle, e);
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
    use crate::utils::constants::PENDING_OBJECT_SERIALIZE_VERSION;
    use crate::api::return_types;
    use serde_json::Value;
    use crate::utils::devsetup::*;
    use crate::utils::httpclient::AgencyMock;

    pub const BAD_PROOF_REQUEST: &str = r#"{"version": "0.1","to_did": "LtMgSjtFcyPwenK9SHCyb8","from_did": "LtMgSjtFcyPwenK9SHCyb8","claim": {"account_num": ["8BEaoLf8TBmK4BUyX8WWnA"],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "Pd4fnFtRBcMKRVC2go5w3j","claim_name": "Account Certificate","claim_id": "3675417066","msg_ref_id": "ymy5nth"}"#;

    fn _vcx_disclosed_proof_create_with_request_c_closure(proof_request: &str) -> Result<Handle<DisclosedProofs>, u32> {
        let (h, cb, r) = return_types::return_u32_dph();
        let proof_req_cstr = CString::new(proof_request).unwrap();
        let rc = vcx_disclosed_proof_create_with_request(h,
                                                         "test_create\0".as_ptr().cast(),
                                                         proof_req_cstr.as_ptr(),
                                                         Some(cb));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        r.recv_medium()
    }

    const EMPTY_JSON: *const c_char = "{}\0".as_ptr().cast();

    #[test]
    fn test_vcx_proof_create_with_request_success() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_proof_create_with_request() {
        let _setup = SetupMocks::init();

        let err = _vcx_disclosed_proof_create_with_request_c_closure(BAD_PROOF_REQUEST).unwrap_err();
        assert_eq!(err, error::INVALID_PROOF_REQUEST.code_num);
    }

    #[test]
    fn test_create_with_msgid() {
        let _setup = SetupMocks::init();

        let cxn = crate::connection::tests::build_test_connection();

        AgencyMock::set_next_response(crate::utils::constants::NEW_PROOF_REQUEST_RESPONSE);

        let (h, cb, r) = return_types::return_u32_dph_str();
        assert_eq!(vcx_disclosed_proof_create_with_msgid(h,
                                                         "test_create_with_msgid\0".as_ptr().cast(),
                                                         cxn,
                                                         "123\0".as_ptr().cast(),
                                                         Some(cb)), error::SUCCESS.code_num);
        let (handle, disclosed_proof) = r.recv_medium().unwrap();
        assert!(handle > 0 && disclosed_proof.is_some());
    }

    #[test]
    fn test_vcx_disclosed_proof_release() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(vcx_disclosed_proof_release(handle), error::SUCCESS.code_num);
    }

    #[test]
    fn test_vcx_disclosed_proof_serialize_and_deserialize() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_disclosed_proof_serialize(h,
                                                 handle,
                                                 Some(cb)), error::SUCCESS.code_num);
        let s = r.recv_short().unwrap().unwrap();

        let j: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(j["version"], PENDING_OBJECT_SERIALIZE_VERSION);

        let (h, cb, r) = return_types::return_u32_dph();
        let cstr = CString::new(s).unwrap();
        assert_eq!(vcx_disclosed_proof_deserialize(h,
                                                   cstr.as_ptr(),
                                                   Some(cb)),
                   error::SUCCESS.code_num);

        let handle = r.recv_short().unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_generate_msg() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_disclosed_proof_get_proof_msg(h,
                                                     handle,
                                                     Some(cb)), error::SUCCESS.code_num);
        let _s = r.recv_short().unwrap().unwrap();
    }

    #[test]
    fn test_vcx_send_proof() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(handle.get_state().unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let connection_handle = connection::tests::build_test_connection();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_disclosed_proof_send_proof(h, handle, connection_handle, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_reject_proof_request() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(handle.get_state().unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let connection_handle = connection::tests::build_test_connection();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_disclosed_proof_reject_proof(h, handle, connection_handle, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_get_reject_msg() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(handle.get_state().unwrap(), VcxStateType::VcxStateRequestReceived as u32);

        let _connection_handle = connection::tests::build_test_connection();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_disclosed_proof_get_reject_msg(h, handle, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_proof_get_requests() {
        let _setup = SetupMocks::init();

        let cxn = crate::connection::tests::build_test_connection();

        AgencyMock::set_next_response(crate::utils::constants::NEW_PROOF_REQUEST_RESPONSE);

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_disclosed_proof_get_requests(h, cxn, Some(cb)), error::SUCCESS.code_num as u32);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_proof_get_state() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let (h, cb, r) = return_types::return_u32_u32();
        assert_eq!(vcx_disclosed_proof_get_state(h, handle, Some(cb)), error::SUCCESS.code_num);
        let state = r.recv_medium().unwrap();
        assert_eq!(state, VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_vcx_disclosed_proof_retrieve_credentials() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_disclosed_proof_retrieve_credentials(h,
                                                            handle,
                                                            Some(cb)),
                   error::SUCCESS.code_num);
        let _credentials = r.recv().unwrap().unwrap();
    }

    #[test]
    fn test_vcx_disclosed_proof_generate_proof() {
        let _setup = SetupMocks::init();

        let handle = _vcx_disclosed_proof_create_with_request_c_closure(crate::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_disclosed_proof_generate_proof(h,
                                                      handle,
                                                      EMPTY_JSON,
                                                      EMPTY_JSON,
                                                      Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }
}
