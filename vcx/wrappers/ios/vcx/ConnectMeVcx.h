//
//  init.h
//  vcx
//
//  Created by GuestUser on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#ifndef init_h
#define init_h
#import "libvcx.h"

extern void VcxWrapperCommonCallback(vcx_command_handle_t xcommand_handle,
                                     vcx_error_t err);

extern void VcxWrapperCommonHandleCallback(vcx_command_handle_t xcommand_handle,
                                           vcx_error_t err,
                                           vcx_command_handle_t pool_handle);

extern void VcxWrapperCommonStringCallback(vcx_command_handle_t xcommand_handle,
                                           vcx_error_t err,
                                           const char *const arg1);

extern void VcxWrapperCommonBoolCallback(vcx_command_handle_t xcommand_handle,
                                         vcx_error_t err,
                                         unsigned int arg1);

extern void VcxWrapperCommonStringStringCallback(vcx_command_handle_t xcommand_handle,
                                                 vcx_error_t err,
                                                 const char *const arg1,
                                                 const char *const arg2);

extern void VcxWrapperCommonStringOptStringCallback(vcx_command_handle_t xcommand_handle,
                                                    vcx_error_t err,
                                                    const char *const arg1,
                                                    const char *const arg2);

extern void VcxWrapperCommonDataCallback(vcx_command_handle_t xcommand_handle,
                                         vcx_error_t err,
                                         const uint8_t *const arg1,
                                         uint32_t arg2);

extern void VcxWrapperCommonStringStringStringCallback(vcx_command_handle_t xcommand_handle,
                                                       vcx_error_t err,
                                                       const char *const arg1,
                                                       const char *const arg2,
                                                       const char *const arg3);

extern void VcxWrapperCommonStringDataCallback(vcx_command_handle_t xcommand_handle,
                                               vcx_error_t err,
                                               const char *const arg1,
                                               const uint8_t *const arg2,
                                               uint32_t arg3);

extern void VcxWrapperCommonNumberCallback(vcx_command_handle_t xcommand_handle,
                                           vcx_error_t err,
                                           vcx_command_handle_t handle);

extern void VcxWrapperCommonStringOptStringOptStringCallback(vcx_command_handle_t xcommand_handle,
                                                             vcx_error_t err,
                                                             const char *const arg1,
                                                             const char *const arg2,
                                                             const char *const arg3);

extern void VcxWrapperCommonStringStringLongCallback(vcx_command_handle_t xcommand_handle,
                                                     vcx_error_t err,
                                                     const char *arg1,
                                                     const char *arg2,
                                                     unsigned long long arg3);

extern void VcxWrapperCommonNumberStringCallback(vcx_command_handle_t xcommand_handle,
                                                 vcx_error_t err,
                                                 vcx_command_handle_t handle,
                                                 const char *const arg2);

@interface ConnectMeVcx : NSObject

- (int)initSovToken;

//- (int)initNullPay;

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
- (void)initWithConfig:(NSString *)config
            completion:(void (^)(NSError *error))completion;

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
/// pool_config: string - the configuration JSON containing pool related settings:
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
///    }
///                 }
///
///
/// cb: Callback that provides no value
///
/// #Returns
/// Error code as u32
- (void)initPool:(NSString *)poolConfig
            completion:(void (^)(NSError *error))completion;

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
/// Configuration (wallet also populated), on error returns NULL
- (void)agentProvisionAsync:(NSString *)config
                 completion:(void (^)(NSError *error, NSString *config))completion;

/// Provision an agent in the agency, populate configuration and wallet for this agent.
///
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
/// Configuration
- (const char *)agentProvisionWithToken:(NSString *)config
                          token:(NSString *)token;

/// Provision an agent in the agency, populate configuration and wallet for this agent.
///
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
/// Configuration
- (void)agentProvisionWithTokenAsync:(NSString *)config
                               token:(NSString *)token
                          completion:(void (^)(NSError *error, NSString *result))completion;


