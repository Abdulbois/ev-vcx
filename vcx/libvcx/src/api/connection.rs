use crate::object_cache::Handle;
use crate::connection::Connections;
use libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use utils::threadpool::spawn;
use std::ptr;
use connection::*;
use error::prelude::*;
use indy_sys::CommandHandle;
use v3::messages::invite_action::invite::InviteActionData;

/*
    Tha API represents a pairwise connection with another identity owner.
    Once the connection, is established communication can happen securely and privately.
    Credentials and Presentations are exchanged using this object.

    # States

    The set of object states, messages and transitions depends on the communication method is used.
    There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
    The communication method can be specified as a config option on one of *_init functions.

    proprietary:
        Inviter:
            VcxStateType::VcxStateInitialized - once `vcx_connection_create` (create Connection object) is called.

            VcxStateType::VcxStateOfferSent - once `vcx_connection_connect` (send Connection invite) is called.

            VcxStateType::VcxStateAccepted - once `connReqAnswer` messages is received.
                                             use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.
            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called.

        Invitee:
            VcxStateType::VcxStateRequestReceived - once `vcx_connection_create_with_invite` (create Connection object with invite) is called.

            VcxStateType::VcxStateAccepted - once `vcx_connection_connect` (accept Connection invite) is called.

            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called.

    aries:
        Inviter:
            VcxStateType::VcxStateInitialized - 1) once `vcx_connection_create` (create Connection object) is called.
                                                2) once `vcx_connection_create_with_outofband_invitation` (create OutofbandConnection object) is called with `handshake:true`.

            VcxStateType::VcxStateOfferSent - once `vcx_connection_connect` (prepared Connection invite) is called.

            VcxStateType::VcxStateRequestReceived - once `ConnectionRequest` messages is received.
                                                    accept `ConnectionRequest` and send `ConnectionResponse` message.
                                                    use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.

            VcxStateType::VcxStateAccepted - 1) once `Ack` messages is received.
                                                use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.
                                             2) once `vcx_connection_connect` is called for Outoband Connection created with `handshake:false`.

            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called
                                            OR
                                        `ConnectionProblemReport` messages is received on state updates.

        Invitee:
            VcxStateType::VcxStateOfferSent - 1) once `vcx_connection_create_with_invite` (create Connection object with invite) is called.
                                              2) once `vcx_connection_create_with_outofband_invitation`
                                                 (create Connection object with Out-of-Band Invitation containing `handshake_protocols`) is called.

            VcxStateType::VcxStateRequestReceived - once `vcx_connection_connect` (accept `ConnectionInvite` and send `ConnectionRequest` message) is called.

            VcxStateType::VcxStateAccepted - 1) once `ConnectionResponse` messages is received.
                                                send `Ack` message if requested.
                                                use `vcx_connection_update_state` or `vcx_connection_update_state_with_message` functions for state updates.
                                             2) once `vcx_connection_create_with_outofband_invitation`
                                                (create one-time Connection object with Out-of-Band Invitation does not containing `handshake_protocols`) is called.

            VcxStateType::VcxStateNone - once `vcx_connection_delete_connection` (delete Connection object) is called
                                            OR
                                        `ConnectionProblemReport` messages is received on state updates.

    # Transitions

    proprietary:
        Inviter:
            VcxStateType::None - `vcx_connection_create` - VcxStateType::VcxStateInitialized
            VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateOfferSent
            VcxStateType::VcxStateOfferSent - received `connReqAnswer` - VcxStateType::VcxStateAccepted
            any state - `vcx_connection_delete_connection` - `VcxStateType::VcxStateNone`

        Invitee:
            VcxStateType::None - `vcx_connection_create_with_invite` - VcxStateType::VcxStateRequestReceived
            VcxStateType::VcxStateRequestReceived - `vcx_connection_connect` - VcxStateType::VcxStateAccepted
            any state - `vcx_connection_delete_connection` - `VcxStateType::VcxStateNone`

    aries - RFC: https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
        Inviter:
            VcxStateType::None - `vcx_connection_create` - VcxStateType::VcxStateInitialized
            VcxStateType::None - `vcx_connection_create_with_outofband_invitation` - VcxStateType::VcxStateInitialized

            VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateOfferSent
            VcxStateType::VcxStateInitialized - `vcx_connection_connect` - VcxStateType::VcxStateAccepted (Out-ob-Band Connection created with `handshake:false`)

            VcxStateType::VcxStateOfferSent - received `ConnectionRequest` - VcxStateType::VcxStateRequestReceived
            VcxStateType::VcxStateOfferSent - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateRequestReceived - received `Ack` - VcxStateType::VcxStateAccepted
            VcxStateType::VcxStateRequestReceived - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateAccepted - received `Ping`, `PingResponse`, `Query`, `Disclose` - VcxStateType::VcxStateAccepted

            any state - `vcx_connection_delete_connection` - VcxStateType::VcxStateNone


        Invitee:
            VcxStateType::None - `vcx_connection_create_with_invite` - VcxStateType::VcxStateOfferSent
            VcxStateType::None - `vcx_connection_create_with_outofband_invitation` (invite contains `handshake_protocols`) - VcxStateType::VcxStateOfferSent
            VcxStateType::None - `vcx_connection_create_with_outofband_invitation` (no `handshake_protocols`) - VcxStateType::VcxStateAccepted

            VcxStateType::VcxStateOfferSent - `vcx_connection_connect` - VcxStateType::VcxStateRequestReceived
            VcxStateType::VcxStateOfferSent - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateRequestReceived - received `ConnectionResponse` - VcxStateType::VcxStateAccepted
            VcxStateType::VcxStateRequestReceived - received `ConnectionProblemReport` - VcxStateType::VcxStateNone

            VcxStateType::VcxStateAccepted - received `Ping`, `PingResponse`, `Query`, `Disclose` - VcxStateType::VcxStateAccepted

            any state - `vcx_connection_delete_connection` - VcxStateType::VcxStateNone

    # Messages

    proprietary:
        ConnectionRequest (`connReq`)
        ConnectionRequestAnswer (`connReqAnswer`)

    aries:
        Invitation - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
        ConnectionRequest - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#1-connection-request
        ConnectionResponse - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#2-connection-response
        ConnectionProblemReport - https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#error-message-example
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
        Ping - https://github.com/hyperledger/aries-rfcs/tree/master/features/0048-trust-ping#messages
        PingResponse - https://github.com/hyperledger/aries-rfcs/tree/master/features/0048-trust-ping#messages
        Query - https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features#query-message-type
        Disclose - https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features#disclose-message-type
        Out-of-Band Invitation - https://github.com/hyperledger/aries-rfcs/tree/master/features/0434-outofband#message-type-httpsdidcommorgout-of-bandverinvitation
*/

