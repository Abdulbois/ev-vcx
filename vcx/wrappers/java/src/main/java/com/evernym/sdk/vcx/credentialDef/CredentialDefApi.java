package com.evernym.sdk.vcx.credentialDef;

import com.evernym.sdk.vcx.LibVcx;
import com.evernym.sdk.vcx.ParamGuard;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.VcxJava;
import com.sun.jna.Callback;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java9.util.concurrent.CompletableFuture;

public class CredentialDefApi extends VcxJava.API {

    private static final Logger logger = LoggerFactory.getLogger("CredentialDefApi");
    private static Callback credentialDefCreateCB = new Callback() {
        // TODO: This callback and jna definition needs to be fixed for this API
        // it should accept connection handle as well
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int credentialDefHandle) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], credentialDefHandle = [" + credentialDefHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = credentialDefHandle;
            future.complete(result);
        }
    };

	/**
	 * Create a new CredentialDefinition object and publish correspondent record on the ledger
	 *
	 * @param  sourceId             enterprise's personal identification for the CredentialDefinition.
	 * @param  credentialName       name of credential definition
	 * @param  schemaId             id of a Schema to use for creating Credential Definition.
	 * @param  issuerId             did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
	 * @param  tag                  way to create a unique credential def with the same schema and issuer did.
	 * @param  config               type-specific configuration of credential definition revocation
	 *                              {
	 *                                  support_revocation: true|false - Optional, by default its false
	 *                                  tails_file: path to tails file - Optional if support_revocation is false
	 *                                  max_creds: size of tails file - Optional if support_revocation is false
	 *                              }
	 * @param  paymentHandle        unused parameter (pass 0)
	 *
	 * @return                      handle that should be used to perform actions with the CredentialDefinition object.
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<Integer> credentialDefCreate(String sourceId,
                                                                 String credentialName,
                                                                 String schemaId,
                                                                 String issuerId,
                                                                 String tag,
                                                                 String config,
                                                                 int paymentHandle
    ) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(sourceId, "sourceId");
        ParamGuard.notNullOrWhiteSpace(credentialName, "credentialName");
        ParamGuard.notNullOrWhiteSpace(schemaId, "schemaId");
        logger.debug("credentialDefCreate() called with: sourceId = [" + sourceId + "], credentialName = [" + credentialName + "], schemaId = [****], issuerId = [****], tag = [****], config = [****], paymentHandle = [" + paymentHandle + "]");
        //TODO: Check for more mandatory params in vcx to add in PamaGuard
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credentialdef_create(
                commandHandle,
                sourceId,
                credentialName,
                schemaId,
                issuerId,
                tag,
                config,
                paymentHandle,
                credentialDefCreateCB
        );
        checkResult(result);
        return future;
    }

    private static Callback credentialDefCreateWithIdCB = new Callback() {
        // TODO: This callback and jna definition needs to be fixed for this API
        // it should accept connection handle as well
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int credentialDefHandle) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], credentialDefHandle = [" + credentialDefHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = credentialDefHandle;
            future.complete(result);
        }
    };

	/**
	 * Create a new CredentialDefinition object from the given credDefId
	 *
	 * @param  sourceId             enterprise's personal identification for the CredentialDefinition.
	 * @param  credDefId            reference to already created cred def
	 * @param  issuerDid            did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
	 * @param  revocationConfig     type-specific configuration of credential definition revocation
	 *                              {
	 *                                  support_revocation: true|false - Optional, by default its false
	 *                                  tails_file: path to tails file - Optional if support_revocation is false
	 *                                  max_creds: size of tails file - Optional if support_revocation is false
	 *                              }
	 *
	 * @return                      handle that should be used to perform actions with the CredentialDefinition object.
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<Integer> credentialDefCreateWithId(String sourceId,
                                                                       String credDefId,
                                                                       String issuerDid,
                                                                       String revocationConfig
    ) throws VcxException {
        ParamGuard.notNullOrWhiteSpace(sourceId, "sourceId");
        ParamGuard.notNullOrWhiteSpace(credDefId, "credDefId");
        logger.debug("credentialDefCreateWithId() called with: sourceId = [" + sourceId + "], credDefId = [****], issuerId = [****], revocationConfig = [****]");
        //TODO: Check for more mandatory params in vcx to add in PamaGuard
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credentialdef_create_with_id(
                commandHandle,
                sourceId,
                credDefId,
                issuerDid,
                revocationConfig,
                credentialDefCreateWithIdCB
        );
        checkResult(result);
        return future;
    }

    private static Callback credentialDefSerializeCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String serializedData) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], serializedData = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            String result = serializedData;
            future.complete(result);
        }
    };

	/**
	 * Get JSON string representation of CredentialDefinition object.
	 *
	 * @param  credentialDefHandle     handle pointing to a CredentialDefinition object.
	 *
	 * @return                         CredentialDefinition object as JSON string.
	 *
	 * @throws VcxException            If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<String> credentialDefSerialize(int credentialDefHandle) throws VcxException {
        ParamGuard.notNull(credentialDefHandle, "credentialDefHandle");
        logger.debug("credentialDefSerialize() called with: credentialDefHandle = [" + credentialDefHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credentialdef_serialize(
                commandHandle,
                credentialDefHandle,
                credentialDefSerializeCB
        );
        checkResult(result);
        return future;
    }

    private static Callback credentialDefDeserialize = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, int credntialDefHandle) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], credntialDefHandle = [" + credntialDefHandle + "]");
            CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            Integer result = credntialDefHandle;
            future.complete(result);
        }
    };

	/**
	 * Takes a json string representing a CredentialDefinition object and recreates an object matching the JSON.
	 *
	 * @param  credentialDefData    JSON string representing a CredentialDefinition object.
	 *
	 * @return                      handle that should be used to perform actions with the CredentialDefinition object.
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<Integer> credentialDefDeserialize(String credentialDefData) throws VcxException {
        ParamGuard.notNull(credentialDefData, "credentialDefData");
        logger.debug("credentialDefSerialize() called with: credentialDefData = [****]");
        CompletableFuture<Integer> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);

        int result = LibVcx.api.vcx_credentialdef_deserialize(
                commandHandle,
                credentialDefData,
                credentialDefDeserialize
        );
        checkResult(result);
        return future;
    }


    private static Callback credentialDefGetCredentialDefIdCb = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int commandHandle, int err, String credentialDefId) {
            logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], credentialDefId = [****]");
            CompletableFuture<String> future = (CompletableFuture<String>) removeFuture(commandHandle);
            if (!checkCallback(future, err)) return;
            future.complete(credentialDefId);
        }
    };

	/**
	 * Retrieves credential definition's id
	 *
	 * @param  credDefHandle     handle pointing to a CredentialDefinition object.
	 *
	 * @return                   id of the CredentialDefinition object.
	 *
	 * @throws VcxException      If an exception occurred in Libvcx library.
	 */
    public static CompletableFuture<String> credentialDefGetCredentialDefId(int credDefHandle) throws VcxException {
        ParamGuard.notNull(credDefHandle, "credDefHandle");
        logger.debug("credentialDefGetCredentialDefId() called with: credDefHandle = [" + credDefHandle + "]");
        CompletableFuture<String> future = new CompletableFuture<>();
        int commandHandle = addFuture(future);
        int result = LibVcx.api.vcx_credentialdef_get_cred_def_id(commandHandle,credDefHandle, credentialDefGetCredentialDefIdCb);
        checkResult(result);
        return future;
    }

	/**
	 * Releases the CredentialDefinition object by de-allocating memory
	 *
	 * @param  handle               handle pointing to a CredentialDefinition object.
	 *
	 * @return                      void
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
    public static int credentialDefRelease(int handle) throws VcxException {
        ParamGuard.notNull(handle, "handle");
        logger.debug("credentialDefRelease() called with: handle = [" + handle + "]");

        int result = LibVcx.api.vcx_credentialdef_release(handle);
        checkResult(result);

        return result;
    }

    private static Callback credentialDefPrepareForEndorserCB = new Callback() {
        @SuppressWarnings({"unused", "unchecked"})
        public void callback(int command_handle, int err, int handle, String credentialDefTxn, String revocRegDefTxn, String revocRegEntryTxn) {
	        System.out.println("callback() called with: command_handle = [" + command_handle + "], err = [" + err + "], handle = [" + handle + "], credentialDefTxn = [****], revocRegDefTxn = [****], revocRegEntryTxn = [****]");
            CompletableFuture<CredentialDefPrepareForEndorserResult> future = (CompletableFuture<CredentialDefPrepareForEndorserResult>) removeFuture(command_handle);
            if (!checkCallback(future, err)) return;
	        CredentialDefPrepareForEndorserResult result = new CredentialDefPrepareForEndorserResult(handle, credentialDefTxn, revocRegDefTxn, revocRegEntryTxn);
            future.complete(result);
        }
    };

	/**
	 * Create a new CredentialDef object that will be published by Endorser later.
	 * <p>
	 * Note that CredentialDef can't be used for credential issuing until it will be published on the ledger.
	 *
	 * @param  sourceId             enterprise's personal identification for the CredentialDefinition.
	 * @param  credentialName       name of credential definition
	 * @param  schemaId             id of a Schema to use for creating Credential Definition.
	 * @param  issuerId             did corresponding to entity issuing a credential. Needs to have Trust Anchor permissions on ledger
	 * @param  tag                  way to create a unique credential def with the same schema and issuer did.
	 * @param  config               type-specific configuration of credential definition revocation
	 *                              {
	 *                                  support_revocation: true|false - Optional, by default its false
	 *                                  tails_file: path to tails file - Optional if support_revocation is false
	 *                                  max_creds: size of tails file - Optional if support_revocation is false
	 *                              }
	 * @param  endorser             DID of the Endorser that will submit the transaction.
	 *
	 * @return                      handle that should be used to perform actions with the CredentialDefinition object.
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<CredentialDefPrepareForEndorserResult> credentialDefPrepareForEndorser(String sourceId,
	                                                                                                       String credentialName,
	                                                                                                       String schemaId,
	                                                                                                       String issuerId,
	                                                                                                       String tag,
	                                                                                                       String config,
	                                                                                                       String endorser
	) throws VcxException {
		ParamGuard.notNullOrWhiteSpace(sourceId, "sourceId");
		ParamGuard.notNull(credentialName, "credentialName");
		ParamGuard.notNull(schemaId, "schemaId");
		ParamGuard.notNull(endorser, "endorser");
		logger.debug("credentialDefCreate() called with: sourceId = [" + sourceId + "], credentialName = [****], schemaId = [****], issuerId = [****], tag = [****], config = [****], endorser = [****]");
		CompletableFuture<CredentialDefPrepareForEndorserResult> future = new CompletableFuture<CredentialDefPrepareForEndorserResult>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_credentialdef_prepare_for_endorser(
				commandHandle,
				sourceId,
				credentialName,
				schemaId,
				issuerId,
				tag,
				config,
				endorser,
				credentialDefPrepareForEndorserCB);
		checkResult(result);

		return future;
	}

	private static Callback vcxIntegerCB = new Callback() {
		@SuppressWarnings({"unused", "unchecked"})
		public void callback(int commandHandle, int err, int s) {
			logger.debug("callback() called with: commandHandle = [" + commandHandle + "], err = [" + err + "], s = [" + s + "]");
			CompletableFuture<Integer> future = (CompletableFuture<Integer>) removeFuture(commandHandle);
			if (! checkCallback(future, err)) return;
			Integer result = s;
			future.complete(result);
		}
	};

	/**
	 * Checks if CredentialDefinition is published on the Ledger and updates the state.
	 *
	 * @param  handle               handle pointing to a CredentialDefinition object.
	 *
	 * @return                      the most current state of CredentialDefinition object.
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> credentialDefUpdateState(int handle) throws VcxException {
		logger.debug("vcxSchemaUpdateState() called with: handle = [" + handle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_credentialdef_update_state(
				commandHandle,
				handle,
				vcxIntegerCB
		);
		checkResult(result);
		return future;
	}

	/**
	 * Get the current state of the CredentialDefinition object
	 * Schema states:
	 *     0 - Built
	 *     1 - Published
	 *
	 * @param  handle               handle pointing to a CredentialDefinition object.
	 *
	 * @return                      the most current state of the CredentialDefinition object.
	 *
	 * @throws VcxException         If an exception occurred in Libvcx library.
	 */
	public static CompletableFuture<Integer> credentialDefGetState(int handle) throws VcxException {
		logger.debug("schemaGetState() called with: handle = [" + handle + "]");
		CompletableFuture<Integer> future = new CompletableFuture<>();
		int commandHandle = addFuture(future);

		int result = LibVcx.api.vcx_credentialdef_get_state(
				commandHandle,
				handle,
				vcxIntegerCB
		);
		checkResult(result);
		return future;
	}
}