/// Get token which can be used for provisioning an agent
/// NOTE: Can be used only for Evernym's applications
///
/// config:
/// {
///     vcx_config: VcxConfig // Same config passed to agent provision
///                           // See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
///     sponsee_id: String,
///     sponsor_id: String,
///     com_method: {
///         type: u32 // 1 means push notifications, 4 means forward to sponsor app
///         id: String,
///         value: String,
///     },
/// }
///
/// #Returns
/// token: provisioning token as JSON
- (void)getProvisionToken:(NSString *)config
                 completion:(void (^)(NSError *error, NSString *token))completion;

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
- (void)connectionCreateWithInvite:(NSString *)invitationId
                     inviteDetails:(NSString *)inviteDetails
                        completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

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
- (void)connectionCreateOutofband:(NSString *)sourceId
                         goalCode:(NSString *)goalCode
                             goal:(NSString *)goal
                        handshake:(BOOL *)handshake
                    requestAttach:(NSString *)requestAttach
                       completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

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
- (void)acceptConnectionWithInvite:(NSString *)invitationId
                     inviteDetails:(NSString *)inviteDetails
                    connectionType:(NSString *)connectionType
                        completion:(void (^)(NSError *error, NSInteger connectionHandle, NSString *serializedConnection))completion;

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
- (void)connectionCreateWithOutofbandInvite:(NSString *)invitationId
                                     invite:(NSString *)invite
                                 completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

/// Establishes connection between institution and its user
///
/// # Params
/// connectionHandle: Connection handle that identifies connection object
///
/// connectionType: Provides details about establishing connection
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
/// #Returns
///  Connection invitation as JSON string
- (void)connectionConnect:(VcxHandle)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *error, NSString *inviteDetails))completion;

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
- (void)connectionGetState:(NSInteger)connectionHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)connectionUpdateState:(NSInteger) connectionHandle
                   completion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)connectionSerialize:(NSInteger)connectionHandle
                 completion:(void (^)(NSError *error, NSString *serializedConnection))completion;

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
- (void)connectionDeserialize:(NSString *)serializedConnection
                   completion:(void (^)(NSError *error, NSInteger connectionHandle))completion;

/// Releases the connection object by de-allocating memory
///
/// #Params
/// connection_handle: was provided during creation. Used to identify connection object
///
/// #Returns
/// Success
- (int)connectionRelease:(NSInteger) connectionHandle;

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
- (void)deleteConnection:(VcxHandle)connectionHandle
          withCompletion:(void (^)(NSError *error))completion;

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
- (void)connectionSendMessage:(VcxHandle)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *error, NSString *msg_id))completion;

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
- (void)connectionSendPing:(VcxHandle)connectionHandle
                   comment:(NSString *)comment
            withCompletion:(void (^)(NSError *error))completion;

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
- (void)connectionSendReuse:(VcxHandle)connectionHandle
                     invite:(NSString *)invite
             withCompletion:(void (^)(NSError *error))completion;

