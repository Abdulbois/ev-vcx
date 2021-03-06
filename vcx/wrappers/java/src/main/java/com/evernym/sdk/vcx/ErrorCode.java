package com.evernym.sdk.vcx;

import java.util.HashMap;
import java.util.Map;

/**
 * Enumeration of error codes returned by the vcx SDK.
 */
public enum ErrorCode {

    SUCCESS(0),
    UNKNOWN_ERROR(1001),
    INVALID_CONNECTION_HANDLE(1003),
    INVALID_CONFIGURATION(1004),
    NOT_READY(1005),
    INVALID_OPTION(1007),
    INVALID_DID(1008),
    INVALID_VERKEY(1009),
    POST_MSG_FAILURE(1010),
    INVALID_NONCE(1011),
    INVALID_URL(1013),
    NOT_BASE58(1014),
    INVALID_ISSUER_CREDENTIAL_HANDLE(1015),
    INVALID_JSON(1016),
    INVALID_PROOF_HANDLE(1017),
    INVALID_CREDENTIAL_REQUEST(1018),
    INVALID_MSGPACK(1019),
    INVALID_AGENCY_RESPONSE(1020),
    INVALID_ATTRIBUTES_STRUCTURE(1021),
    BIG_NUMBER_ERROR(1022),
    INVALID_PROOF(1023),
    INVALID_GENESIS_TXN_PATH(1024),
    POOL_LEDGER_CONNECT(1025),
    CREATE_POOL_CONFIG(1026),
    INVALID_PROOF_CREDENTIAL_DATA(1027),
    INVALID_PREDICATES_STRUCTURE(1028),
    INVALID_AGENCY_REQUEST(1029),
    NO_POOL_OPEN(1030),
    INVALID_SCHEMA(1031),
    CREATE_CREDENTIAL_DEF_ERR(1034),
    UNKNOWN_LIBINDY_ERROR(1035),
    CREDENTIAL_DEFINITION_NOT_FOUND(1036),
    INVALID_CREDENTIAL_DEF_HANDLE(1037),
    TIMEOUT_LIBINDY_ERROR(1038),
    CREDENTIAL_DEF_ALREADY_CREATED(1039),
    INVALID_SCHEMA_SEQ_NO(1040),
    INVALID_SCHEMA_CREATION(1041),
    INVALID_SCHEMA_HANDLE(1042),
    INVALID_CREDENTIAL_OFFER(1043),
    ALREADY_INITIALIZED(1044),
    INVALID_INVITE_DETAILS(1045),
    INVALID_OBJ_HANDLE(1048),
    INVALID_DISCLOSED_PROOF_HANDLE(1049),
    SERIALIZATION_ERROR(1050),
    WALLET_ALREADY_EXISTS(1051),
    WALLET_ALREADY_OPEN(1052),
    INVALID_CREDENTIAL_HANDLE(1053),
    INVALID_CREDENTIAL_JSON(1054),
    WALLET_ITEM_NOT_FOUND(212),
    WALLET_ITEM_CANNOT_ADD(213),
    CREATE_PROOF_ERROR(1056),
    INVALID_WALLET_HANDLE(1057),
    INVALID_WALLET_CREATION(1058),
    CANNOT_DELETE_CONNECTION(1058),
    CREATE_CONNECTION_ERROR(1061),
    CONNECTION_ALREADY_EXISTS(1062),
    CONNECTION_DOES_NOT_EXIST(1063),
    INSUFFICIENT_TOKEN_AMOUNT(1064),
    CONNECTION_NOT_READY_TO_UPGRADE(1065),
    INVALID_PAYMENT_ADDRESS(1066),
    INVALID_LIBINDY_PARAM(1067),
    MISSING_WALLET_KEY(1069),
    OBJECT_CACHE_ERROR(1070),
    NO_PAYMENT_INFORMATION(1071),
    DUPLICATE_WALLET_RECORD(1072),
    WALLET_RECORD_NOT_FOUND(1073),
    IOERROR(1074),
    WALLET_ACCESS_FAILED(1075),
    INVALID_WALLET_IMPORT_CONFIG(1076),
    MISSING_BACKUP_KEY(1078),
    WALLET_NOT_FOUND(1079),
    LIBINDY_INVALID_STRUCTURE(1080),
    INVALID_STATE(1081),
    INVALID_LEDGER_RESPONSE(1082),
    DID_ALREADY_EXISTS_IN_WALLET(1083),
    DUPLICATE_MASTER_SECRET(1084),
    INVALID_PROOF_REQUEST(1086),
    MISSING_PAYMENT_METHOD(1087),
    DUPLICATE_SCHEMA(1088),
    LOGGING_ERROR(1090),
    INVALID_REVOCATION_DETAILS(1091),
    INVALID_REV_ENTRY(1092),
    INVALID_REVOCATION_TIMESTAMP(1093),
    UNKNOWN_SCHEMA_REJECTION(1094),
    INVALID_REV_REG_DEF_CREATION(1095),
    CREATE_WALLET_BACKUP(1096),
    RETRIEVE_EXPORTED_WALLET(1097),
    RETRIEVE_DEAD_DROP(1099),
    INVALID_ATTACHMENT_ENCODING(1100),
    ACTION_NOT_SUPPORTED(1103),
    INVALID_REDIRECT_DETAILS(1104),
    MAX_BACKUP_SIZE(1105),
    NO_AGENT_INFO(1106),
    INVALID_PROVISION_TOKEN(1107),
    INVALID_DID_DOC(1108),
    MESSAGE_IS_OUT_OF_THREAD(1109),
    INVALID_PROOF_PROPOSAL(1110),
    INCOMPATIBLE_PARAMETERS(1111),
    CONNECTION_NOT_COMPLETED(1112),
    UNIDENTIFIED_ERROR_CODE(9999); //Wrapper expects to never receive 9999 from libindy. If libindy ever reaches this number in error codes, please increment number in UNIDENTIFIED_ERROR_CODE(<new bigger number>)

    private int value;
    private static Map<Integer, ErrorCode> map = new HashMap<Integer, ErrorCode>();

    private ErrorCode(int value) {

        this.value = value;
    }

    static {

        for (ErrorCode errorCode : ErrorCode.values()) {

            map.put(errorCode.value, errorCode);
        }
    }

    /**
     * Gets the ErrorCode that corresponds to the specified int value.
     *
     * @param value The integer to get the error code for.
     * @return The ErrorCode that corresponds to the specified integer.
     */
    public static ErrorCode valueOf(int value) {
        if(map.containsKey(value)){
            return map.get(value);
        }else{
            return UNIDENTIFIED_ERROR_CODE;
        }

    }

    /**
     * Gets the integer value for a specific ErrorCode.
     *
     * @return The integer value of the ErrorCode.
     */
    public int value() {

        return this.value;
    }
}
