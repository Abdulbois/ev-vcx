use libc::c_char;
use crate::utils::cstring::CStringUtils;
use crate::utils::error;
use crate::utils::object_cache::Handle;
use crate::proof::Proofs;
use crate::proof;
use std::ptr;
use crate::utils::threadpool::spawn;
use crate::error::prelude::*;
use vdrtools_sys::CommandHandle;

use crate::connection::Connections;

/*
    APIs in this module are called by a verifier throughout the request-proof-and-verify process.
    Assumes that pairwise connection between Verifier and Prover is already established.

    # State

    The set of object states, agent and transitions depends on the communication method is used.
    There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
    The communication method can be specified as a config option on one of *_init functions.

    proprietary:
        VcxStateType::VcxStateInitialized - once `vcx_proof_create` (create Proof object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `PROOF_REQ` message) is called.

        VcxStateType::VcxStateAccepted - once `PROOF` agent is received.
                                         use `vcx_proof_update_state` or `vcx_proof_update_state_with_message` functions for state updates.

    aries:
        VcxStateType::VcxStateInitialized - once `vcx_proof_create` (create Proof object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `PresentationRequest` message) is called.

        VcxStateType::VcxStateAccepted - once `Presentation` agent is received.
        VcxStateType::VcxStateRejected - once `ProblemReport` agent is received.
        VcxStateType::None - once `PresentationProposal` agent is received.
        VcxStateType::None - on `Presentation` validation failed.
                                                use `vcx_proof_update_state` or `vcx_proof_update_state_with_message` functions for state updates.

    # Transitions

    proprietary:
        VcxStateType::None - `vcx_proof_create` - VcxStateType::VcxStateInitialized

        VcxStateType::VcxStateInitialized - `vcx_credential_send_request` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `PROOF` - VcxStateType::VcxStateAccepted

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        VcxStateType::None - `vcx_proof_create` - VcxStateType::VcxStateInitialized

        VcxStateType::VcxStateInitialized - `vcx_credential_send_request` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `Presentation` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `PresentationProposal` - VcxStateType::None
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::VcxStateRejected

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

/// Create a new Proof object that requests a proof for an enterprise
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Enterprise's personal identification for the proof, should be unique..
///
/// requested_attrs: Describes requested attribute
///     [{
///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
///                                              // NOTE: should either be "name" or "names", not both and not none of them.
///                                              // Use "names" to specify several attributes that have to match a single credential.
///         "restrictions":  Optional<wql query> - set of restrictions applying to requested credentials. (see below)
///         "non_revoked": {
///             "from": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
///             "to": Optional<(u64)>
///                 //Requested time represented as a total number of seconds from Unix Epoch, Optional
///         }
///     }]
///
/// # Example requested_attrs -> "[{"name":"attrName","restrictions":["issuer_did":"did","schema_id":"id","schema_issuer_did":"did","schema_name":"name","schema_version":"1.1.1","cred_def_id":"id"}]]"
///
/// requested_predicates: predicate specifications prover must provide claim for
///          [{ // set of requested predicates
///             "name": attribute name, (case insensitive and ignore spaces)
///             "p_type": predicate type (">=", ">", "<=", "<")
///             "p_value": int predicate value
///             "restrictions":  Optional<wql query> -  set of restrictions applying to requested credentials. (see below)
///             "non_revoked": Optional<{
///                 "from": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
///                 "to": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
///             }>
///          }]
///
/// # Example requested_predicates -> "[{"name":"attrName","p_type":"GE","p_value":9,"restrictions":["issuer_did":"did","schema_id":"id","schema_issuer_did":"did","schema_name":"name","schema_version":"1.1.1","cred_def_id":"id"}]]"
///
/// revocation_interval:  Optional<<revocation_interval>>, // see below,
///                        // If specified, prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (can be overridden on attribute level)
///     from: Optional<u64> // timestamp of interval beginning
///     to: Optional<u64> // timestamp of interval beginning
///         // Requested time represented as a total number of seconds from Unix Epoch, Optional
/// # Examples config ->  "{}" | "{"to": 123} | "{"from": 100, "to": 123}"
///
/// wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
///     The list of allowed keys that can be combine into complex queries.
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///         // the following keys can be used for every `attribute name` in credential.
///         "attr::<attribute name>::marker": "1", - to filter based on existence of a specific attribute
///         "attr::<attribute name>::value": <attribute raw value>, - to filter based on value of a specific attribute
///
/// cb: Callback that provides proof handle and error status of request.
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_create(command_handle: CommandHandle,
                               source_id: *const c_char,
                               requested_attrs: *const c_char,
                               requested_predicates: *const c_char,
                               revocation_interval: *const c_char,
                               name: *const c_char,
                               cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_handle: Handle<Proofs>)>) -> u32 {
    info!("vcx_proof_create >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(requested_attrs, VcxErrorKind::InvalidOption);
    check_useful_c_str!(requested_predicates, VcxErrorKind::InvalidOption);
    check_useful_c_str!(name, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(revocation_interval, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_create(command_handle: {}, source_id: {}, requested_attrs: {}, requested_predicates: {}, revocation_interval: {}, name: {})",
           command_handle, source_id, secret!(requested_attrs), secret!(requested_predicates), secret!(revocation_interval), secret!(name));

    spawn(move || {
        let (rc, handle) = match proof::create_proof(source_id.clone(), requested_attrs, requested_predicates, revocation_interval, name) {
            Ok(x) => {
                trace!("vcx_proof_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), x, source_id);
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_proof_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                (x.into(), Handle::dummy())
            }
        };
        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a new Proof object based on the given Presentation Proposal message
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: Enterprise's personal identification for the proof, should be unique..
///
/// presentation_proposal: Message sent by the Prover to the verifier to initiate a proof presentation process:
///     {
///         "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/propose-presentation",
///         "@id": "<uuid-propose-presentation>",
///         "comment": "some comment",
///         "presentation_proposal": {
///             "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation-preview",
///             "attributes": [
///                 {
///                     "name": "<attribute_name>", - name of the attribute.
///                     "cred_def_id": "<cred_def_id>", - maps to the credential definition identifier of the credential with the current attribute
///                     "mime-type": Optional"<type>", - optional type of value. if mime-type is missing (null), then value is a string.
///                     "value": "<value>", - value of the attribute to reveal in presentation
///                 },
///                 // more attributes
///               ],
///              "predicates": [
///                 {
///                     "name": "<attribute_name>", - name of the attribute.
///                     "cred_def_id": "<cred_def_id>", - maps to the credential definition identifier of the credential with the current attribute
///                     "predicate": "<predicate>", - predicate operator: "<", "<=", ">=", ">"
///                     "threshold": <threshold> - threshold value for the predicate.
///                 },
///                 // more predicates
///             ]
///         }
///     }
///
/// cb: Callback that provides proof handle and error status of request.
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_create_with_proposal(command_handle: CommandHandle,
                                             source_id: *const c_char,
                                             presentation_proposal: *const c_char,
                                             name: *const c_char,
                                             cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_handle: Handle<Proofs>)>) -> u32 {
    info!("vcx_proof_create_with_proposal >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(name, VcxErrorKind::InvalidOption);
    check_useful_c_str!(presentation_proposal, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_create_with_proposal(command_handle: {}, source_id: {}, name: {}, presentation_proposal: {})",
           command_handle, source_id, secret!(name), secret!(presentation_proposal));

    spawn(move || {
        let (rc, handle) = match proof::create_proof_with_proposal(source_id.clone(), name, presentation_proposal) {
            Ok(x) => {
                trace!("vcx_proof_create_with_proposal_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), x, source_id);
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_proof_create_with_proposal_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                (x.into(), Handle::dummy())
            }
        };
        cb(command_handle, rc, handle);

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
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// cb: Callback that provides most current state of the proof and error status of request
///     States:
///         1 - Initialized
///         2 - Request Sent
///         3 - Proof Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_update_state(command_handle: CommandHandle,
                                     proof_handle: Handle<Proofs>,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_proof_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_update_state(command_handle: {}, proof_handle: {})",
          command_handle, proof_handle);

    spawn(move|| {
        match proof_handle.update_state(None) {
            Ok(state) => {
                trace!("vcx_proof_update_state_cb(command_handle: {}, rc: {}, proof_handle: {}, state: {})",
                      command_handle, error::SUCCESS.as_str(), proof_handle, state);
                cb(command_handle, error::SUCCESS.code_num, state);
            },
            Err(x) => {
                warn!("vcx_proof_update_state_cb(command_handle: {}, rc: {}, proof_handle: {}, state: {})",
                      command_handle, x, proof_handle, 0);
                cb(command_handle, x.into(), 0);
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Update the state of the proof based on the given message.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// message: message to process for state changes
///
/// cb: Callback that provides most current state of the proof and error status of request
///     States:
///         1 - Initialized
///         2 - Request Sent
///         3 - Proof Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_update_state_with_message(command_handle: CommandHandle,
                                                  proof_handle: Handle<Proofs>,
                                                  message: *const c_char,
                                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_proof_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_update_state_with_message(command_handle: {}, proof_handle: {})",
          command_handle, proof_handle);

    spawn(move|| {
        match proof_handle.update_state(Some(message)) {
            Ok(x) => {
                trace!("vcx_proof_update_state_with_message_cb(command_handle: {}, rc: {}, proof_handle: {}, state: {})",
                      command_handle, error::SUCCESS.as_str(), proof_handle, x);
                cb(command_handle, error::SUCCESS.code_num, x);
            },
            Err(x) => {
                warn!("vcx_proof_update_state_with_message_cb(command_handle: {}, rc: {}, proof_handle: {}, state: {})",
                      command_handle, x, proof_handle, 0);
                cb(command_handle, x.into(), 0);
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the current state of the proof object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// cb: Callback that provides most current state of the proof and error status of request
///     States:
///         1 - Initialized
///         2 - Request Sent
///         3 - Proof Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_get_state(command_handle: CommandHandle,
                                  proof_handle: Handle<Proofs>,
                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_proof_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_get_state(command_handle: {}, proof_handle: {})", command_handle, proof_handle);

    spawn(move|| {
        match proof_handle.get_state() {
            Ok(x) => {
                trace!("vcx_proof_get_state_cb(command_handle: {}, rc: {}, proof_handle: {}, state: {})",
                      command_handle, error::SUCCESS.as_str(), proof_handle, x);
                cb(command_handle, error::SUCCESS.code_num, x);
            },
            Err(x) => {
                warn!("vcx_proof_get_state_cb(command_handle: {}, rc: {}, proof_handle: {}, state: {})",
                      command_handle, x, proof_handle, 0);
                cb(command_handle, x.into(), 0);
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes the proof object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// cb: Callback that provides json string of the proof's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_serialize(command_handle: CommandHandle,
                                  proof_handle: Handle<Proofs>,
                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_state: *const c_char)>) -> u32 {
    info!("vcx_proof_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_serialize(command_handle: {}, proof_handle: {})", command_handle, proof_handle);

    spawn(move|| {
        match proof_handle.to_string() {
            Ok(x) => {
                trace!("vcx_proof_serialize_cb(command_handle: {}, proof_handle: {}, rc: {}, state: {})",
                      command_handle, proof_handle, error::SUCCESS.as_str(), secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_proof_serialize_cb(command_handle: {}, proof_handle: {}, rc: {}, state: {})",
                      command_handle, proof_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            },
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing a proof object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_data: json string representing a proof object
///
/// cb: Callback that provides proof handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_deserialize(command_handle: CommandHandle,
                                    proof_data: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_handle: Handle<Proofs>)>) -> u32 {
    info!("vcx_proof_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(proof_data, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_deserialize(command_handle: {}, proof_data: {})",
          command_handle, secret!(proof_data));

    spawn(move|| {
        let (rc, handle) = match proof::from_string(&proof_data) {
            Ok(x) => {
                trace!("vcx_proof_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, error::SUCCESS.as_str(), x);
                (error::SUCCESS.code_num, x)
            },
            Err(x) => {
                warn!("vcx_proof_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                      command_handle, x, 0);
                (x.into(), Handle::dummy())
            },
        };
        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the proof object by de-allocating memory
///
/// #Params
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_proof_release(proof_handle: Handle<Proofs>) -> u32 {
    info!("vcx_proof_release >>>");

    spawn(move || {
        match proof_handle.release() {
            Ok(()) => {
                trace!("vcx_proof_release(proof_handle: {}, rc: {})",
                       proof_handle, error::SUCCESS.as_str());
            }
            Err(_e) => {
                // FIXME logging here results in panic while python tests
                // warn!("vcx_proof_release(proof_handle: {}), rc: {})",
                //       proof_handle, e);
            }
        };
        Ok(())
    });
    error::SUCCESS.code_num
}

/// Sends a proof request to pairwise connection
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: provides any error status of the proof_request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_send_request(command_handle: CommandHandle,
                                     proof_handle: Handle<Proofs>,
                                     connection_handle: Handle<Connections>,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_proof_send_request >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_send_request(command_handle: {}, proof_handle: {}, connection_handle: {})",
          command_handle, proof_handle, connection_handle);

    spawn(move|| {
        let err = match proof_handle.send_proof_request(connection_handle) {
            Ok(x) => {
                trace!("vcx_proof_send_request_cb(command_handle: {}, rc: {}, proof_handle: {})",
                      command_handle, 0, proof_handle);
                x
            },
            Err(x) => {
                warn!("vcx_proof_send_request_cb(command_handle: {}, rc: {}, proof_handle: {})",
                      command_handle, x, proof_handle);
                x.into()
            },
        };

        cb(command_handle,err);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Request a new proof after receiving a proof proposal (this enables negotiation)
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// requested_attrs: Describes requested attribute
///     [{
///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
///                                              // NOTE: should either be "name" or "names", not both and not none of them.
///                                              // Use "names" to specify several attributes that have to match a single credential.
///         "restrictions":  Optional<wql query> - set of restrictions applying to requested credentials. (see below)
///         "non_revoked": {
///             "from": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
///             "to": Optional<(u64)>
///                 //Requested time represented as a total number of seconds from Unix Epoch, Optional
///         }
///     }]
///
/// # Example requested_attrs -> "[{"name":"attrName","restrictions":["issuer_did":"did","schema_id":"id","schema_issuer_did":"did","schema_name":"name","schema_version":"1.1.1","cred_def_id":"id"}]]"
///
/// requested_predicates: predicate specifications prover must provide claim for
///          [{ // set of requested predicates
///             "name": attribute name, (case insensitive and ignore spaces)
///             "p_type": predicate type (">=", ">", "<=", "<")
///             "p_value": int predicate value
///             "restrictions":  Optional<wql query> -  set of restrictions applying to requested credentials. (see below)
///             "non_revoked": Optional<{
///                 "from": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
///                 "to": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
///             }>
///          }]
///
/// # Example requested_predicates -> "[{"name":"attrName","p_type":"GE","p_value":9,"restrictions":["issuer_did":"did","schema_id":"id","schema_issuer_did":"did","schema_name":"name","schema_version":"1.1.1","cred_def_id":"id"}]]"
///
/// revocation_interval:  Optional<<revocation_interval>>, // see below,
///                        // If specified, prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (can be overridden on attribute level)
///     from: Optional<u64> // timestamp of interval beginning
///     to: Optional<u64> // timestamp of interval beginning
///         // Requested time represented as a total number of seconds from Unix Epoch, Optional
/// # Examples config ->  "{}" | "{"to": 123} | "{"from": 100, "to": 123}"
///
/// wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
///     The list of allowed keys that can be combine into complex queries.
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///         // the following keys can be used for every `attribute name` in credential.
///         "attr::<attribute name>::marker": "1", - to filter based on existence of a specific attribute
///         "attr::<attribute name>::value": <attribute raw value>, - to filter based on value of a specific attribute
///
/// cb: Callback that provides proof handle and error status of request.
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_request_proof(command_handle: CommandHandle,
                                      proof_handle: Handle<Proofs>,
                                      connection_handle: Handle<Connections>,
                                      requested_attrs: *const c_char,
                                      requested_predicates: *const c_char,
                                      revocation_interval: *const c_char,
                                      name: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_proof_request_proof >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(requested_attrs, VcxErrorKind::InvalidOption);
    check_useful_c_str!(requested_predicates, VcxErrorKind::InvalidOption);
    check_useful_c_str!(revocation_interval, VcxErrorKind::InvalidOption);
    check_useful_c_str!(name, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_request_proof(command_handle: {}, proof_handle: {}, connection_handle: {}, requested_attrs: {}, requested_predicates: {}, revocation_interval: {}, name: {})",
          command_handle, proof_handle, connection_handle, secret!(requested_attrs), secret!(requested_predicates), secret!(revocation_interval), secret!(name));

    spawn(move|| {
        let err = match proof_handle.request_proof(connection_handle, requested_attrs, requested_predicates, revocation_interval, name) {
            Ok(x) => {
                trace!("vcx_proof_request_proof_cb(command_handle: {}, rc: {}, proof_handle: {})",
                      command_handle, 0, proof_handle);
                x
            },
            Err(x) => {
                warn!("vcx_proof_request_proof_cb(command_handle: {}, rc: {}, proof_handle: {})",
                      command_handle, x, proof_handle);
                x.into()
            },
        };

        cb(command_handle,err);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the proof request message that can be sent to the specified connection
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// cb: provides any error status of the proof_request
///
/// # Example proof_request -> "{'@topic': {'tid': 0, 'mid': 0}, '@type': {'version': '1.0', 'name': 'PROOF_REQUEST'}, 'proof_request_data': {'name': 'proof_req', 'nonce': '118065925949165739229152', 'version': '0.1', 'requested_predicates': {}, 'non_revoked': None, 'requested_attributes': {'attribute_0': {'name': 'name', 'restrictions': {'$or': [{'issuer_did': 'did'}]}}}, 'ver': '1.0'}, 'thread_id': '40bdb5b2'}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_get_request_msg(command_handle: CommandHandle,
                                        proof_handle: Handle<Proofs>,
                                        cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_proof_get_request_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_get_request_msg(command_handle: {}, proof_handle: {})",
          command_handle, proof_handle);

    spawn(move|| {
        match proof_handle.generate_proof_request_msg() {
            Ok(msg) => {
                trace!("vcx_proof_get_request_msg_cb(command_handle: {}, rc: {}, proof_handle: {}, msg: {})",
                       command_handle, error::SUCCESS.code_num, proof_handle, secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_proof_get_request_msg_cb(command_handle: {}, rc: {}, proof_handle: {})",
                      command_handle, x, proof_handle);
                cb(command_handle, x.into(), ptr::null_mut())
            },
        };


        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the proof request attachment that you send along the out of band credential
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// cb: provides any error status of the proof_request
///
/// # Example presentation_request_attachment -> "{"@id": "8b23c2b6-b432-45d8-a377-d003950c0fcc", "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/request-presentation", "comment": "Person Proving", "request_presentations~attach": [{"@id": "libindy-request-presentation-0", "data": {"base64": "eyJuYW1lIjoiUGVyc29uIFByb3ZpbmciLCJub25fcmV2b2tlZCI6bnVsbCwibm9uY2UiOiI2MzQxNzYyOTk0NjI5NTQ5MzA4MjY1MzQiLCJyZXF1ZXN0ZWRfYXR0cmlidXRlcyI6eyJhdHRyaWJ1dGVfMCI6eyJuYW1lIjoibmFtZSJ9LCJhdHRyaWJ1dGVfMSI6eyJuYW1lIjoiZW1haWwifX0sInJlcXVlc3RlZF9wcmVkaWNhdGVzIjp7fSwidmVyIjpudWxsLCJ2ZXJzaW9uIjoiMS4wIn0="}, "mime-type": "application/json"}]}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_get_request_attach(command_handle: CommandHandle,
                                           proof_handle: Handle<Proofs>,
                                           cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg: *const c_char)>) -> u32 {
    info!("vcx_proof_get_request_attach >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_get_request_attach(command_handle: {}, proof_handle: {})", command_handle, proof_handle);

    spawn(move || {
        match proof_handle.generate_request_attach() {
            Ok(x) => {
                trace!("vcx_proof_get_request_msg_cb(command_handle: {}, rc: {}, proof_handle: {}, request_attach: {})",
                       command_handle, error::SUCCESS.code_num, proof_handle, secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());

            },
            Err(x) => {
                warn!("vcx_proof_get_request_msg_cb(command_handle: {}, rc: {}, proof_handle: {})",
                      command_handle, x, proof_handle);
                cb(command_handle, x.into(), ptr::null_mut())
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the proof proposal received for deciding whether to accept it
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to access proof object
///
/// cb: provides any error status of the proof_request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_get_proof_proposal(command_handle: CommandHandle,
                                     proof_handle: Handle<Proofs>,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proposal: *const c_char)>) -> u32 {
    info!("vcx_get_proof_proposal >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_get_proof_proposal(command_handle: {}, proof_handle: {})",
          command_handle, proof_handle);

    spawn(move|| {
        match proof_handle.get_presentation_proposal_request() {
            Ok(msg) => {
                trace!("vcx_get_proof_proposal_cb(command_handle: {}, rc: {}, proof_handle: {}, msg: {})",
                       command_handle, error::SUCCESS.code_num, proof_handle, secret!(msg));
                let msg = CStringUtils::string_to_cstring(msg);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_get_proof_proposal_cb(command_handle: {}, rc: {}, proof_handle: {})",
                      command_handle, x, proof_handle);
                cb(command_handle, x.into(), ptr::null_mut())
            },
        };


        Ok(())
    });

    error::SUCCESS.code_num
}

/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to identify proof object
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides Proof attributes and error status of sending the credential
///
/// #Returns
/// Error code as a u32
#[deprecated(
since = "1.15.0",
note = "Use vcx_get_proof_msg() instead. This api is similar, but requires an extra parameter (connection_handle) which is unnecessary and unused in the internals."
)]
#[no_mangle]
pub extern fn vcx_get_proof(command_handle: CommandHandle,
                            proof_handle: Handle<Proofs>,
                            _unused_connection_handle: Handle<Connections>,
                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_state:u32, response_data: *const c_char)>) -> u32 {
    info!("vcx_get_proof >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if let Some(err) = proof_to_cb(command_handle, proof_handle, cb).err() { return err.into() }

    error::SUCCESS.code_num
}

/// Get Proof Msg
///
/// *Note* This replaces vcx_get_proof. You no longer need a connection handle.
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: Proof handle that was provided during creation. Used to identify proof object
///
/// cb: Callback that provides Proof attributes and error status of sending the credential
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_get_proof_msg(command_handle: CommandHandle,
                                proof_handle: Handle<Proofs>,
                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, proof_state: u32, response_data: *const c_char)>) -> u32 {
    info!("vcx_get_proof_msg >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if let Some(err) = proof_to_cb(command_handle, proof_handle, cb).err() { return err.into() }

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_proof_set_connection(command_handle: CommandHandle,
                                       proof_handle: Handle<Proofs>,
                                       connection_handle: Handle<Connections>,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_proof_set_connection >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    if !proof_handle.is_valid_handle() {
        return VcxError::from(VcxErrorKind::InvalidIssuerCredentialHandle).into();
    }

    if !connection_handle.is_valid_handle() {
        return VcxError::from(VcxErrorKind::InvalidConnectionHandle).into();
    }

    trace!("vcx_proof_set_connection(command_handle: {}, proof_handle: {}, connection_handle: {})",
           command_handle, proof_handle, connection_handle);

    spawn(move || {
        let err = match proof_handle.set_connection(connection_handle) {
            Ok(x) => {
                trace!("vcx_proof_set_connection_cb(command_handle: {}, credential_handle: {}, rc: {})",
                       command_handle, proof_handle, error::SUCCESS.as_str());
                x
            }
            Err(x) => {
                warn!("vcx_proof_set_connection_cb(command_handle: {}, credential_handle: {}, rc: {})",
                      command_handle, proof_handle, x);
                x.into()
            }
        };

        cb(command_handle, err);

        Ok(())
    });

    error::SUCCESS.code_num
}


fn proof_to_cb(command_handle: CommandHandle,
               proof_handle: Handle<Proofs>,
               cb: extern fn(xcommand_handle: CommandHandle, err: u32, proof_state: u32, response_data: *const c_char))
               -> VcxResult<()>{
    trace!("vcx_get_proof(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move|| {
        //update the state to see if proof has come, ignore any errors
        let _ = proof_handle.update_state(None);

        match proof_handle.get_proof() {
            Ok(x) => {
                trace!("vcx_get_proof_cb(command_handle: {}, proof_handle: {}, rc: {}, proof: {})",
                       command_handle, proof_handle, 0, secret!(x));
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, proof_handle.get_proof_state().unwrap_or(0), msg.as_ptr());
            },
            Err(x) => {
                warn!("vcx_get_proof_cb(command_handle: {}, proof_handle: {}, rc: {}, proof: {})", command_handle, proof_handle, x, "null");
                cb(command_handle, x.into(), proof_handle.get_proof_state().unwrap_or(0), ptr::null_mut());
            },
        };

        Ok(())
    });

    Ok(())
}

#[allow(unused_variables)]
pub extern fn vcx_proof_accepted(proof_handle: Handle<Proofs>, response_data: *const c_char) -> u32 {
    info!("vcx_proof_accepted >>>");
    error::SUCCESS.code_num
}

/// Get Problem Report message for Proof object in Failed or Rejected state.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// proof_handle: handle pointing to Proof state object.
///
/// cb: Callback that returns Problem Report as JSON string or null
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_proof_get_problem_report(command_handle: CommandHandle,
                                           proof_handle: Handle<Proofs>,
                                           cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                err: u32,
                                                                message: *const c_char)>) -> u32 {
    info!("vcx_proof_get_problem_report >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_proof_get_problem_report(command_handle: {}, proof_handle: {})",
           command_handle, proof_handle);

    spawn(move || {
        match proof_handle.get_problem_report_message() {
            Ok(message) => {
                trace!("vcx_proof_get_problem_report_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(message));
                let message = CStringUtils::string_to_cstring(message);
                cb(command_handle, error::SUCCESS.code_num, message.as_ptr());
            }
            Err(x) => {
                error!("vcx_proof_get_problem_report_cb(command_handle: {}, rc: {})",
                       command_handle, x);
                cb(command_handle, x.into(), ptr::null_mut());
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
    use std::ptr;
    use crate::proof;
    use crate::api::{ProofStateType, return_types, VcxStateType};
    use crate::utils::constants::*;
    use crate::utils::devsetup::*;
    use crate::connection::tests::build_test_connection;
    use crate::disclosed_proof;

    const REV_INT: *const c_char = concat!(r#"{"support_revocation":false}"#, "\0").as_ptr().cast();

    fn create_proof_util() -> Result<Handle<Proofs>, u32> {
        let (h, cb, r) = return_types::return_u32_ph();
        let requested_atrs = CString::new(REQUESTED_ATTRS).unwrap();
        let requested_preds = CString::new(REQUESTED_PREDICATES).unwrap();
        let rc = vcx_proof_create(h,
                                  "PROOF_NAME\0".as_ptr().cast(),
                                  requested_atrs.as_ptr(),
                                  requested_preds.as_ptr(),
                                  REV_INT,
                                  "optional\0".as_ptr().cast(),
                                  Some(cb));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        r.recv_medium()
    }

    #[test]
    fn test_vcx_create_proof_success() {
        let _setup = SetupMocks::init();

        let handle = create_proof_util().unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_proof_no_agency() {
        let _setup = SetupMocks::init();

        let ph = create_proof_util().unwrap();
        let request = ph.generate_proof_request_msg().unwrap();
        let dp = disclosed_proof::create_proof("test", &request).unwrap();
        let p = dp.generate_proof_msg().unwrap();
        ph.update_state(Some(p)).unwrap();
        assert_eq!(ph.get_state().unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    fn test_vcx_create_proof_fails() {
        let _setup = SetupMocks::init();

        let (h, _cb, _r) = return_types::return_u32_u32();
        assert_eq!(vcx_proof_create(h,
                                    ptr::null(),
                                    ptr::null(),
                                    ptr::null(),
                                    REV_INT,
                                    ptr::null(),
                                    None),
                   error::INVALID_OPTION.code_num);
    }

    #[test]
    fn test_vcx_proof_get_request_msg() {
        let _setup = SetupMocks::init();

        let proof_handle = create_proof_util().unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_proof_get_request_msg(h, proof_handle, Some(cb)),
                   error::SUCCESS.code_num);
        let _msg = r.recv_medium().unwrap().unwrap();
    }

    #[test]
    fn test_vcx_proof_serialize() {
        let _setup = SetupMocks::init();

        let proof_handle = create_proof_util().unwrap();

        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_proof_serialize(h,
                                       proof_handle,
                                       Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_vcx_proof_deserialize_succeeds() {
        let _setup = SetupMocks::init();
        let data =  CString::new(PROOF_WITH_INVALID_STATE).unwrap();
        let (h, cb, r) = return_types::return_u32_ph();
        assert_eq!(vcx_proof_deserialize(h,
                                         data.as_ptr().cast(),
                                         Some(cb)),
                   error::SUCCESS.code_num);
        let handle = r.recv_medium().unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_proof_update_state() {
        let _setup = SetupMocks::init();

        let proof_handle = create_proof_util().unwrap();

        let (h, cb, r) = return_types::return_u32_u32();
        assert_eq!(vcx_proof_update_state(h,
                                          proof_handle,
                                          Some(cb)),
                   error::SUCCESS.code_num);
        let state = r.recv_medium().unwrap();
        assert_eq!(state, VcxStateType::VcxStateInitialized as u32);
    }

    #[test]
    fn test_vcx_proof_send_request() {
        let _setup = SetupMocks::init();

        let proof_handle = create_proof_util().unwrap();

        assert_eq!(proof_handle.get_state().unwrap(), VcxStateType::VcxStateInitialized as u32);

        let connection_handle = build_test_connection();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_proof_send_request(h,
                                          proof_handle,
                                          connection_handle,
                                          Some(cb)),
                   error::SUCCESS.code_num);
        r.recv_medium().unwrap();

        assert_eq!(proof_handle.get_state().unwrap(), VcxStateType::VcxStateOfferSent as u32);

        let (h, cb, r) = return_types::return_u32_u32();
        let response = CString::new(PROOF_RESPONSE_STR).unwrap();
        assert_eq!(vcx_proof_update_state_with_message(h,
                                                       proof_handle,
                                                       response.as_ptr(),
                                                       Some(cb)),
                   error::SUCCESS.code_num);
        let _state = r.recv_medium().unwrap();

        assert_eq!(proof_handle.get_state().unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[allow(deprecated)]
    #[test]
    fn test_get_proof_fails_when_not_ready_with_proof() {
        let _setup = SetupMocks::init();

        let proof_handle = create_proof_util().unwrap();

        let (h, cb, r) = return_types::return_u32_u32_str();
        assert_eq!(vcx_get_proof(h,
                                 proof_handle,
                                 Handle::dummy(),
                                 Some(cb)),
                   error::SUCCESS.code_num);
        let _ = r.recv_medium().is_err();
    }

    #[allow(deprecated)]
    #[test]
    fn test_get_proof_returns_proof_with_proof_state_invalid() {
        let _setup = SetupMocks::init();

        let proof_handle = proof::from_string(PROOF_WITH_INVALID_STATE).unwrap();

        let (h, cb, r) = return_types::return_u32_u32_str();
        assert_eq!(vcx_get_proof(h,
                                 proof_handle,
                                 Handle::dummy(),
                                 Some(cb)),
                   error::SUCCESS.code_num);
        let (state, _) = r.recv_medium().unwrap();
        assert_eq!(state, ProofStateType::ProofInvalid as u32);

        vcx_proof_release(proof_handle);
    }

    #[test]
    fn test_vcx_connection_get_state() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_u32();
        let handle = proof::from_string(PROOF_OFFER_SENT).unwrap();

        let rc = vcx_proof_get_state(h, handle, Some(cb));
        assert_eq!(rc, error::SUCCESS.code_num);
        let state = r.recv_short().unwrap();
        assert_eq!(state, VcxStateType::VcxStateOfferSent as u32);
    }
}