/// Send answer on received question message according to Aries question-answer protocol.
///
/// The related protocol can be found here: https://github.com/hyperledger/aries-rfcs/tree/master/features/0113-question-answer
///
/// Note that this function works in case `aries` communication method is used.
///     In other cases it returns ActionNotSupported error.
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
/// question ->
///     {
///         "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/questionanswer/1.0/question",
///         "@id": "518be002-de8e-456e-b3d5-8fe472477a86",
///         "question_text": "Alice, are you on the phone with Bob from Faber Bank right now?",
///         "question_detail": "This is optional fine-print giving context to the question and its various answers.",
///         "nonce": "<valid_nonce>",
///         "signature_required": true,
///         "valid_responses" : [
///             {"text": "Yes, it's me"},
///             {"text": "No, that's not me!"}],
///         "~timing": {
///             "expires_time": "2018-12-13T17:29:06+0000"
///         }
///     }
/// answer ->
///     {"text": "Yes, it's me"}
///
/// #Returns
/// Error code as a u32
- (void)connectionSendAnswer:(VcxHandle)connectionHandle
                    question:(NSString *)question
                      answer:(NSString *)answer
             withCompletion:(void (^)(NSError *error))completion;

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
- (void)connectionSendInviteAction:(VcxHandle)connectionHandle
                              data:(NSString *)data
                    withCompletion:(void (^)(NSError *error, NSString *message))completion;

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
- (void)connectionSignData:(VcxHandle)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *error, NSData *signature_raw, vcx_u32_t signature_len))completion;

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
- (void)connectionVerifySignature:(VcxHandle)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *error, vcx_bool_t valid))completion;

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
- (void)connectionUpdateState:(VcxHandle) connectionHandle
               withCompletion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)connectionUpdateStateWithMessage:(VcxHandle) connectionHandle
                                 message:(NSString *)message
                          withCompletion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)connectionGetState:(VcxHandle) connectionHandle
            withCompletion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)connectionGetProblemReport:(NSInteger) connectionHandle
                        completion:(void (^)(NSError *error, NSString *message))completion;

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
- (void)connectionCreateInvite:(NSString *)sourceId
             completion:(void (^)(NSError *error, NSInteger connectionHandle)) completion;

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
- (void) getConnectionInviteDetails:(NSInteger) connectionHandle
                        abbreviated:(BOOL *) abbreviated
         withCompletion:(void (^)(NSError *error, NSString *inviteDetails))completion;

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
- (void)agentUpdateInfo:(NSString *)config
             completion:(void (^)(NSError *error))completion;

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
- (void)getCredential:(NSInteger )credentailHandle
           completion:(void (^)(NSError *error, NSString *credential))completion;

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
- (void)deleteCredential:(NSInteger )credentialHandle
              completion:(void (^)(NSError *error))completion;

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
- (void)credentialCreateWithOffer:(NSString *)sourceId
                            offer:(NSString *)credentialOffer
                       completion:(void (^)(NSError *error, NSInteger credentailHandle))completion;

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
- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(VcxHandle)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *error, NSInteger credentialHandle, NSString *credentialOffer))completion;

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
- (void)credentialAcceptCredentialOffer:(NSString *)sourceId
                                  offer:(NSString *)credentialOffer
                       connectionHandle:(VcxHandle)connectionHandle
                             completion:(void (^)(NSError *error, NSInteger credentialHandle, NSString *credentialSerialized))completion;

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
- (void)credentialSendRequest:(NSInteger)credentialHandle
             connectionHandle:(VcxHandle)connectionHandle
                paymentHandle:(vcx_payment_handle_t)paymentHandle
                   completion:(void (^)(NSError *error))completion;

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
- (void)credentialGetState:(NSInteger )credentialHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

/// Query the agency for the received messages.
/// Checks for any messages changing state in the credential object and updates the state attribute.
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
- (void)credentialUpdateState:(NSInteger )credentailHandle
                completion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)credentialUpdateStateWithMessage:(VcxHandle) credentialHandle
                                 message:(NSString *)message
                          withCompletion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)credentialGetOffers:(VcxHandle)connectionHandle
                 completion:(void (^)(NSError *error, NSString *offers))completion;

/// Send a Credential rejection to the connection.
/// It can be called once Credential Offer or Credential messages are received.
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
- (void)credentialReject:(NSInteger)credentialHandle
        connectionHandle:(VcxHandle)connectionHandle
                 comment:(NSString *)comment
              completion:(void (^)(NSError *error))completion;

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
- (void)credentialGetPresentationProposal:(NSInteger )credentialHandle
                               completion:(void (^)(NSError *error, NSString *presentationProposal))completion;

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
- (void)credentialSerialize:(NSInteger)credentialHandle
                 completion:(void (^)(NSError *error, NSString *state))completion;

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
- (void)credentialDeserialize:(NSString *)serializedCredential
                   completion:(void (^)(NSError *error, NSInteger credentialHandle))completion;

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
- (void)credentialGetProblemReport:(NSInteger) credentialHandle
                        completion:(void (^)(NSError *error, NSString *message))completion;

/// Releases the credential object by de-allocating memory
///
/// #Params
/// handle: Credential handle that was provided during creation. Used to access credential object
///
/// #Returns
/// Success
- (int)credentialRelease:(NSInteger) credentialHandle;

