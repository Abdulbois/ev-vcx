# Changelog

## Release notes - EvLibVcx 0.14.0

### Tasks
* Dropped support of Ubuntu 16
* Added support for Ubuntu 20

## Release notes - EvLibVcx 0.13.1 Jan 21 2022

EvLibVcx 0.12.0 to EvLibVcx 0.13.1 [Migration guide](./docs/migration-guide-0.12.x-0.13.0.md).

### Bugfixes
* Deserialization does not work for Connection state objects created with usage of previous EvVCX versions.

## Release notes - EvLibVcx 0.13.0 Dec 30 2021

EvLibVcx 0.12.0 to EvLibVcx 0.13.0 [Migration guide](./docs/migration-guide-0.12.x-0.13.0.md).

### Breaking Change
* Removed `vcx_get_ledger_author_agreement` and `vcx_set_active_txn_author_agreement_meta` functions to get and set transaction author agreement data in the runtime. \
  Now you **MUST** specify transaction author agreement data for every connecting pool network inside of library initialization config.

### Tasks
* Added protocol type `4.0` (`protocol_type` setting in configuration JSON) implying that all input and output messages (`Credential Offers`, `Credential`, `Proof Requests`) will be in the Aries message format instead of legacy one.
* Migrated from the usage of [Hyperledger Indy-SDK](https://github.com/hyperledger/indy-sdk.git) to Evernym [VDR-Tools](https://gitlab.com/evernym/verity/vdr-tools.git).
* Added ability to connect to multiple Indy Pool Ledger networks and read data from them.
* Added helper function `vcx_extract_attached_message` to extract attachment content from Aries message containing attachment decorator.
* Added helper function `vcx_resolve_message_by_url` to resolve message by URL.
* Added helper function `vcx_extract_thread_id` to extract thread id from a message.
* Extend the result of `vcx_disclosed_proof_retrieve_credentials` to return more information for each requested attribute and predicate:
  * values of requested attributes fetched from credential (case-insensitive)
  * if an attribute can be self-attested (only when `protocol_type:4.0` is used)
  * if an attribute is missing (only when `protocol_type:4.0` is used)
* Added support for [Out-Of-Band 1.1](https://github.com/hyperledger/aries-rfcs/tree/main/features/0434-outofband) Aries protocol.
* Added support for NodeJS 12.
* Added React-Native package.

### Bugfixes
* Various efficiency improvements.
* Added functions missing in Objective-C wrapper (`connectionGetPwDid`, `connectionGetTheirDid`, `connectionInfo`, wallet related functions).

## Release notes - EvLibVcx 0.12.0 Jun 17 2021

### Tasks
* Added async version of function to provision agent with token - `vcx_provision_agent_with_token_async`.
* Added `vcx_create_pairwise_agent` function to create a pairwise agent which can be later used for connection establishment.
  It allows to speed up the connection establishing process.
  The result value should be passed into `vcx_connection_connect` function as `pairwise_agent_info` field of `connection_options` parameter.
* Added support `Transports Return Route` Aries RFC.
* Added support for connectionless credentials. You can pass 0 as connection handle into `vcx_credential_send_request`
  function in order to get a credential for a connectionless credential offer (containing `~service` decorator).
* Added wrapper for React-Native.
* Updated `vcx_connection_send_answer` function to reply on a question related to `committedanswer` protocol.
* Changed division of published artifacts for iOS devices.
  * From:
    *  vcx.libvcxall_*_universal.zip`  - for phones + simulators
    *  vcx.libvcxpartial_*_universal.zip` - for phones
  * To:
    * vcx.libvcxarm64 - for phones
    * vcx.libvcxx86_64 - for simulators

### Bugfixes
  * Various efficiency improvements.
  * VCX does not update the status of the Connection Request message as read.
  * Pool connection is optional for getting records from the cache.
  * Objective-C: Added missing function to get connection information - `getConnectionInviteDetails`.
  * Objective-C: Added API functions for Proof Verifier.

## Release notes - EvLibVcx 0.11.2 Mar 22 2021

### Bugfixes
  * Provisioning with token does not work after its enforcing (disabling of old provisioning protocol) on the Agency side.

## Release notes - EvLibVcx 0.11.0 Feb 20 2021

### Change
* IMPORTANT: Default `protocol_type` has been changed from `1.0` to `3.0`. It means that we are using Aries by default now.

### Tasks
* Added Proof Proposal functionality:
  * Added `vcx_disclosed_proof_create_proposal` function that starts presentation protocol from a proposal
  * Added `vcx_disclosed_proof_send_proposal` function that sends a proposal through connection
  * Added `vcx_proof_request_proof` function that sends a proof request in response with new data
  * Added `vcx_get_proof_proposal` function that returns proof proposal object
* Supported Ephemeral Proof Request via `~request-attach`:
  * Added `vcx_proof_get_request_attach` function that generates request attachment from a proof protocol instance.
  * Added `vcx_proof_set_connection` function that sets the connection for proof request that was send along with OOB proof request
* Added `vcx_health_check` call that queries Verity instance to see if it is alive.

## Release notes - EvLibVcx 0.10.1 Nov 24 2020

### Tasks
* Aries Invite Action Protocol:
  * Added `vcx_connection_send_invite_action` function which prepares and sends a message to invite another side to take a particular action.

### Bugfixes
* Fixed deadlock that may have happened when VCX of 0.9.4/1.10.0 versions was used via NodeJS.
* Corrected building of an Android aar artifact for armeabi-v7a architecture.
* Changed `get_provision_token` function to return token as JSON string value instead of void.
* Updated VCX Aries message threading handling to use `@id` of the first message as thread id (thid) only if that first message doesn't contain `thid`.

## Release notes - EvLibVcx 0.10.0 Nov 20 2020

### Tasks
* Added `vcx_credential_reject` function to explicitly reject Aries Credential Offer by sending a ProblemReport message.
* Added `vcx_delete_credential` function to delete Credential from the wallet.
* Supported Aries Question Answer Protocol:
  * added `vcx_connection_send_answer` which prepares and sends the answer on the received question.
* Partial support of Out-of-Band Aries protocol:
  * Sender - Added `vcx_connection_create_outofband` function which prepares Connection object containing Out-of-Band invitation.
    The parameter `handshake` specifies whether the Sender wants to establish a regular connection using connections handshake protocol or create a public channel.
    Next when you called `vcx_connection_connect` Connection state machine either goes by regular steps or transit to Accepted state when no handshake requested.

  * Received - Added `vcx_connection_create_with_outofband_invitation` function which accepts Out-of-Band invitation.
    If invitation contains `handshake_protocols` connection goes regular flow else transits to Completed state.
  * HandshakeReuse - Added `vcx_connection_send_reuse` function to send HandshakeReuse message.
  * request~attach:
    * Sender - It can be set into Out-of-Band invitation but VCX Issuance and Presentation state machines are not compatible with that protocol.
    * Receiver - User should start attached process once Connection is established.
* Add a helper function to download a single message from the Agency by the given id `vcx_agency_download_message`.
* Changed the logic for updating the status of the messages on the Agency (for Aries protocol only):
  * vcx_*_update_state - still update messages state on agency internally.
  * vcx_*_update_state_with_message - caller has full control, passes messages, and is also responsible to update states in agency.
* Updated handling of `~thread` decorator for Aries messages to track and set `sender_order` and `received_orders` fields.
* Updated building of DIDDoc to set `id` field according to W3C RFC.
* Adopted Aries `Problem Report` message for `issue-credential` and `present-proof` protocols.
  Previous VCX versions send general `Problem Report` messages from `notification` message family in case an error occurred in Issuance and Presentation protocols.
  VCX 0.10.0 sets appropriate `issue-credential`/`present-proof` message family while sending `Problem Report` message.
* Put `institution_logo_url` into Aries Connection invitation as `profileUrl` field.
* Added separate function for Pool initialization.
  Now we can deffer connecting to the Pool Ledger during library initialization(to decrease the time taken) by omitting `genesis_path` field in the config JSON.
  Next, we can use `vcx_init_pool` function (for instance as a background task) to perform a connection to the Pool Ledger.
* Added helper function `vcx_fetch_public_entities` to fetch public entities from the ledger.
  This function performs two steps:
  1) Retrieves the list of all credentials stored in the opened wallet.
  2) Fetch and cache Schemas / Credential Definitions / Revocation Registry Definitions correspondent to received credentials from the connected Ledger.
  This helper function can be used, for instance as a background task, to refresh library cache.
