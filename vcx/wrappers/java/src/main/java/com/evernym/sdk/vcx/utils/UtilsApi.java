package com.evernym.sdk.vcx.utils;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java9.util.concurrent.CompletableFuture;


/**
 * Created by abdussami on 17/05/18.
 */

public class UtilsApi extends VcxJava.API {
    private static final Logger logger = LoggerFactory.getLogger("UtilsApi");
    private static Callback provAsyncCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String config) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], config = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;

            String result = config;
            future.complete(result);
        }
    };

    /**
     * Provision an agent in the agency, populate configuration and wallet for this agent.
     *
     * @param  config         Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
     *
     * @return                populated config that can be used for library initialization.
     */
    public static String vcxProvisionAgent(String config) {
        ParamGuard.notNullOrWhiteSpace(config, "config");
        logger.debug("vcxProvisionAgent() called with: config = [****]");
        String result = LibVcx.api.vcx_provision_agent(config);

        return result;

    }

    /**
     * Provision an agent in the agency, populate configuration and wallet for this agent.
     *
     * @param  conf           Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
     *
     * @return                populated config that can be used for library initialization.
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxAgentProvisionAsync(String conf) throws VcxException {
        CompletableFuture<String> future = new CompletableFuture<String>();
        logger.debug("vcxAgentProvisionAsync() called with: conf = [****]");
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_agent_provision_async(
                commandHandle, conf,
                provAsyncCB);
        checkResult(result);
        return future;
    }

    /**
     * Provision an agent in the agency, populate configuration and wallet for this agent.
     *
     * @param  config         Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
     * @param  token          provisioning token.
     *      {
     *          This can be a push notification endpoint to contact the sponsee or
     *          an id that the sponsor uses to reference the sponsee in its backend system
     *          "sponseeId": String,
     *          "sponsorId": String, //Persistent Id of the Enterprise sponsoring the provisioning
     *          "nonce": String,
     *          "timestamp": String,
     *          "sig": String, // Base64Encoded(sig(nonce + timestamp + id))
     *          "sponsorVerKey": String,
     *          "attestationAlgorithm": Optional[String], // device attestation signature algorithm. Can be one of: SafetyNet | DeviceCheck
     *          "attestationData": Optional[String], // device attestation signature matching to specified algorithm
     *        }
     *
     * @return                populated config that can be used for library initialization.
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     *
     **/
      public static String vcxAgentProvisionWithToken(String config, String token) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(config, "config");
        ParamGuard.notNullOrWhiteSpace(token, "token");
        logger.debug("vcxAgentProvisionWithToken() called with: config = [****], token = [***]");

        String result = LibVcx.api.vcx_provision_agent_with_token(config, token);

        return result;
    }

    /**
     * Provision an agent in the agency, populate configuration and wallet for this agent.
     *
     * @param  conf           Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
     *
     * @return                populated config that can be used for library initialization.
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxAgentProvisionWithTokenAsync(String config, String token) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(config, "config");
        ParamGuard.notNullOrWhiteSpace(token, "token");

        CompletableFuture<String> future = new CompletableFuture<String>();
        logger.debug("vcxAgentProvisionWithTokenAsync() called with: config = [****], token = [****]");
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_provision_agent_with_token_async(
                commandHandle,
                config,
                token,
                provAsyncCB);
        checkResult(result);
        return future;
    }

    private static Callback vcxGetProvisionTokenCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String token) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], token = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;

            String result = token;
            future.complete(result);
        }
    };

    /**
     * Get token that can be used for provisioning an agent
     * NOTE: Can be used only for Evernym's applications
     *
     * @param  config           provisioning configuration.
     * {
     *     vcx_config: VcxConfig // Same config passed to agent provision
     *                           // See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
     *     sponsee_id: String,
     *     sponsor_id: String,
     * }
     *
     * @return                provisioning token as JSON
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     *
     **/
    public static CompletableFuture<String> vcxGetProvisionToken(String config) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(config, "config");
        logger.debug("vcxGetProvisionToken() called with: config = [****]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_get_provision_token(
                commandHandle,
                config,
                vcxGetProvisionTokenCB
        );
        checkResult(result);
        return future;
    }

    private static Callback vcxUpdateAgentInfoCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = commandHandle;
            future.complete(null);
        }
    };

    /**
     * Update information on the agent (ie, comm method and type)
     *
     * @param  config         New agent updated configuration as JSON
     *                        "{"id":"123","value":"value"}"
     *
     * @return                void
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<Void> vcxUpdateAgentInfo(String config) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(config, "config");
        logger.debug("vcxUpdateAgentInfo() called with: config = [****]");
        CompletableFuture<Void> future = new CompletableFuture<Void>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_agent_update_info(
                commandHandle,
                config,
                vcxUpdateAgentInfoCB
        );
        checkResult(result);
        return future;
    }

    private static Callback vcxGetMessagesCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String messages) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], messages = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            String result = messages;
            future.complete(result);
        }
    };

    /**
     * Retrieve messages from the agent
     *
     * @param  messageStatus  optional, comma separated - query for messages with the specified status.
     *                             Statuses:
     *                                  MS-101 - Created
     *                                  MS-102 - Sent
     *                                  MS-103 - Received
     *                                  MS-104 - Accepted
     *                                  MS-105 - Rejected
     *                                  MS-106 - Reviewed
     *                        "MS-103,MS-106"
     * @param  uids           optional, comma separated - query for messages with the specified uids
     *                        "s82g63,a2h587"
     * @param  pwdids         optional, comma separated - DID's pointing to specific connection
     *                        "did1,did2"
     *
     * @return                The list of all found messages
     *                        "[{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}]"
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxGetMessages(String messageStatus, String uids, String pwdids) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(messageStatus, "messageStatus");
        logger.debug("vcxGetMessages() called with: messageStatus = [" + messageStatus + "], uids = [" + uids + "], pwdids = [****]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_messages_download(
                commandHandle,
                messageStatus,
                uids,
                pwdids,
                vcxGetMessagesCB
        );
        checkResult(result);
        return future;
    }

    /**
     * Retrieves single message from the agency by the given uid.
     *
     * @param  uid  id of the message to query.
     *
     * @return                Received message:
     *                        "{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}"
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxGetMessage(String uid) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(uid, "uid");
        logger.debug("vcxGetMessage() called with: uid = [" + uid + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_download_message(
                commandHandle,
                uid,
                vcxGetMessagesCB
        );
        checkResult(result);
        return future;
    }

    /**
     * Retrieve messages from the Cloud Agent
     *
     * @param  messageStatus  optional, comma separated - query for messages with the specified status.
     *                             Statuses:
     *                                  MS-101 - Created
     *                                  MS-102 - Sent
     *                                  MS-103 - Received
     *                                  MS-104 - Accepted
     *                                  MS-105 - Rejected
     *                                  MS-106 - Reviewed
     *                        "MS-103,MS-106"
     * @param  uids           optional, comma separated - query for messages with the specified uids
     *                        "s82g63,a2h587"
     *
     * @return                The list of all found messages
     *                        "[{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}]"
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxGetAgentMessages(String messageStatus, String uids) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(messageStatus, "messageStatus");
        logger.debug("vcxGetAgentMessages() called with: messageStatus = [" + messageStatus + "], uids = [" + uids + "]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_download_agent_messages(
                commandHandle,
                messageStatus,
                uids,
                vcxGetMessagesCB
        );
        checkResult(result);
        return future;
    }

    private static Callback vcxUpdateMessagesCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(null);
        }
    };

    /**
     * Update the status of messages from the specified connection
     *
     * @param  messageStatus  message status to set
     *                             Statuses:
     *                                  MS-101 - Created
     *                                  MS-102 - Sent
     *                                  MS-103 - Received
     *                                  MS-104 - Accepted
     *                                  MS-105 - Rejected
     *                                  MS-106 - Reviewed
     *                        "MS-103,MS-106"
     * @param  msgJson        list of messages to update
     *                        [{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]},...]
     *
     * @return               void
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<Void> vcxUpdateMessages(String messageStatus, String msgJson) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(messageStatus, "messageStatus");
        ParamGuard.notNull(msgJson, "msgJson");
        logger.debug("vcxUpdateMessages() called with: messageStatus = [" + messageStatus + "], msgJson = [****]");
        CompletableFuture<Void> future = new CompletableFuture<Void>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_messages_update_status(
                commandHandle,
                messageStatus,
                msgJson,
                vcxUpdateMessagesCB
        );
        checkResult(result);
        return future;
    }

    private static Callback stringCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String fees) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], fees = [" + fees + "]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            String result = fees;
            future.complete(result);
        }
    };

    /**
     * Get ledger fees from the network
     *
     * @return                the fee structure for the sovrin network
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> getLedgerFees() throws VcxException {
        logger.debug("getLedgerFees() called");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_ledger_get_fees(
                commandHandle,
                stringCB
        );
        checkResult(result);
        return future;
    }

    public static void vcxMockSetAgencyResponse(int messageIndex) {
        logger.debug("vcxMockSetAgencyResponse() called");
        LibVcx.api.vcx_set_next_agency_response(messageIndex);
    }

    private static Callback getReqPriceAsyncCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, long price) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], price = [" + price + "]");
            CompletableFuture<Long> future = (CompletableFuture<Long>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;

            long result = price;
            future.complete(result);
        }
    };

    /**
     * Gets minimal request price for performing an action in case the requester can perform this action.
     *
     * @param  actionJson       definition of action to get price
     *                          {
     *                              "auth_type": ledger transaction alias or associated value,
     *                              "auth_action": type of an action.,
     *                              "field": transaction field,
     *                              "old_value": (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action),
     *                              "new_value": (Optional) new value that can be used to fill the field,
     *                          }
     * @param  requesterInfoJson  (Optional) request definition ( otherwise context info will be used).
     *                          {
     *                              "role": string - role of a user which can sign transaction.
     *                              "count": string - count of users.
     *                              "is_owner": bool - if user is an owner of transaction.
     *                          }
     *
     * @return                 price must be paid to perform the requested action
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<Long> vcxGetRequestPrice(String actionJson, String requesterInfoJson) throws VcxException {
        ParamGuard.notNull(actionJson, "actionJson");
        logger.debug("vcxGetRequestPrice() called with: actionJson = [" + actionJson + "], requesterInfoJson = [" + requesterInfoJson + "]");
        CompletableFuture<Long> future = new CompletableFuture<Long>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_get_request_price(
                commandHandle, actionJson, requesterInfoJson,
                getReqPriceAsyncCB);
        checkResult(result);
        return future;
    }

    private static Callback vcxEndorseTransactionCb = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(null);
        }
    };

    /**
     * Endorse transaction to the ledger preserving an original author
     *
     * @param  transactionJson  transaction to endorse
     *
     * @return                  void
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<Void> vcxEndorseTransaction(String transactionJson) throws VcxException {
        ParamGuard.notNull(transactionJson, "transactionJson");
        logger.debug("vcxEndorseTransaction() called with: transactionJson = [****]");
        CompletableFuture<Void> future = new CompletableFuture<Void>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_endorse_transaction(
                commandHandle, transactionJson,
                vcxEndorseTransactionCb);
        checkResult(result);
        return future;
    }

    private static Callback vcxFetchPublicEntitiesCb = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(null);
        }
    };

    /**
     * Fetch and Cache public entities from the Ledger associated with stored in the wallet credentials.
     * This function performs two steps:
     *     1) Retrieves the list of all credentials stored in the opened wallet.
     *     2) Fetch and cache Schemas / Credential Definitions / Revocation Registry Definitions
     *        correspondent to received credentials from the connected Ledger.
     *
     * This helper function can be used, for instance as a background task, to refresh library cache.
     * This allows us to reduce the time taken for Proof generation by using already cached entities instead of queering the Ledger.
     *
     * NOTE: Library must be already initialized (wallet and pool must be opened).
     *
     * @return                  void
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<Void> vcxFetchPublicEntities() throws VcxException {
        logger.debug("vcxFetchPublicEntities() called");
        CompletableFuture<Void> future = new CompletableFuture<Void>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_fetch_public_entities(
                commandHandle,
                vcxFetchPublicEntitiesCb);
        checkResult(result);
        return future;
    }


    private static Callback vcxHealthCheckCb = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "]");
            CompletableFuture<Void> future = (CompletableFuture<Void>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(null);
        }
    };

    /**
     * This function allows you to check the health of LibVCX and EAS/CAS instance.
     * It will return error in case of any problems on EAS or will resolve pretty long if VCX is thread-hungry.
     * WARNING: this call may take a lot of time returning answer in case of load, be careful.
     * NOTE: Library must be initialized, ENDPOINT_URL should be set
     * @return                  void
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<Void> vcxHealthCheck() throws VcxException {
        logger.debug("vcxHealthCheck() called");
        CompletableFuture<Void> future = new CompletableFuture<Void>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_health_check(
                commandHandle,
                vcxFetchPublicEntitiesCb);
        checkResult(result);
        return future;
    }

    private static Callback vcxCreatePairwiseAgentCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String agentInfo) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], agentInfo = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            String result = agentInfo;
            future.complete(result);
        }
    };

    /**
     * Create pairwise agent which can be later used for connection establishing.
     *
     * You can pass `agent_info` into `vcx_connection_connect` function as field of `connection_options` JSON parameter.
     * The passed Pairwise Agent will be used for connection establishing instead of creation a new one.
     *
     * @return   Agent info as JSON string:
     *     {
     *         "pw_did": string,
     *         "pw_vk": string,
     *         "agent_did": string,
     *         "agent_vk": string,
     *     }
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxCreatePairwiseAgent() throws VcxException {
        logger.debug("vcxCreatePairwiseAgent() called");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_create_pairwise_agent(
                commandHandle,
                vcxCreatePairwiseAgentCB
        );
        checkResult(result);
        return future;
    }

    /**
     * Extract content of Aries message containing attachment decorator.
     * RFC: https://github.com/hyperledger/aries-rfcs/tree/main/features/0592-indy-attachments
     *
     * @param  message        Aries message containing attachment decorator
     *
     * @return                Attached message as JSON string
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxExtractAttachedMessage(String message) throws VcxException {
        ParamGuard.notNull(message, "message");
        logger.debug("vcxExtractAttachedMessage() called with: message = [****]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_extract_attached_message(
                commandHandle,
                message,
                stringCB
        );
        checkResult(result);
        return future;
    }

    /**
     * Extract thread id for message.
     *
     * @param  message        Message to get thread id from
     *
     * @return                Thread id
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxExtractThreadId(String message) throws VcxException {
        ParamGuard.notNull(message, "message");
        logger.debug("extractThreadId() called with: message = [****]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_extract_thread_id(
                commandHandle,
                message,
                stringCB
        );
        checkResult(result);
        return future;
    }

    /**
     * Resolve message by the given URL.
     * Supported cases:
     *   1. Message inside of query parameters (c_i, oob, d_m, m) as base64 encoded string
     *   2. Message inside response `location` header for GET request
     *   3. Message inside response for GET request
     *
     * @param  url            url to fetch message
     *
     * @return                Resolved message as JSON string
     *
     * @throws VcxException   If an exception occurred in Libvcx library.
     */
    public static CompletableFuture<String> vcxResolveMessageByUrl(String url) throws VcxException {
        ParamGuard.notNull(url, "url");
        logger.debug("vcxResolveMessageByUrl() called with: url = [****]");
        CompletableFuture<String> future = new CompletableFuture<String>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_resolve_message_by_url(
                commandHandle,
                url,
                stringCB
        );
        checkResult(result);
        return future;
    }
}