/// Exports opened wallet
///
/// Note this endpoint is EXPERIMENTAL. Function signature and behaviour may change
/// in the future releases.
///
/// #Params:
/// command_handle: Handle for User's Reference only.
/// path: Path to export wallet to User's File System.
/// backup_key: String representing the User's Key for securing (encrypting) the exported Wallet.
/// cb: Callback that provides the success/failure of the api call.
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
- (void)exportWallet:(NSString *)exportPath
         encryptWith:(NSString *)encryptionKey
          completion:(void (^)(NSError *error, NSInteger exportHandle))completion;

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
- (void)importWallet:(NSString *)config
           completion:(void (^)(NSError *error))completion;

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
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
- (void)addRecordWallet:(NSString *)recordType
            recordId:(NSString *)recordId
            recordValue:(NSString *) recordValue
           completion:(void (^)(NSError *error))completion;

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
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
- (void)updateRecordWallet:(NSString *)recordType
              withRecordId:(NSString *)recordId
           withRecordValue:(NSString *) recordValue
            withCompletion:(void (^)(NSError *error))completion;

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
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
/// Error will be a libindy error code
- (void)getRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
             completion:(void (^)(NSError *error, NSString *walletValue))completion;

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
/// cb: Callback that any errors or a receipt of transfer
///
/// #Returns
/// Error code as a u32
/// Error will be a libindy error code
- (void)deleteRecordWallet:(NSString *)recordType
            recordId:(NSString *)recordId
           completion:(void (^)(NSError *error))completion;

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
- (void) proofGetRequests:(NSInteger)connectionHandle
              completion:(void (^)(NSError *error, NSString *requests))completion;

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
- (void) proofRetrieveCredentials:(vcx_proof_handle_t)proofHandle
                   withCompletion:(void (^)(NSError *error, NSString *matchingCredentials))completion;

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
- (void) proofGenerate:(vcx_proof_handle_t)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
 withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
        withCompletion:(void (^)(NSError *error))completion;

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
- (void) proofCreateWithMsgId:(NSString *)source_id
         withConnectionHandle:(vcx_connection_handle_t)connectionHandle
                    withMsgId:(NSString *)msgId
               withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle, NSString *proofRequest))completion;

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
- (void) requestProof:(vcx_proof_handle_t)proof_handle
 withConnectionHandle:(vcx_connection_handle_t)connection_handle
       requestedAttrs:(NSString *)requestedAttrs
  requestedPredicates:(NSString *)requestedPredicates
            proofName:(NSString *)proofName
   revocationInterval:(NSString *)revocationInterval
       withCompletion:(void (^)(NSError *error))completion;

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
- (void)proofGetPresentationProposal:(vcx_proof_handle_t)proof_handle
                          completion:(void (^)(NSError *error, NSString *presentationProposal))completion;

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
- (void) proofSend:(vcx_proof_handle_t)proof_handle
withConnectionHandle:(vcx_connection_handle_t)connection_handle
    withCompletion:(void (^)(NSError *error))completion;

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
- (void) proofSendProposal:(vcx_proof_handle_t)proof_handle
      withConnectionHandle:(vcx_connection_handle_t)connection_handle
            withCompletion:(void (^)(NSError *error))completion;

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
- (void)proofGetState:(NSInteger)proofHandle
           completion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)proofUpdateState:(NSInteger) proofHandle
              completion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void) proofReject: (vcx_proof_handle_t)proof_handle
      withConnectionHandle:(vcx_connection_handle_t)connection_handle
      withCompletion: (void (^)(NSError *error))completion;

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
- (void) proofDeclinePresentationRequest:(vcx_proof_handle_t)proof_handle
                    withConnectionHandle:(vcx_connection_handle_t)connection_handle
                              withReason:(NSString *)reason
                            withProposal:(NSString *)proposal
                          withCompletion:(void (^)(NSError *error))completion;

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
- (void) getProofMsg:(vcx_proof_handle_t) proofHandle
      withCompletion:(void (^)(NSError *error, NSString *proofMsg))completion;

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
- (void) getRejectMsg:(vcx_proof_handle_t) proofHandle
       withCompletion:(void (^)(NSError *error, NSString *rejectMsg))completion;