* Updated VCX library to make payment plugin optional dependency. We can omit installation of payment plugin in case we are not going to use payments for our application.

### Bugfixes
* Connection handles in Aries state machines can't be serialized properly.
  Overview: Aries Issuance and Presentation state machines held `connection_handle` as property.
  But actual Connection object matching to handle will be destroyed once the library is unloaded.
  That will break Aries state machines.
  Change:  Updated Aries Issuance and Presentation state machines to embed required connection-related data.
  Consequences: Deserialization for Aries Issuance and Presentation state machines in the intermediate state is broken but will work for Started and Finished.
* Added check that Credential Offer attributes fully match to Credential Definition. Partially filled credentials cannot be issued.
* Updated signature of Java API functions to return null (Void type) value for functions that actually do not return any result.
  Consequences: the combination of `exceptionally/thenAccept` function to handle results may treat null as an error.
  Tip: Use `whenComplete` function to proper handling instead of combination `exceptionally/thenAccept`.
* Fixed custom logger to generate only logs with a set level.
* Corrected `vcx_download_agent_messages` function to set `msg_ref_id` field for downloaded messages.

## Release notes - EvLibVcx 0.9.4 Aug 30 2020

### Tasks
* Significantly improved performance of the library by updating Object Cache to use concurrent map instead of the blocking variant.
* Updating user profile happens during provisioning and optionally during connecting.

## Release notes - EvLibVcx 0.9.3 Jul 31 2020

### Tasks
* Update the config parameter passed into vcx_agent_update_info function to allow an optional type field that can be used to distinguish between different classes of push notifications.

## Release notes - EvLibVcx 0.9.2 Jun 30 2020

### Tasks
* Backmerge from upstream Indy SDK repo

## Release notes - EvLibVcx 0.9.1 Jun 24 2020

### Bugfixes
* Bugfixes in wrappers. You do not need to migrate onto this release if you are not using  NodeJS or Python wrapper.

## Release notes - EvLibVcx 0.9.2 May 25 2020

### Tasks
* Supported `libmysqlstorage` plugin.