/// Delete a Connection object from the agency and release its handle.
///
/// NOTE: This eliminates the connection and any ability to use it for any communication.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: handle of the connection to delete.
///
/// cb: Callback that provides feedback of the api call.
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_connection_delete_connection(command_handle: CommandHandle,
                                               connection_handle: Handle<Connections>,
                                               cb: Option<extern fn(
                                                   xcommand_handle: CommandHandle,
                                                   err: u32)>) -> u32 {
    info!("vcx_connection_delete_connection >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_delete_connection(command_handle: {}, connection_handle: {})", command_handle, connection_handle);

    spawn(move || {
        match connection_handle.delete_connection() {
            Ok(_) => {
                trace!("vcx_connection_delete_connection_cb(command_handle: {}, rc: {})", command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_connection_delete_connection_cb(command_handle: {}, rc: {})", command_handle, e);
                cb(command_handle, e.into());
            }
        }

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Connection object that provides a pairwise connection for an institution's user
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: user personal identification for the connection, should be unique.
///
/// cb: Callback that provides connection handle and error status of request
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_connection_create(command_handle: CommandHandle,
                                    source_id: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: Handle<Connections>)>) -> u32 {
    info!("vcx_connection_create >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_create(command_handle: {}, source_id: {})", command_handle, source_id);

    spawn(move || {
        match create_connection(&source_id) {
            Ok(handle) => {
                trace!("vcx_connection_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_connection_create_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Connection object from the given Invitation that provides a pairwise connection.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: user personal identification for the connection, should be unique.
///
/// invite_details: A string representing a json object which is provided by an entity that wishes to make a connection.
///
/// cb: Callback that provides connection handle and error status of request
///
/// # Examples
/// invite_details -> depends on communication protocol is used by another side:
///     proprietary:
///         {"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}
///     aries: https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
///      {
///         "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation",
///         "label": "Alice",
///         "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "serviceEndpoint": "https://example.com/endpoint",
///         "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]
///      }
///
/// # Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_create_with_invite(command_handle: CommandHandle,
                                                source_id: *const c_char,
                                                invite_details: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: Handle<Connections>)>) -> u32 {
    info!("vcx_connection_create_with_invite >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(invite_details, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_create_with_invite(command_handle: {}, source_id: {}, invite_details: {})",
           command_handle, source_id, secret!(invite_details));

    spawn(move || {
        match create_connection_with_invite(&source_id, &invite_details) {
            Ok(handle) => {
                trace!("vcx_connection_create_with_invite_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_connection_create_with_invite_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Connection object that provides an Out-of-Band Connection for an institution's user.
///
/// NOTE: this method can be used when `aries` protocol is set.
///
/// NOTE: this method is EXPERIMENTAL
///
/// WARN: `request_attach` field is not fully supported in the current library state.
///        You can use simple messages like Question but it cannot be used
///         for Credential Issuance and Credential Presentation.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: user personal identification for the connection, should be unique.
///
/// goal_code: Optional<string> - a self-attested code the receiver may want to display to
///                               the user or use in automatically deciding what to do with the out-of-band message.
///
/// goal:  Optional<string> - a self-attested string that the receiver may want to display to the user about
///                           the context-specific goal of the out-of-band message.
///
/// handshake: whether Inviter wants to establish regular connection using `connections` handshake protocol.
///            if false, one-time connection channel will be created.
///
/// request_attach: Optional<string> - An additional message as JSON that will be put into attachment decorator
///                                    that the receiver can using in responding to the message (for example Question message).
///
/// cb: Callback that provides
///     - error status of function
///     - connection handle that should be used to perform actions with the Connection object.
///
/// # Returns
/// Error code as a u32
#[no_mangle]
#[allow(unused_assignments)]
pub extern fn vcx_connection_create_outofband(command_handle: CommandHandle,
                                              source_id: *const c_char,
                                              goal_code: *const c_char,
                                              goal: *const c_char,
                                              handshake: bool,
                                              request_attach: *const c_char,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: Handle<Connections>)>) -> u32 {
    info!("vcx_connection_create_outofband >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(goal_code, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(goal, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(request_attach, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_create_outofband(command_handle: {}, source_id: {}, goal_code: {:?}, goal: {:?}, handshake: {}, request_attach: {:?})",
           command_handle, source_id, secret!(goal_code), secret!(goal), secret!(handshake), secret!(request_attach));

    spawn(move || {
        match create_outofband_connection(&source_id, goal_code, goal, handshake, request_attach) {
            Ok(handle) => {
                trace!("vcx_connection_create_outofband_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_connection_create_outofband_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Create a Connection object from the given Out-of-Band Invitation.
/// Depending on the format of Invitation there are two way of follow interaction:
///     * Invitation contains `handshake_protocols`: regular Connection process will be ran.
///         Follow steps as for regular Connection establishment.
///     * Invitation does not contain `handshake_protocols`: one-time completed Connection object will be created.
///         You can use `vcx_connection_send_message` or specific function to send a response message.
///         Note that on repeated message sending an error will be thrown.
///
/// NOTE: this method can be used when `aries` protocol is set.
///
/// WARN: The user has to analyze the value of "request~attach" field yourself and
///       create/handle the correspondent state object or send a reply once the connection is established.
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: user personal identification for the connection, should be unique.
///
/// invite: A JSON string representing Out-of-Band Invitation provided by an entity that wishes interaction.
///
/// cb: Callback that provides connection handle and error status of request
///
/// # Examples
/// invite ->
///     {
///         "@type": "https://didcomm.org/out-of-band/%VER/invitation",
///         "@id": "<id used for context as pthid>", -  the unique ID of the message.
///         "label": Optional<string>, - a string that the receiver may want to display to the user,
///                                      likely about who sent the out-of-band message.
///         "goal_code": Optional<string>, - a self-attested code the receiver may want to display to
///                                          the user or use in automatically deciding what to do with the out-of-band message.
///         "goal": Optional<string>, - a self-attested string that the receiver may want to display to the user
///                                     about the context-specific goal of the out-of-band message.
///         "handshake_protocols": Optional<[string]>, - an array of protocols in the order of preference of the sender
///                                                     that the receiver can use in responding to the message in order to create or reuse a connection with the sender.
///                                                     One or both of handshake_protocols and request~attach MUST be included in the message.
///         "request~attach": Optional<[
///             {
///                 "@id": "request-0",
///                 "mime-type": "application/json",
///                 "data": {
///                     "json": "<json of protocol message>"
///                 }
///             }
///         ]>, - an attachment decorator containing an array of request messages in order of preference that the receiver can using in responding to the message.
///               One or both of handshake_protocols and request~attach MUST be included in the message.
///         "service": [
///             {
///                 "id": string
///                 "type": string,
///                 "recipientKeys": [string],
///                 "routingKeys": [string],
///                 "serviceEndpoint": string
///             }
///         ] - an item that is the equivalent of the service block of a DIDDoc that the receiver is to use in responding to the message.
///     }
///
/// # Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_create_with_outofband_invitation(command_handle: CommandHandle,
                                                              source_id: *const c_char,
                                                              invite: *const c_char,
                                                              cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                                   err: u32,
                                                                                   connection_handle: Handle<Connections>)>) -> u32 {
    info!("vcx_connection_create_with_outofband_invitation >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(invite, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_create_with_outofband_invitation(command_handle: {}, source_id: {}, invite: {})",
           command_handle, source_id, secret!(invite));

    spawn(move || {
        match create_connection_with_outofband_invite(&source_id, &invite) {
            Ok(handle) => {
                trace!("vcx_connection_create_with_outofband_invitation_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), handle, source_id);
                cb(command_handle, error::SUCCESS.code_num, handle);
            }
            Err(x) => {
                warn!("vcx_connection_create_with_outofband_invitation_cb(command_handle: {}, rc: {}, handle: {}) source_id: {}",
                      command_handle, x, 0, source_id);
                cb(command_handle, x.into(), Handle::dummy());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Accept connection for the given invitation.
///
/// This function performs the following actions:
/// 1. Creates Connection state object from the given invitation
///     (equal to `vcx_connection_create_with_invite` function).
/// 2. Replies to the inviting side
///     (equal to `vcx_connection_connect` function).
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// source_id: user personal identification for the connection, should be unique.
///
/// invite_details: a string representing a json object which is provided by an entity
///     that wishes to make a connection.
///
/// connection_options: Provides details about establishing connection
///     {
///         "connection_type": Option<"string"> - one of "SMS", "QR",
///         "phone": "string": Option<"string"> - phone number in case "connection_type" is set into "SMS",
///         "update_agent_info": Option<bool> - whether agent information needs to be updated.
///                                             default value for `update_agent_info`=true
///                                             if agent info does not need to be updated, set `update_agent_info`=false
///         "use_public_did": Option<bool> - whether to use public DID for an establishing connection
///                                          default value for `use_public_did`=false
///         "pairwise_agent_info": Optional<JSON object> - pairwise agent to use instead of creating a new one.
///                                                        Can be received by calling `vcx_create_pairwise_agent` function.
///                                                         {
///                                                             "pw_did": string,
///                                                             "pw_vk": string,
///                                                             "agent_did": string,
///                                                             "agent_vk": string,
///                                                         }
///     }
///
/// cb: Callback that provides connection handle and error status of request.
///
/// # Examples
/// invite_details -> two formats are allowed depending on communication protocol:
///     proprietary:
///         {
///             "targetName":"",
///             "statusMsg":"message created",
///             "connReqId":"mugIkrWeMr",
///             "statusCode":"MS-101",
///             "threadId":null,
///             "senderAgencyDetail":{
///                 "endpoint":"http://localhost:8080",
///                 "verKey":"key",
///                 "DID":"did"
///             },
///             "senderDetail":{
///                 "agentKeyDlgProof":{
///                     "agentDID":"8f6gqnT13GGMNPWDa2TRQ7",
///                     "agentDelegatedKey":"5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1",
///                     "signature":"TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/+D8ZDg=="
///                  },
///                 "publicDID":"7YLxxEfHRiZkCMVNii1RCy",
///                 "name":"Faber",
///                 "logoUrl":"http://robohash.org/234",
///                 "verKey":"CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx",
///                 "DID":"Ney2FxHT4rdEyy6EDCCtxZ"
///                 }
///             }
///     aries: https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol#0-invitation-to-connect
///      {
///         "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation",
///         "label": "Alice",
///         "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "serviceEndpoint": "https://example.com/endpoint",
///         "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]
///      }
///
/// connection_options ->
/// "{"connection_type":"SMS","phone":"123","use_public_did":true}"
///     OR:
/// "{"connection_type":"QR","phone":"","use_public_did":false}"
///
/// # Returns
/// err: the result code as a u32
/// connection_handle: the handle associated with the created Connection object.
/// connection_serialized: the json string representing the created Connection object.
#[no_mangle]
pub extern fn vcx_connection_accept_connection_invite(command_handle: CommandHandle,
                                                      source_id: *const c_char,
                                                      invite_details: *const c_char,
                                                      connection_options: *const c_char,
                                                      cb: Option<extern fn(
                                                          xcommand_handle: CommandHandle,
                                                          err: u32,
                                                          connection_handle: Handle<Connections>,
                                                          connection_serialized: *const c_char)>) -> u32 {
    info!("vcx_connection_accept_connection_invite >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(source_id, VcxErrorKind::InvalidOption);
    check_useful_c_str!(invite_details, VcxErrorKind::InvalidOption);

    let connection_options_ = if !connection_options.is_null() {
        check_useful_opt_c_str!(connection_options, VcxErrorKind::InvalidOption);
        connection_options.to_owned()
    } else {
        None
    };

    trace!("vcx_connection_accept_connection_invite(command_handle: {}, source_id: {}, invite_details: {:?}, connection_options: {:?})",
           command_handle, source_id, secret!(invite_details), secret!(connection_options_));

    spawn(move || {
        match accept_connection_invite(&source_id, &invite_details, connection_options_) {
            Ok((connection_handle, connection_serialized)) => {
                trace!("vcx_connection_accept_connection_invite(command_handle: {}, rc: {}, connection_handle: {}, connection_serialized: {}) source_id: {}",
                       command_handle, error::SUCCESS.as_str(), connection_handle, secret!(connection_serialized), source_id);
                let connection_serialized_ = CStringUtils::string_to_cstring(connection_serialized);
                cb(command_handle, error::SUCCESS.code_num, connection_handle, connection_serialized_.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_accept_connection_invite(command_handle: {}, rc: {}) source_id: {}",
                      command_handle, x, source_id);
                cb(command_handle, x.into(), Handle::dummy(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Establishes connection between institution and its user
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies connection object
///
/// connection_options: Provides details about establishing connection
///     {
///         "connection_type": Option<"string"> - one of "SMS", "QR",
///         "phone": "string": Option<"string"> - phone number in case "connection_type" is set into "SMS",
///         "update_agent_info": Option<bool> - whether agent information needs to be updated.
///                                             default value for `update_agent_info`=true
///                                             if agent info does not need to be updated, set `update_agent_info`=false
///         "use_public_did": Option<bool> - whether to use public DID for an establishing connection
///                                          default value for `use_public_did`=false
///         "pairwise_agent_info": Optional<JSON object> - pairwise agent to use instead of creating a new one.
///                                                        Can be received by calling `vcx_create_pairwise_agent` function.
///                                                         {
///                                                             "pw_did": string,
///                                                             "pw_vk": string,
///                                                             "agent_did": string,
///                                                             "agent_vk": string,
///                                                         }
///     }
/// # Examples connection_options ->
/// "{"connection_type":"SMS","phone":"123","use_public_did":true, "update_agent_info": Option<true>}"
///     OR:
/// "{"connection_type":"QR","phone":"","use_public_did":false}"
///
/// cb: Callback that provides error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_connect(command_handle: CommandHandle,
                                     connection_handle: Handle<Connections>,
                                     connection_options: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, invite_details: *const c_char)>) -> u32 {
    info!("vcx_connection_connect >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let options = if !connection_options.is_null() {
        check_useful_opt_c_str!(connection_options, VcxErrorKind::InvalidOption);
        connection_options.to_owned()
    } else {
        None
    };

    trace!("vcx_connection_connect(command_handle: {}, connection_handle: {}, connection_options: {:?})",
           command_handle, connection_handle, secret!(options));

    spawn(move || {
        match connection_handle.connect(options) {
            Ok(_) => {
                match connection_handle.get_invite_details(true) {
                    Ok(x) => {
                        trace!("vcx_connection_connect_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {})",
                               command_handle, connection_handle, error::SUCCESS.as_str(), secret!(x));
                        let msg = CStringUtils::string_to_cstring(x);
                        cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
                    }
                    Err(_) => {
                        warn!("vcx_connection_connect_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {})",
                              command_handle, connection_handle, error::SUCCESS.as_str(), "null"); // TODO: why Success?????
                        cb(command_handle, error::SUCCESS.code_num, ptr::null_mut());
                    }
                }
            }
            Err(x) => {
                warn!("vcx_connection_connect_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {})",
                      command_handle, connection_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_connection_redirect(command_handle: CommandHandle,
                                      connection_handle: Handle<Connections>,
                                      redirect_connection_handle: Handle<Connections>,
                                      cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_connection_redirect >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_redirect(command_handle: {}, connection_handle: {}, redirect_connection_handle: {})",
           command_handle, connection_handle, redirect_connection_handle);

    spawn(move || {
        match connection_handle.redirect(redirect_connection_handle) {
            Ok(_) => {
                trace!("vcx_connection_redirect_cb(command_handle: {}, rc: {})", command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_connection_redirect_cb(command_handle: {}, rc: {})", command_handle, e);
                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_connection_get_redirect_details(command_handle: CommandHandle,
                                                  connection_handle: Handle<Connections>,
                                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, details: *const c_char)>) -> u32 {
    info!("vcx_connection_get_redirect_details >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_get_redirect_details(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match connection_handle.get_redirect_details() {
            Ok(str) => {
                trace!("vcx_connection_get_redirect_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {})",
                       command_handle, connection_handle, error::SUCCESS.as_str(), secret!(str));
                let msg = CStringUtils::string_to_cstring(str);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_get_redirect_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {})",
                      command_handle, connection_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes the Connection object and returns a json string of all its attributes
///
/// # Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides json string of the connection's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_serialize(command_handle: CommandHandle,
                                       connection_handle: Handle<Connections>,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_serialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_serialize(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match connection_handle.to_string() {
            Ok(json) => {
                trace!("vcx_connection_serialize_cb(command_handle: {}, connection_handle: {}, rc: {}, state: {})",
                       command_handle, connection_handle, error::SUCCESS.as_str(), secret!(json));
                let msg = CStringUtils::string_to_cstring(json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_serialize_cb(command_handle: {}, connection_handle: {}, rc: {}, state: {})",
                      command_handle, connection_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Takes a json string representing a connection object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_data: json string representing a connection object. Is an output of `vcx_connection_serialize` function.
///
/// cb: Callback that provides credential handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_deserialize(command_handle: CommandHandle,
                                         connection_data: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, connection_handle: Handle<Connections>)>) -> u32 {
    info!("vcx_connection_deserialize >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(connection_data, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_deserialize(command_handle: {}, connection_data: {})",
           command_handle, secret!(connection_data));

    spawn(move || {
        let (rc, handle) = match from_string(&connection_data) {
            Ok(x) => {
                trace!("vcx_connection_deserialize_cb(command_handle: {}, rc: {}, handle: {})",
                       command_handle, error::SUCCESS.as_str(), x);
                (error::SUCCESS.code_num, x)
            }
            Err(x) => {
                warn!("vcx_connection_deserialize_cb(command_handle: {}, rc: {}, handle: {} )",
                      command_handle, x, 0);
                (x.into(), Handle::dummy())
            }
        };

        cb(command_handle, rc, handle);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Query the agency for the received messages.
/// Checks for any messages changing state in the connection and updates the state attribute.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// cb: Callback that provides most current state of the credential and error status of request
///     Connection states:
///         1 - Initialized
///         2 - Request Sent
///         3 - Offer Received
///         4 - Accepted
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_update_state(command_handle: CommandHandle,
                                          connection_handle: Handle<Connections>,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_update_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_update_state(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match connection_handle.update_state(None) {
            Ok(state) => {
                trace!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), connection_handle, state);
                cb(command_handle, error::SUCCESS.code_num, state);
            }
            Err(x) => {
                let state = connection_handle.get_state();
                warn!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {})",
                      command_handle, x, connection_handle, state);
                cb(command_handle, x.into(), state);
            }
        };
        Ok(())
    });

    error::SUCCESS.code_num
}

/// Update the state of the connection based on the given message.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// message: message to process.
///
/// cb: Callback that provides most current state of the connection and error status of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_update_state_with_message(command_handle: CommandHandle,
                                                       connection_handle: Handle<Connections>,
                                                       message: *const c_char,
                                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_update_state_with_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(message, VcxErrorKind::InvalidOption);

    spawn(move || {
        match connection_handle.update_state_with_message(message) {
            Ok(state) => {
                trace!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {})",
                       command_handle, error::SUCCESS.as_str(), connection_handle, state);
                cb(command_handle, error::SUCCESS.code_num, state);
            }
            Err(x) => {
                let state = connection_handle.get_state();
                warn!("vcx_connection_update_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {})",
                      command_handle, x, connection_handle, state);
                cb(command_handle, x.into(), state);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Returns the current internal state of the connection. Does NOT query agency for state updates.
///     Possible states:
///         1 - Initialized
///         2 - Offer Sent
///         3 - Request Received
///         4 - Accepted
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that was provided during creation. Used to access connection object
///
/// cb: Callback that provides most current state of the connection and error status of request
///
/// #Returns
#[no_mangle]
pub extern fn vcx_connection_get_state(command_handle: CommandHandle,
                                       connection_handle: Handle<Connections>,
                                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, state: u32)>) -> u32 {
    info!("vcx_connection_get_state >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_get_state(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        let state = connection_handle.get_state();
        trace!("vcx_connection_get_state_cb(command_handle: {}, rc: {}, connection_handle: {}, state: {})",
               command_handle, error::SUCCESS.as_str(), connection_handle, state);
        cb(command_handle, error::SUCCESS.code_num, state);

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get the invite details that were sent or can be sent to the remote side.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// abbreviated: abbreviated connection details for QR codes or not (applicable for `proprietary` communication method only)
///
/// cb: Callback that provides the json string of details
///
/// # Example
/// details -> depends on communication method:
///     proprietary:
///       {"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}
///     aries:
///      {
///         "label": "Alice",
///         "serviceEndpoint": "https://example.com/endpoint",
///         "recipientKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "routingKeys": ["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],
///         "protocols": [
///             {"pid": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0", "roles": "Invitee"},
///             ...
///         ] - optional array. The set of protocol supported by remote side. Is filled after DiscoveryFeatures process was completed.
/////    }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_invite_details(command_handle: CommandHandle,
                                            connection_handle: Handle<Connections>,
                                            abbreviated: bool,
                                            cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, details: *const c_char)>) -> u32 {
    info!("vcx_connection_invite_details >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_invite_details(command_handle: {}, connection_handle: {}, abbreviated: {})",
           command_handle, connection_handle, abbreviated);

    spawn(move || {
        match connection_handle.get_invite_details(abbreviated) {
            Ok(str) => {
                trace!("vcx_connection_invite_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {})",
                       command_handle, connection_handle, error::SUCCESS.as_str(), secret!(str));
                let msg = CStringUtils::string_to_cstring(str);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_invite_details_cb(command_handle: {}, connection_handle: {}, rc: {}, details: {})",
                      command_handle, connection_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a message to the specified connection
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send the message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// msg: actual message to send
///
/// send_msg_options: (applicable for `proprietary` communication method only)
///     {
///         msg_type: String, // type of message to send. can be any string.
///         msg_title: String, // message title (user notification)
///         ref_msg_id: Option<String>, // If responding to a message, id of the message
///     }
///
/// # Example:
/// msg ->
///     "HI"
///   OR
///     {"key": "value"}
///   OR
///     {
///         "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/trust_ping/1.0/ping",
///         "@id": "518be002-de8e-456e-b3d5-8fe472477a86",
///         "comment": "Hi. Are you listening?",
///         "response_requested": true
///     }
///
/// send_msg_options ->
///     {
///         "msg_type":"Greeting",
///         "msg_title": "Hi There"
///     }
///   OR
///     {
///         "msg_type":"Greeting",
///         "msg_title": "Hi There",
///         "ref_msg_id" "as2d343sag"
///     }
///
/// cb: Callback that provides id of retrieved response message
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_message(command_handle: CommandHandle,
                                          connection_handle: Handle<Connections>,
                                          msg: *const c_char,
                                          send_msg_options: *const c_char,
                                          cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, msg_id: *const c_char)>) -> u32 {
    info!("vcx_connection_send_message >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);
    check_useful_c_str!(msg, VcxErrorKind::InvalidOption);
    check_useful_c_str!(send_msg_options, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_message(command_handle: {}, connection_handle: {}, msg: {}, send_msg_options: {})",
           command_handle, connection_handle, secret!(msg), secret!(send_msg_options));

    spawn(move || {
        match connection_handle.send_generic_message(&msg, &send_msg_options) {
            Ok(msg_id) => {
                trace!("vcx_connection_send_message_cb(command_handle: {}, rc: {}, msg_id: {})",
                       command_handle, error::SUCCESS.as_str(), msg_id);

                let msg_id = CStringUtils::string_to_cstring(msg_id);
                cb(command_handle, error::SUCCESS.code_num, msg_id.as_ptr());
            }
            Err(e) => {
                warn!("vcx_connection_send_message_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send trust ping message to the specified connection to prove that two agents have a functional pairwise channel.
///
/// Note that this function is useful in case `aries` communication method is used.
/// In other cases it returns ActionNotSupported error.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send ping message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// comment: (Optional) human-friendly description of the ping.
///
/// cb: Callback that provides success or failure of request
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_ping(command_handle: u32,
                                       connection_handle: Handle<Connections>,
                                       comment: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_ping >>>");

    check_useful_opt_c_str!(comment, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_ping(command_handle: {}, connection_handle: {}, comment: {:?})",
           command_handle, connection_handle, secret!(comment));

    spawn(move || {
        match connection_handle.send_ping(comment) {
            Ok(()) => {
                trace!("vcx_connection_send_ping(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_connection_send_ping(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Generate a signature for the specified data using connection pairwise keys
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to sign the message.
///                    Was provided during creation. Used to identify connection object.
///
/// data_raw: raw data buffer for signature
///
/// data_len: length of data buffer
///
/// cb: Callback that provides the generated signature
///
/// # Example
/// data_raw -> [1, 2, 3, 4, 5, 6]
/// data_len -> 6
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_sign_data(command_handle: CommandHandle,
                                       connection_handle: Handle<Connections>,
                                       data_raw: *const u8,
                                       data_len: u32,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: u32,
                                                            signature_raw: *const u8,
                                                            signature_len: u32)>) -> u32 {
    info!("vcx_connection_sign_data >>>");

    check_useful_c_byte_array!(data_raw, data_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_sign_data(command_handle: {}, connection_handle: {}, data_raw: {:?}, data_len: {:?})",
           command_handle, connection_handle, secret!(data_raw), secret!(data_len));


    let vk = match connection_handle.get_pw_verkey() {
        Ok(x) => x,
        Err(e) => return e.into(),
    };

    spawn(move || {
        match ::utils::libindy::crypto::sign(&vk, &data_raw) {
            Ok(x) => {
                trace!("vcx_connection_sign_data_cb(command_handle: {}, connection_handle: {}, rc: {}, signature: {:?})",
                       command_handle, connection_handle, error::SUCCESS.as_str(), x);

                let (signature_raw, signature_len) = ::utils::cstring::vec_to_pointer(&x);
                cb(command_handle, error::SUCCESS.code_num, signature_raw, signature_len);
            }
            Err(e) => {
                warn!("vcx_messages_sign_data_cb(command_handle: {}, rc: {}, signature: null)",
                      command_handle, e);

                cb(command_handle, e.into(), ptr::null_mut(), 0);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Verify the signature is valid for the specified data using connection pairwise keys
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to verify signature.
///                    Was provided during creation. Used to identify connection object.
///
/// data_raw: raw data buffer for signature
///
/// data_len: length of data buffer
///
/// signature_raw: raw data buffer for signature
///
/// signature_len: length of data buffer
///
/// cb: Callback that specifies whether the signature was valid or not
///
/// # Example
/// data_raw -> [1, 2, 3, 4, 5, 6]
/// data_len -> 6
/// signature_raw -> [2, 3, 4, 5, 6, 7]
/// signature_len -> 6
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_verify_signature(command_handle: CommandHandle,
                                              connection_handle: Handle<Connections>,
                                              data_raw: *const u8,
                                              data_len: u32,
                                              signature_raw: *const u8,
                                              signature_len: u32,
                                              cb: Option<extern fn(command_handle_: CommandHandle,
                                                                   err: u32,
                                                                   valid: bool)>) -> u32 {
    info!("vcx_connection_verify_signature >>>");

    check_useful_c_byte_array!(data_raw, data_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_byte_array!(signature_raw, signature_len, VcxErrorKind::InvalidOption, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_verify_signature(command_handle: {}, connection_handle: {}, data_raw: {:?}, data_len: {:?}, signature_raw: {:?}, signature_len: {:?})",
           command_handle, connection_handle, secret!(data_raw), secret!(data_len), secret!(signature_raw), secret!(signature_len));

    let vk = match connection_handle.get_their_pw_verkey() {
        Ok(x) => x,
        Err(e) => return e.into(),
    };

    spawn(move || {
        match ::utils::libindy::crypto::verify(&vk, &data_raw, &signature_raw) {
            Ok(x) => {
                trace!("vcx_connection_verify_signature_cb(command_handle: {}, rc: {}, valid: {})",
                       command_handle, error::SUCCESS.as_str(), x);

                cb(command_handle, error::SUCCESS.code_num, x);
            }
            Err(e) => {
                warn!("vcx_connection_verify_signature_cb(command_handle: {}, rc: {}, valid: {})",
                      command_handle, e, false);

                cb(command_handle, e.into(), false);
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Releases the connection object by de-allocating memory
///
/// #Params
/// connection_handle: was provided during creation. Used to identify connection object
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_connection_release(connection_handle: Handle<Connections>) -> u32 {
    info!("vcx_connection_release >>>");

    spawn(move || {
        match connection_handle.release() {
            Ok(()) => {
                trace!("vcx_connection_release(connection_handle: {}, rc: {})",
                       connection_handle, error::SUCCESS.as_str());
            }
            Err(e) => {
                warn!("vcx_connection_release(connection_handle: {}), rc: {})",
                      connection_handle, e);
            }
        };
        Ok(())
    });
    error::SUCCESS.code_num
}

/// Send discovery features message to the specified connection to discover which features it supports, and to what extent.
///
/// Note that this function is useful in case `aries` communication method is used.
/// In other cases it returns ActionNotSupported error.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: connection to use to send message.
///                    Was provided during creation. Used to identify connection object.
///                    Note that connection must be in Accepted state.
///
/// query: (Optional) query string to match against supported message types.
///
/// comment: (Optional) human-friendly description of the query.
///
/// cb: Callback that provides success or failure of request
///
/// # Example
/// query -> `did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/`
///
/// comment -> `share please`
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_discovery_features(command_handle: u32,
                                                     connection_handle: Handle<Connections>,
                                                     query: *const c_char,
                                                     comment: *const c_char,
                                                     cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_discovery_features >>>");

    check_useful_opt_c_str!(query, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(comment, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_discovery_features(command_handle: {}, connection_handle: {}, query: {:?}, comment: {:?})",
           command_handle, connection_handle, secret!(query), secret!(comment));

    spawn(move || {
        match connection_handle.send_discovery_features(query, comment) {
            Ok(()) => {
                trace!("vcx_connection_send_discovery_features(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_connection_send_discovery_features(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a message to reuse existing Connection instead of setting up a new one
/// as response on received Out-of-Band Invitation.
///
/// Note that this function works in case `aries` communication method is used.
///     In other cases it returns ActionNotSupported error.
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: handle pointing to Connection to awaken and send reuse message.
///
/// invite: A JSON string representing Out-of-Band Invitation provided by an entity that wishes interaction.
///
/// cb: Callback that provides success or failure of request
///
/// # Examples
/// invite ->
///     {
///         "@type": "https://didcomm.org/out-of-band/%VER/invitation",
///         "@id": "<id used for context as pthid>", -  the unique ID of the message.
///         "label": Optional<string>, - a string that the receiver may want to display to the user,
///                                      likely about who sent the out-of-band message.
///         "goal_code": Optional<string>, - a self-attested code the receiver may want to display to
///                                          the user or use in automatically deciding what to do with the out-of-band message.
///         "goal": Optional<string>, - a self-attested string that the receiver may want to display to the user
///                                     about the context-specific goal of the out-of-band message.
///         "handshake_protocols": Optional<[string]>, - an array of protocols in the order of preference of the sender
///                                                     that the receiver can use in responding to the message in order to create or reuse a connection with the sender.
///                                                     One or both of handshake_protocols and request~attach MUST be included in the message.
///         "request~attach": Optional<[
///             {
///                 "@id": "request-0",
///                 "mime-type": "application/json",
///                 "data": {
///                     "json": "<json of protocol message>"
///                 }
///             }
///         ]>, - an attachment decorator containing an array of request messages in order of preference that the receiver can using in responding to the message.
///               One or both of handshake_protocols and request~attach MUST be included in the message.
///         "service": [
///             {
///                 "id": string
///                 "type": string,
///                 "recipientKeys": [string],
///                 "routingKeys": [string],
///                 "serviceEndpoint": string
///             }
///         ] - an item that is the equivalent of the service block of a DIDDoc that the receiver is to use in responding to the message.
///     }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_reuse(command_handle: u32,
                                        connection_handle: Handle<Connections>,
                                        invite: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_reuse >>>");

    check_useful_c_str!(invite, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_reuse(command_handle: {}, connection_handle: {}, invite: {})",
           command_handle, connection_handle, secret!(invite));

    spawn(move || {
        match connection_handle.send_reuse(invite) {
            Ok(()) => {
                trace!("vvcx_connection_send_reuse_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vvcx_connection_send_reuse_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send answer on received question message according to Aries question-answer or committedanswer protocols.
///
/// The related Aries question-answer protocol can be found here: https://github.com/hyperledger/aries-rfcs/tree/master/features/0113-question-answer
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: handle pointing to Connection to use for sending answer message.
///
/// question: A JSON string representing Question received via pairwise connection.
///
/// answer: An answer to use which is a JSON string representing chosen `valid_response` option from Question message.
///
/// cb: Callback that provides success or failure of request
///
/// # Examples
/// Aries question-answer:
///     question ->
///         {
///             "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/questionanswer/1.0/question",
///             "@id": "518be002-de8e-456e-b3d5-8fe472477a86",
///             "question_text": "Alice, are you on the phone with Bob from Faber Bank right now?",
///             "question_detail": "This is optional fine-print giving context to the question and its various answers.",
///             "nonce": "<valid_nonce>",
///             "signature_required": true,
///             "valid_responses" : [
///                 {"text": "Yes, it's me"},
///                 {"text": "No, that's not me!"}],
///             "~timing": {
///             "   expires_time": "2018-12-13T17:29:06+0000"
///             }
///         }
///     answer ->
///         {"text": "Yes, it's me"}
/// committedanswer:
///     question ->
///         {
///            '@type': 'did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/committedanswer/1.0/question',
///            '@id': '518be002-de8e-456e-b3d5-8fe472477a86',
///            'question_text': 'Alice, are you on the phone with Bob from Faber Bank right now?',
///            'question_detail': 'This is optional fine-print giving context to the question and its various answers.',
///            'valid_responses': [
///                {'text': 'Yes, it is me', 'nonce': '<unique_identifier_a+2018-12-13T17:00:00+0000>'},
///                {'text': 'No, that is not me!', 'nonce': '<unique_identifier_b+2018-12-13T17:00:00+0000'},
///                {'text': 'Hi!', 'nonce': '<unique_identifier_c+2018-12-13T17:00:00+0000'}],
///            '@timing': {
///                'expires_time': future
///            },
///            'external_links': [
///                {
///                    'text': 'Some external link with so many characters that it can go outside of two lines range from here onwards',
///                    'src': '1'},
///                {
///                    'src': 'Some external link with so many characters that it can go outside of two lines range from here onwards'},
///            ]
///        }
///     answer ->
///         {'text': 'Yes, it is me', 'nonce': '<unique_identifier_a+2018-12-13T17:00:00+0000>'}
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_answer(command_handle: u32,
                                         connection_handle: Handle<Connections>,
                                         question: *const c_char,
                                         answer: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    info!("vcx_connection_send_answer >>>");

    check_useful_c_str!(question, VcxErrorKind::InvalidOption);
    check_useful_c_str!(answer, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_send_answer(command_handle: {}, connection_handle: {}, question: {}, answer: {})",
           command_handle, connection_handle, secret!(question), secret!(answer));

    spawn(move || {
        match connection_handle.send_answer(question, answer) {
            Ok(()) => {
                trace!("vcx_connection_send_answer_cb(command_handle: {}, rc: {})",
                       command_handle, error::SUCCESS.as_str());
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                warn!("vcx_connection_send_answer_cb(command_handle: {}, rc: {})",
                      command_handle, e);

                cb(command_handle, e.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Send a message to invite another side to take a particular action.
/// The action is represented as a `goal_code` and should be described in a way that can be automated.
///
/// The related protocol can be found here:
///     https://github.com/hyperledger/aries-rfcs/blob/ecf4090b591b1d424813b6468f5fc391bf7f495b/features/0547-invite-action-protocol
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: handle pointing to Connection to send invite action.
///
/// data: string - JSON containing information to build message
///     {
///         goal_code: string - A code the receiver may want to display to the user or use in automatically deciding what to do after receiving the message.
///         ack_on: Optional<array<string>> - Specify when ACKs message need to be sent back from invitee to inviter:
///             * not needed - None or empty array
///             * at the time the invitation is accepted - ["ACCEPT"]
///             * at the time out outcome for the action is known - ["OUTCOME"]
///             * both - ["ACCEPT", "OUTCOME"]
///     }
///
/// cb: Callback that provides sent message
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_send_invite_action(command_handle: u32,
                                                connection_handle: Handle<Connections>,
                                                data: *const c_char,
                                                cb: Option<extern fn(xcommand_handle: u32, err: u32, message: *const c_char)>) -> u32 {
    info!("vcx_connection_send_invite_action >>>");

    check_useful_c_str!(data, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    let data: InviteActionData = match serde_json::from_str(&data) {
        Ok(x) => x,
        Err(err) => {
            return VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse InviteData from `msg_options` JSON string. Err: {:?}", err)).into();
        }
    };

    trace!("vcx_connection_send_invite_action(command_handle: {}, connection_handle: {}, data: {:?})",
           command_handle, connection_handle, secret!(data));

    spawn(move || {
        match connection_handle.send_invite_action(data) {
            Ok(message) => {
                trace!("vcx_connection_send_invite_action_cb(command_handle: {}, rc: {}, message: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(message));
                let msg = CStringUtils::string_to_cstring(message);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                warn!("vcx_connection_send_invite_action_cb(command_handle: {}, rc: {})",
                      command_handle, e);
                cb(command_handle, e.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}


/// Get the information about the connection state.
///
/// Note: This method can be used for `aries` communication method only.
///     For other communication method it returns ActionNotSupported error.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: was provided during creation. Used to identify connection object
///
/// cb: Callback that provides the json string of connection information
///
/// # Example
/// info ->
///      {
///         "current": {
///             "did": <str>
///             "recipientKeys": array<str>
///             "routingKeys": array<str>
///             "serviceEndpoint": <str>,
///             "protocols": array<str> -  The set of protocol supported by current side.
///         },
///         "remote: { <Option> - details about remote connection side
///             "did": <str> - DID of remote side
///             "recipientKeys": array<str> - Recipient keys
///             "routingKeys": array<str> - Routing keys
///             "serviceEndpoint": <str> - Endpoint
///             "protocols": array<str> - The set of protocol supported by side. Is filled after DiscoveryFeatures process was completed.
///          }
///    }
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_info(command_handle: CommandHandle,
                                  connection_handle: Handle<Connections>,
                                  cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, info: *const c_char)>) -> u32 {
    info!("vcx_connection_info >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_info(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match connection_handle.get_connection_info() {
            Ok(info) => {
                trace!("vcx_connection_info(command_handle: {}, connection_handle: {}, rc: {}, info: {})",
                       command_handle, connection_handle, error::SUCCESS.as_str(), secret!(info));
                let info = CStringUtils::string_to_cstring(info);
                cb(command_handle, error::SUCCESS.code_num, info.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_info(command_handle: {}, connection_handle: {}, rc: {}, info: {})",
                      command_handle, connection_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves pw_did from Connection object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides your pw_did for this connection
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_get_pw_did(command_handle: u32,
                                        connection_handle: Handle<Connections>,
                                        cb: Option<extern fn(xcommand_handle: u32, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_get_pw_did >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_get_pw_did(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match connection_handle.get_pw_did() {
            Ok(did) => {
                trace!("vcx_connection_get_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, pw_did: {})",
                       command_handle, connection_handle, error::SUCCESS.as_str(), secret!(did));
                let msg = CStringUtils::string_to_cstring(did);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_get_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, pw_did: {})",
                      command_handle, connection_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieves their_pw_did from Connection object
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: Connection handle that identifies pairwise connection
///
/// cb: Callback that provides your pw_did for this connection
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_get_their_pw_did(command_handle: u32,
                                              connection_handle: Handle<Connections>,
                                              cb: Option<extern fn(xcommand_handle: u32, err: u32, serialized_data: *const c_char)>) -> u32 {
    info!("vcx_connection_get_pw_did >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_get_their_pw_did(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match connection_handle.get_their_pw_did() {
            Ok(json) => {
                trace!("vcx_connection_get_their_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, their_pw_did: {})",
                       command_handle, connection_handle, error::SUCCESS.as_str(), secret!(json));
                let msg = CStringUtils::string_to_cstring(json);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(x) => {
                warn!("vcx_connection_get_their_pw_did_cb(command_handle: {}, connection_handle: {}, rc: {}, their_pw_did: {})",
                      command_handle, connection_handle, x, "null");
                cb(command_handle, x.into(), ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Get Problem Report message for Connection object in Failed or Rejected state.
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// connection_handle: handle pointing to Connection state object.
///
/// cb: Callback that returns Problem Report as JSON string or null
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_connection_get_problem_report(command_handle: CommandHandle,
                                                connection_handle: Handle<Connections>,
                                                cb: Option<extern fn(xcommand_handle: CommandHandle,
                                                                     err: u32,
                                                                     message: *const c_char)>) -> u32 {
    info!("vcx_connection_get_problem_report >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_connection_get_problem_report(command_handle: {}, connection_handle: {})",
           command_handle, connection_handle);

    spawn(move || {
        match connection_handle.get_problem_report_message() {
            Ok(message) => {
                trace!("vcx_connection_get_problem_report_message_cb(command_handle: {}, rc: {}, msg: {})",
                       command_handle, error::SUCCESS.as_str(), secret!(message));
                let message = CStringUtils::string_to_cstring(message);
                cb(command_handle, error::SUCCESS.code_num, message.as_ptr());
            }
            Err(x) => {
                error!("vcx_connection_get_problem_report_message_cb(command_handle: {}, rc: {})",
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
    use connection::tests::build_test_connection;
    use utils::error;
    use api::{return_types, VcxStateType};
    use utils::constants::{GET_MESSAGES_RESPONSE, INVITE_ACCEPTED_RESPONSE};
    use utils::error::SUCCESS;
    use utils::devsetup::*;
    use utils::httpclient::AgencyMock;

    const EMPTY_JSON: *const c_char = "{}\0".as_ptr().cast();
    #[test]
    fn test_vcx_connection_create() {
        let _setup = SetupMocks::init();
        let (h, cb, r) = return_types::return_u32_cxnh();
        let _rc = vcx_connection_create(h,
                                        "test_create\0".as_ptr().cast(),
                                        Some(cb));
        assert!(r.recv_medium().unwrap() > 0)
    }

    #[test]
    fn test_vcx_connection_create_fails() {
        let _setup = SetupMocks::init();

        let rc = vcx_connection_create(0,
                                       "test_create_fails\0".as_ptr().cast(),
                                       None);
        assert_eq!(rc, error::INVALID_OPTION.code_num);
        let (h, cb, _r) = return_types::return_u32_cxnh();
        let rc = vcx_connection_create(h,
                                       ptr::null(),
                                       Some(cb));
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[test]
    fn test_vcx_connection_connect() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32_str();
        vcx_connection_connect(h, Handle::dummy(), EMPTY_JSON, Some(cb));
        let rc = r.recv_medium().unwrap_err();
        assert_eq!(rc, error::INVALID_CONNECTION_HANDLE.code_num);


        let handle = build_test_connection();
        assert!(handle > 0);
        let (h, cb, r) = return_types::return_u32_str();
        vcx_connection_connect(h, handle, EMPTY_JSON, Some(cb));
        let invite_details = r.recv_medium().unwrap();
        assert!(invite_details.is_some());
    }

    #[test]
    fn test_vcx_connection_redirect() {
        let _setup = SetupMocks::init();

        let (h, cb, r) = return_types::return_u32();
        vcx_connection_redirect(h, Handle::dummy(), Handle::dummy(), Some(cb));
        let rc = r.recv_medium().unwrap_err();
        assert_eq!(rc, error::INVALID_CONNECTION_HANDLE.code_num);

        let handle = build_test_connection();
        assert!(handle > 0);

        let (h, cb, r) = return_types::return_u32();
        vcx_connection_redirect(h, handle, Handle::dummy(), Some(cb));
        let rc = r.recv_medium().unwrap_err();
        assert_eq!(rc, error::INVALID_CONNECTION_HANDLE.code_num);

        let handle2 = create_connection("alice2").unwrap();
        handle2.connect(Some("{}".to_string())).unwrap();
        assert!(handle2 > 0);

        let (h, cb, _r) = return_types::return_u32();
        let rc = vcx_connection_redirect(h, handle, handle2, Some(cb));
        assert_eq!(rc, error::SUCCESS.code_num);
    }

    #[test]
    fn test_vcx_connection_update_state() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();
        assert!(handle > 0);
        handle.connect(None).unwrap();
        let (h, cb, r) = return_types::return_u32_u32();
        AgencyMock::set_next_response(GET_MESSAGES_RESPONSE);
        let rc = vcx_connection_update_state(h, handle, Some(cb));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    fn test_vcx_connection_update_state_with_message() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();
        assert!(handle > 0);
        handle.connect(None).unwrap();
        let (h, cb, r) = return_types::return_u32_u32();
        let response = CString::new(INVITE_ACCEPTED_RESPONSE).unwrap();
        let rc = vcx_connection_update_state_with_message(h, handle, response.as_ptr(), Some(cb));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[test]
    fn test_vcx_connection_update_state_fails() {
        let _setup = SetupMocks::init();

        let rc = vcx_connection_update_state(0, Handle::dummy(), None);
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[test]
    fn test_vcx_connection_serialize() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();
        assert!(handle > 0);

        let (h, cb, r) = return_types::return_u32_str();
        let rc = vcx_connection_serialize(h, handle, Some(cb));
        assert_eq!(rc, error::SUCCESS.code_num);

        assert!(r.recv_medium().unwrap().is_some());
    }

    #[test]
    fn test_vcx_connection_release() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();

        let rc = vcx_connection_release(handle);
        assert_eq!(rc, error::SUCCESS.code_num);
    }

    #[test]
    fn test_vcx_connection_deserialize_succeeds() {
        let _setup = SetupMocks::init();

        let string = ::utils::constants::DEFAULT_CONNECTION;
        let data = CString::new(string).unwrap();
        let (h, cb, r) = return_types::return_u32_cxnh();
        let err = vcx_connection_deserialize(h,
                                             data.as_ptr(),
                                             Some(cb));
        assert_eq!(err, SUCCESS.code_num);
        let handle = r.recv_short().unwrap();
        assert!(handle > 0);
    }

    #[test]
    fn test_vcx_connection_get_state() {
        let _setup = SetupMocks::init();

        let handle = build_test_connection();

        AgencyMock::set_next_response(GET_MESSAGES_RESPONSE);

        let (h, cb, r) = return_types::return_u32_u32();
        let _rc = vcx_connection_update_state(h, handle, Some(cb));
        assert_eq!(r.recv_medium().unwrap(), VcxStateType::VcxStateAccepted as u32);

        let (h, cb, r) = return_types::return_u32_u32();
        let rc = vcx_connection_get_state(h, handle, Some(cb));
        assert_eq!(rc, error::SUCCESS.code_num);
        assert_eq!(r.recv_medium().unwrap(), VcxStateType::VcxStateAccepted as u32)
    }

    #[test]
    fn test_vcx_connection_delete_connection() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();

        let (h, cb, r) = return_types::return_u32();
        assert_eq!(vcx_connection_delete_connection(h, connection_handle, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();

        assert_eq!(connection_handle.get_source_id().unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_send_message() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();
        connection_handle.set_state(VcxStateType::VcxStateAccepted).unwrap();

        let msg = "MESSAGE\0".as_ptr().cast();
        let send_msg_options = concat!(r#"{"msg_type":"type", "msg_title": "title", "ref_msg_id":null}"#, "\0").as_ptr().cast();
        let (h, cb, r) = return_types::return_u32_str();
        assert_eq!(vcx_connection_send_message(h, connection_handle, msg, send_msg_options, Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }

    #[test]
    fn test_sign() {
        let _setup = SetupMocks::init();

        let connection_handle = ::connection::tests::build_test_connection();

        let msg = "My message\0";
        let msg_len = msg.len() - 1;

        let (h, cb, r) = return_types::return_u32_bin();
        assert_eq!(vcx_connection_sign_data(h,
                                            connection_handle,
                                            msg.as_ptr().cast(),
                                            msg_len as u32,
                                            Some(cb)), error::SUCCESS.code_num);
        let _sig = r.recv_medium().unwrap();
    }

    #[test]
    fn test_verify_signature() {
        let _setup = SetupMocks::init();

        let connection_handle = ::connection::tests::build_test_connection();

        let msg = "My message\0";
        let msg_len = msg.len() - 1;

        let signature = "signature\0";
        let signature_length = signature.len() - 1;

        let (h, cb, r) = return_types::return_u32_bool();
        assert_eq!(vcx_connection_verify_signature(h,
                                                   connection_handle,
                                                   msg.as_ptr().cast(),
                                                   msg_len as u32,
                                                   signature.as_ptr().cast(),
                                                   signature_length as u32,
                                                   Some(cb)), error::SUCCESS.code_num);
        r.recv_medium().unwrap();
    }
}