- (void)connectionRedirect:(vcx_connection_handle_t)redirect_connection_handle
      withConnectionHandle:(vcx_connection_handle_t)connection_handle
            withCompletion:(void (^)(NSError *error))completion;

- (void)getRedirectDetails:(vcx_connection_handle_t)connection_handle
      withCompletion:(void (^)(NSError *error, NSString *redirectDetails))completion;

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
- (void) proofCreateWithRequest:(NSString *) source_id
               withProofRequest:(NSString *) proofRequest
                 withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle))completion;

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
- (void) proofCreateProposal:(NSString *) source_id
           withProofProposal:(NSString *) proofProposal
                 withComment:(NSString *) comment
              withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle))completion;

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
- (void) proofSerialize:(vcx_proof_handle_t) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *proof_request))completion;

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
- (void) proofDeserialize:(NSString *) serializedProof
           withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle)) completion;

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
- (void)proofUpdateStateWithMessage:(VcxHandle) proofHandle
                            message:(NSString *)message
                     withCompletion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void)proofGetProblemReport:(VcxHandle) proofHandle
                   completion:(void (^)(NSError *error, NSString *message))completion;

/// Releases the disclosed proof object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// Success
- (int)proofRelease:(NSInteger) proofHandle;

/// Reset libvcx to a pre-configured state, releasing/deleting any handles and freeing memory
///
/// libvcx will be inoperable and must be initialized again with vcx_init_with_config
///
/// #Params
/// delete: specify whether wallet/pool should be deleted
///
/// #Returns
/// Success
- (int)vcxShutdown:(BOOL *)deleteWallet;

/// Add a payment address to the wallet
///
/// #params
///
/// cb: Callback that provides payment address info
///
/// # Example payment_address -> "pay:null:9UFgyjuJxi1i1HD"
///
/// #Returns
/// Error code as u32
- (void)createPaymentAddress:(NSString *)seed
              withCompletion:(void (^)(NSError *error, NSString *address))completion;

/// Get the total balance from all addresses contained in the configured wallet
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// payment_handle: for future use
///
/// cb: Callback that provides wallet balance
///
/// # Example info -> "{"balance":6,"balance_str":"6","addresses":[{"address":"pay:null:9UFgyjuJxi1i1HD","balance":3,"utxo":[{"source":"pay:null:1","paymentAddress":"pay:null:zR3GN9lfbCVtHjp","amount":1,"extra":"yqeiv5SisTeUGkw"}]}]}"
///
/// #Returns
/// Error code as a u32
- (void)getTokenInfo:(vcx_payment_handle_t)payment_handle
      withCompletion:(void (^)(NSError *error, NSString *tokenInfo))completion;

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
- (void)sendTokens:(vcx_payment_handle_t)payment_handle
        withTokens:(NSString *)tokens
     withRecipient:(NSString *)recipient
    withCompletion:(void (^)(NSError *error, NSString *recipient))completion;

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
- (void)downloadMessages:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
                  pwdids:(NSString *)pwdids
              completion:(void (^)(NSError *error, NSString* messages))completion;

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
- (void)downloadMessage:(NSString *)uid
             completion:(void (^)(NSError *error, NSString* message))completion;

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
- (void)updateMessages:(NSString *)messageStatus
            pwdidsJson:(NSString *)pwdidsJson
            completion:(void (^)(NSError *error))completion;

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
- (void)downloadAgentMessages:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
                    completion:(void (^)(NSError *error, NSString* messages))completion;

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
- (void) getLedgerFees:(void(^)(NSError *error, NSString *fees)) completion;

/// Retrieve author agreement and acceptance mechanisms set on the Ledger
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides array of matching messages retrieved
///
/// # Example author_agreement -> "{"text":"Default agreement", "version":"1.0.0", "aml": {"label1": "description"}}"
///
/// #Returns
/// Error code as a u32
- (void) getTxnAuthorAgreement:(void(^)(NSError *error, NSString *authorAgreement)) completion;

/// Set some accepted agreement as active.
///
/// As result of successful call of this function appropriate metadata will be appended to each write request.
///
/// #Params
/// text and version - (optional) raw data about TAA from ledger.
///     These parameters should be passed together.
///     These parameters are required if hash parameter is ommited.
/// hash - (optional) hash on text and version. This parameter is required if text and version parameters are ommited.
/// acc_mech_type - mechanism how user has accepted the TAA
/// time_of_acceptance - UTC timestamp when user has accepted the TAA
///
/// #Returns
/// Error code as a u32
- (vcx_error_t) activateTxnAuthorAgreement:(NSString *)text
                               withVersion:(NSString *)version
                                  withHash:(NSString *)hash
                             withMechanism:(NSString *)mechanism
                             withTimestamp:(long)timestamp;

/**
 Fetch and Cache public entities from the Ledger associated with stored in the wallet credentials.
 This function performs two steps:
     1) Retrieves the list of all credentials stored in the opened wallet.
     2) Fetch and cache Schemas / Credential Definitions / Revocation Registry Definitions
        correspondent to received credentials from the connected Ledger.

 This helper function can be used, for instance as a background task, to refresh library cache.
 This allows us to reduce the time taken for Proof generation by using already cached entities instead of queering the Ledger.

 NOTE: Library must be already initialized (wallet and pool must be opened).

 Returns: void
*/

- (void)fetchPublicEntities:(void (^)(NSError *error))completion;

/**
 This function allows you to check the health of LibVCX and EAS/CAS instance.
 It will return error in case of any problems on EAS or will resolve pretty long if VCX is thread-hungry.
 WARNING: this call may take a lot of time returning answer in case of load, be careful.
 NOTE: Library must be initialized, ENDPOINT_URL should be set

 Returns: void
*/

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
- (void)healthCheck:(void (^)(NSError *error))completion;

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
- (void) createWalletBackup:(NSString *)sourceID
                  backupKey:(NSString *)backupKey
                 completion:(void (^)(NSError *error, NSInteger walletBackupHandle))completion;

/// Wallet Backup to the Cloud
///
/// #Params:
/// command_handle: Handle for User's Reference only.
/// wallet_backup_handle: Wallet Backup handle that was provided during creation. Used to access object
/// path: Path to export wallet to User's File System. (This instance of the export
/// cb: Callback that provides the success/failure of the api call.
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
- (void) backupWalletBackup:(vcx_wallet_backup_handle_t) walletBackupHandle
                       path:(NSString *)path
                 completion:(void(^)(NSError *error))completion;

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
- (void) updateWalletBackupState:(vcx_wallet_backup_handle_t) walletBackupHandle
                      completion:(void (^)(NSError *error, NSInteger state))completion;

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
- (void) updateWalletBackupStateWithMessage:(vcx_wallet_backup_handle_t) walletBackupHandle
                                    message:(NSString *)message
                                 completion:(void (^)(NSError *error, NSInteger state))completion;

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
// should *walletBackupStr be just data here?
- (void) serializeBackupWallet:(vcx_wallet_backup_handle_t) walletBackupHandle
              completion:(void (^)(NSError *error, NSString *data))completion;

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
- (void) deserializeBackupWallet:(NSString *) walletBackupStr
                completion:(void (^)(NSError *error, NSInteger walletBackupHandle))completion;

/// Requests a recovery of a backup previously stored with a cloud agent
///
/// config: "{"wallet_name":"","wallet_key":"","exported_wallet_path":"","backup_key":"","key_derivation":""}"
/// backup_key: Key used when creating the backup of the wallet (For encryption/decrption)
/// cb: Callback that provides the success/failure of the api call.
/// #Returns
/// Error code - success indicates that the api call was successfully created and execution
/// is scheduled to begin in a separate thread.
- (void)restoreWallet:(NSString *)config
           completion:(void (^)(NSError *error))completion;

/*
* Verifier API
*/

/// Create a new Proof object that requests a proof for an enterprise
///
/// #Params
/// sourceId: Enterprise's personal identification for the proof, should be unique.
///
/// proofName: Proof name
///
/// requestedAttributes: Describes requested attribute
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
/// requestedPredicates: predicate specifications prover must provide claim for
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
/// revocationInterval:  Optional<<revocation_interval>>, // see below,
///                        // If specified, prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (can be overridden on attribute level)
///     from: Optional<u64> // timestamp of interval beginning
///     to: Optional<u64> // timestamp of interval beginning
///         // Requested time represented as a total number of seconds from Unix Epoch, Optional
/// # Examples config ->  "{}" | "{"to": 123} | "{"from": 100, "to": 123}"
///
/// #Returns
/// Handle pointing to Proof Verifier object
- (void) createProofVerifier:(NSString *)sourceId
         requestedAttributes:(NSString *)requestedAttributes
         requestedPredicates:(NSString *)requestedPredicates
          revocationInterval:(NSString *)revocationInterval
                   proofName:(NSString *)proofName
                  completion:(void (^)(NSError *error, NSInteger handle))completion;

/// Create a new Proof object based on the given Presentation Proposal message
///
/// #Params
/// sourceId: Enterprise's personal identification for the proof, should be unique..
///
/// name: Proof name
///
/// presentationProposal: Message sent by the Prover to the verifier to initiate a proof presentation process:
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
/// #Returns
/// Handle pointing to Proof Verifier object
- (void) createProofVerifierWithProposal:(NSString *)sourceId
                    presentationProposal:(NSString *)presentationProposal
                                    name:(NSString *)name
                              completion:(void (^)(NSError *error, NSInteger handle))completion;

/// Query the agency for the received messages.
/// Checks for any messages changing state in the object and updates the state attribute.
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// The most current state of the Proof Object
- (void) proofVerifierUpdateState:(NSInteger) proofHandle
                       completion:(void (^)(NSError *error, NSInteger state))completion;

/// Update the state of the proof based on the given message.
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// message: message to process for state changes
///
/// #Returns
/// The most current state of the Proof Object
- (void) proofVerifierUpdateStateWithMessage:(NSInteger)proofHandle
                                    message:(NSString *)message
                                 completion:(void (^)(NSError *error, NSInteger state))completion;

/// Get the current state of the proof object
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// The most current state of the Proof Object
- (void) proofVerifierGetState:(NSInteger) proofHandle
                    completion:(void (^)(NSError *error, NSInteger state))completion;

/// Takes the proof object and returns a json string of all its attributes
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// json string of the state object
- (void) proofVerifierSerialize:(NSInteger) proofHandle
                     completion:(void (^)(NSError *error, NSString* serialized))completion;

/// Takes a json string representing a proof object and recreates an object matching the json
///
/// #Params
/// serialized: json string representing a proof object
///
/// #Returns
/// Handle pointing to Proof Verifier object
- (void) proofVerifierDeserialize:(NSString *) serialized
                       completion:(void (^)(NSError *error, NSInteger proofHandle))completion;

/// Sends a proof request to pairwise connection
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// Null
- (void) proofVerifierSendRequest:(NSInteger) proofHandle
                 connectionHandle:(NSInteger) connectionHandle
                       completion:(void (^)(NSError *error))completion;

/// Request a new proof after receiving a proof proposal (this enables negotiation)
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// connectionHandle: Connection handle that identifies pairwise connection
///
/// requestedAttributes: Describes requested attribute
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
/// requestedPredicates: predicate specifications prover must provide claim for
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
/// revocationInterval:  Optional<<revocation_interval>>, // see below,
///                        // If specified, prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (can be overridden on attribute level)
///     from: Optional<u64> // timestamp of interval beginning
///     to: Optional<u64> // timestamp of interval beginning
///         // Requested time represented as a total number of seconds from Unix Epoch, Optional
/// # Examples config ->  "{}" | "{"to": 123} | "{"from": 100, "to": 123}"
///
/// #Returns
/// Null
- (void) proofVerifierRequestForProposal:(NSInteger) proofHandle
                        connectionHandle:(NSInteger) connectionHandle
                     requestedAttributes:(NSString *)requestedAttributes
                     requestedPredicates:(NSString *)requestedPredicates
                      revocationInterval:(NSString *)revocationInterval
                               proofName:(NSString *)proofName
                              completion:(void (^)(NSError *error))completion;

/// Get the proof request message that can be sent to the specified connection
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// # Example proof_request -> "{'@topic': {'tid': 0, 'mid': 0}, '@type': {'version': '1.0', 'name': 'PROOF_REQUEST'}, 'proof_request_data': {'name': 'proof_req', 'nonce': '118065925949165739229152', 'version': '0.1', 'requested_predicates': {}, 'non_revoked': None, 'requested_attributes': {'attribute_0': {'name': 'name', 'restrictions': {'$or': [{'issuer_did': 'did'}]}}}, 'ver': '1.0'}, 'thread_id': '40bdb5b2'}"
///
/// #Returns
/// Message as JSON
- (void) proofVerifierGetProofRequestMessage:(NSInteger) proofHandle
                                 completion:(void (^)(NSError *error, NSString* message))completion;

/// Get the proof proposal received for deciding whether to accept it
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// Message as JSON
- (void) proofVerifierGetProofProposalMessage:(NSInteger) proofHandle
                                 completion:(void (^)(NSError *error, NSString* message))completion;

/// Get the proof request attachment that you send along the out of band credential
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to access proof object
///
/// # Example presentation_request_attachment -> "{"@id": "8b23c2b6-b432-45d8-a377-d003950c0fcc", "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/request-presentation", "comment": "Person Proving", "request_presentations~attach": [{"@id": "libindy-request-presentation-0", "data": {"base64": "eyJuYW1lIjoiUGVyc29uIFByb3ZpbmciLCJub25fcmV2b2tlZCI6bnVsbCwibm9uY2UiOiI2MzQxNzYyOTk0NjI5NTQ5MzA4MjY1MzQiLCJyZXF1ZXN0ZWRfYXR0cmlidXRlcyI6eyJhdHRyaWJ1dGVfMCI6eyJuYW1lIjoibmFtZSJ9LCJhdHRyaWJ1dGVfMSI6eyJuYW1lIjoiZW1haWwifX0sInJlcXVlc3RlZF9wcmVkaWNhdGVzIjp7fSwidmVyIjpudWxsLCJ2ZXJzaW9uIjoiMS4wIn0="}, "mime-type": "application/json"}]}"
///
/// #Returns
/// Message as JSON
- (void) proofVerifierGetProofRequestAttach:(NSInteger) proofHandle
                                 completion:(void (^)(NSError *error, NSString* message))completion;

/// Get Proof Msg
///
/// #Params
/// proofHandle: Proof handle that was provided during creation. Used to identify proof object
///
/// #Returns
/// Status and Message as JSON
- (void) proofVerifierGetProofMessage:(NSInteger) proofHandle
                                 completion:(void (^)(NSError *error, NSInteger proofState, NSString* message))completion;

/// Get Problem Report message for Proof object in Failed or Rejected state.
///
/// #Params
/// proofHandle: handle pointing to Proof state object.
///
/// #Returns
/// Status and Message as JSON
- (void) proofVerifierGetProblemReportMessage:(NSInteger) proofHandle
                                 completion:(void (^)(NSError *error, NSString* message))completion;

/// Bind proof state object with connection
///
/// #Params
/// proofHandle: handle pointing to Proof state object.
///
/// #Returns
/// Null
- (void) proofVerifierSetConnection:(NSInteger) proofHandle
                            connectionHandle:(NSInteger) connectionHandle
                                 completion:(void (^)(NSError *error))completion;

/// Create pairwise agent which can be later used for connection establishing.
///
/// You can pass `agent_info` into `vcx_connection_connect` function as field of `connection_options` JSON parameter.
/// The passed Pairwise Agent will be used for connection establishing instead of creation a new one.
///
/// #Returns
/// agent info as JSON string:
///     {
///         "pw_did": string,
///         "pw_vk": string,
///         "agent_did": string,
///         "agent_vk": string,
///     }
- (void) createPairwiseAgent:(void (^)(NSError *error, NSString *agentInfo))completion;

@end

#endif /* init_h */
