import json
from ctypes import *
import logging
from typing import Optional

from vcx.common import do_call, create_cb, do_call_sync


async def vcx_agent_provision(config: str) -> None:
    """
    Provision an agent in the agency, populate configuration and wallet for this agent.

   Params:
     config - Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options

    Example:
    import json
    enterprise_config = {
        'agency_url': 'http://localhost:8080',
        'agency_did': 'VsKV7grR1BUE29mG2Fm2kX',
        'agency_verkey': "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
        'wallet_name': 'LIBVCX_SDK_WALLET',
        'agent_seed': '00000000000000000000000001234561',
        'enterprise_seed': '000000000000000000000000Trustee1',
        'wallet_key': '1234'
    }
    vcx_config = await vcx_agent_provision(json.dumps(enterprise_config))
    :param config: JSON configuration
    :return: Configuration for vcx_init call.
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_agent_provision, "cb"):
        logger.debug("vcx_agent_provision: Creating callback")
        vcx_agent_provision.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    c_config = c_char_p(config.encode('utf-8'))

    result = await do_call('vcx_agent_provision_async',
                           c_config,
                           vcx_agent_provision.cb)

    logger.debug("vcx_agent_provision completed")
    return result.decode()

async def vcx_provision_agent_with_token(config: str, token: str) -> str:
    """
    Provision an agent in the agency, populate configuration and wallet for this agent.

    Params:
     config - Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
     token  -  Provisioning token provided by sponsor

    Example:
    enterprise_config = {
        'agency_url': 'http://localhost:8080',
        'agency_did': 'VsKV7grR1BUE29mG2Fm2kX',
        'agency_verkey': "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
        'wallet_name': 'LIBVCX_SDK_WALLET',
        'agent_seed': '00000000000000000000000001234561',
        'enterprise_seed': '000000000000000000000000Trustee1',
        'wallet_key': '1234'
    }
    token = {
      "id": String,
      "sponsor": String, //Name of Enterprise sponsoring the provisioning
      "nonce": String,
      "timestamp": String,
      "sig": String, // Base64Encoded(sig(nonce + timestamp + id))
      "sponsor_vk": String,
    }

    :return: Configuration for vcx_init call.
    """
    logger = logging.getLogger(__name__)

    c_config = c_char_p(config.encode('utf-8'))
    c_token = c_char_p(token.encode('utf-8'))

    c_result = do_call_sync('vcx_provision_agent_with_token',
                          c_config,
                          c_token)
    result = cast(c_result, c_char_p).value
    logger.debug("vcx_provision_agent_with_token completed")
    return result.decode()

async def vcx_provision_agent_with_token_async(config: str, token: str) -> str:
    """
    Provision an agent in the agency, populate configuration and wallet for this agent.

    Params:
     config - Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
     token  -  Provisioning token provided by sponsor

    Example:
    enterprise_config = {
        'agency_url': 'http://localhost:8080',
        'agency_did': 'VsKV7grR1BUE29mG2Fm2kX',
        'agency_verkey': "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
        'wallet_name': 'LIBVCX_SDK_WALLET',
        'agent_seed': '00000000000000000000000001234561',
        'enterprise_seed': '000000000000000000000000Trustee1',
        'wallet_key': '1234'
    }
    token = {
      "id": String,
      "sponsor": String, //Name of Enterprise sponsoring the provisioning
      "nonce": String,
      "timestamp": String,
      "sig": String, // Base64Encoded(sig(nonce + timestamp + id))
      "sponsor_vk": String,
    }

    :return: Configuration for vcx_init call.
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_provision_agent_with_token_async, "cb"):
        logger.debug("vcx_provision_agent_with_token_async: Creating callback")
        vcx_provision_agent_with_token_async.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    c_config = c_char_p(config.encode('utf-8'))
    c_token = c_char_p(token.encode('utf-8'))

    result = await do_call('vcx_provision_agent_with_token_async',
                           c_config,
                           c_token,
                           vcx_provision_agent_with_token_async.cb)

    logger.debug("vcx_provision_agent_with_token_async completed")
    return result.decode()

async def vcx_get_provision_token(config: str) -> str:
    """
    Get token which can be used for provisioning an agent
    NOTE: Can be used only for Evernym's applications

    :param config:
     {
         vcx_config: VcxConfig // Same config passed to agent provision
                               // See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
         sponsee_id: String,
         sponsor_id: String,
         algorithm: Optional[String], // signature algorithm. Can be one of: SafetyNet | DeviceCheck
         signature: Optional[String], // signature matching to specified algorithm
     }
    :return: provisioning token as JSON
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_get_provision_token, "cb"):
        logger.debug("vcx_agent_update_info: Creating callback")
        vcx_get_provision_token.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    c_config = c_char_p(config.encode('utf-8'))

    result = await do_call('vcx_get_provision_token',
                           c_config,
                           vcx_get_provision_token.cb)

    logger.debug("vcx_get_provision_token completed")
    return result.decode()

async def vcx_agent_update_info(config: str) -> None:
    """
    Update information on the agent (ie, comm method and type)
    :param config: updated configuration
                   {
                       "id": "string", 1 means push notifications, its the only one registered
                       "type": Optional(int), notifications type (1 is used by default).
                       "value": "string",
                   }
    :return:
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_agent_update_info, "cb"):
        logger.debug("vcx_agent_update_info: Creating callback")
        vcx_agent_update_info.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

    c_config = c_char_p(config.encode('utf-8'))

    result = await do_call('vcx_agent_update_info',
                           c_config,
                           vcx_agent_update_info.cb)

    logger.debug("vcx_agent_update_info completed")
    return result


async def vcx_ledger_get_fees() -> str:
    """
    Get ledger fees from the sovrin network
    Example:
    fees = await vcx_ledger_get_fees()
    :return: JSON representing fees
        { "txnType1": amount1, "txnType2": amount2, ..., "txnTypeN": amountN }
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_ledger_get_fees, "cb"):
        logger.debug("vcx_ledger_get_fees: Creating callback")
        vcx_ledger_get_fees.cb = create_cb(CFUNCTYPE(None, c_uint32))

    result = await do_call('vcx_ledger_get_fees',
                           vcx_ledger_get_fees.cb)

    logger.debug("vcx_ledger_get_fees completed")
    return result


async def vcx_messages_download(status: str = None, uids: str = None, pw_dids: str = None) -> str:
    """
    Retrieve messages from the agent
    :param status: optional, comma separated - query for messages with the specified status.
                                     Possible statuses:
                                     MS-101 - Created
                                     MS-102 - Sent
                                     MS-103 - Received
                                     MS-104 - Accepted
                                     MS-105 - Rejected
                                     MS-106 - Reviewed
    :param uids: optional, comma separated - query for messages with the specified uids
    :param pw_dids: optional, comma separated - DID's pointing to specific connection
    :return: message
        [{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}]
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_messages_download, "cb"):
        logger.debug("vcx_messages_download: Creating callback")
        vcx_messages_download.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    if status:
        c_status = c_char_p(status.encode('utf-8'))
    else:
        c_status = None

    if uids:
        c_uids = c_char_p(uids.encode('utf-8'))
    else:
        c_uids = None

    if pw_dids:
        c_pw_dids = c_char_p(pw_dids.encode('utf-8'))
    else:
        c_pw_dids = None

    result = await do_call('vcx_messages_download',
                           c_status,
                           c_uids,
                           c_pw_dids,
                           vcx_messages_download.cb)

    logger.debug("vcx_messages_download completed")
    return result


async def vcx_messages_update_status(msg_json: str):
    """
    Update the status of messages from the specified connection
    :param msg_json: messages to update:
        [
            {
                "pairwiseDID":"string",
                "uids":["string"]
            },
            ...
        ]
    :return:
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_messages_update_status, "cb"):
        logger.debug("vcx_messages_update_status: Creating callback")
        vcx_messages_update_status.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

    c_msg_json = c_char_p(msg_json.encode('utf-8'))
    c_status = c_char_p("MS-106".encode('utf-8'))

    result = await do_call('vcx_messages_update_status',
                           c_status,
                           c_msg_json,
                           vcx_messages_update_status.cb)

    logger.debug("vcx_messages_update_status completed")
    return result


async def vcx_get_request_price(action_json: str,
                                requester_info_json: Optional[str]):
    """
    Update the status of messages from the specified connection
    :param action_json: {
         "auth_type": ledger transaction alias or associated value,
         "auth_action": type of an action.,
         "field": transaction field,
         "old_value": (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action),
         "new_value": (Optional) new value that can be used to fill the field,
     }
    :param requester_info_json: (Optional) {
         "role": string - role of a user which can sign transaction.
         "count": string - count of users.
         "is_owner": bool - if user is an owner of transaction.
     } otherwise context info will be used

    :return: price - tokens amount required for action performing
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_get_request_price, "cb"):
        logger.debug("vcx_get_request_price: Creating callback")
        vcx_get_request_price.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_uint64))

    c_action_json = c_char_p(action_json.encode('utf-8'))
    c_requester_info_json = c_char_p(requester_info_json.encode('utf-8')) if requester_info_json is not None else None

    result = await do_call('vcx_get_request_price',
                           c_action_json,
                           c_requester_info_json,
                           vcx_get_request_price.cb)

    logger.debug("vcx_get_request_price completed")
    return result


async def vcx_endorse_transaction(transaction: str) -> None:
    """
    Endorse transaction to the ledger preserving an original author
    :param transaction: transaction to endorse
    :return:
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_endorse_transaction, "cb"):
        logger.debug("vcx_endorse_transaction: Creating callback")
        vcx_endorse_transaction.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

    c_transaction = c_char_p(transaction.encode('utf-8'))

    result = await do_call('vcx_endorse_transaction',
                           c_transaction,
                           vcx_endorse_transaction.cb)

    logger.debug("vcx_endorse_transaction completed")
    return result


async def vcx_download_message(uid: str) -> str:
    """
    Retrieves single message from the agency by the given uid.

    :param uid: id of the message to query.
    :return: message
        {
            "statusCode": string,
            "payload":optional(string),
            "senderDID":string,
            "uid":string,
            "type":string,
            "refMsgId":optional(string),
            "deliveryDetails":[],
            "decryptedPayload":"{"@msg":string,"@type":{"fmt":string,"name":string"ver":string}}"
        }
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_download_message, "cb"):
        logger.debug("vcx_download_message: Creating callback")
        vcx_download_message.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    c_uid = c_char_p(uid.encode('utf-8'))

    result = await do_call('vcx_download_message',
                           c_uid,
                           vcx_download_message.cb)

    logger.debug("vcx_download_message completed")
    return result.decode()


async def vcx_fetch_public_entities() -> None:
    """
    Fetch and Cache public entities from the Ledger associated with stored in the wallet credentials.
    This function performs two steps:
        1) Retrieves the list of all credentials stored in the opened wallet.
        2) Fetch and cache Schemas / Credential Definitions / Revocation Registry Definitions
           correspondent to received credentials from the connected Ledger.

    This helper function can be used, for instance as a background task, to refresh library cache.
    This allows us to reduce the time taken for Proof generation by using already cached entities instead of queering the Ledger.

    NOTE: Library must be already initialized (wallet and pool must be opened).

    :return: None
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_fetch_public_entities, "cb"):
        logger.debug("vcx_fetch_public_entities: Creating callback")
        vcx_fetch_public_entities.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

    result = await do_call('vcx_fetch_public_entities',
                           vcx_fetch_public_entities.cb)

    logger.debug("vcx_fetch_public_entities completed")
    return result


async def vcx_health_check() -> None:
    """
    This function allows you to check the health of LibVCX and EAS/CAS instance.
    It will return error in case of any problems on EAS or will resolve pretty long if VCX is thread-hungry.
    WARNING: this call may take a lot of time returning answer in case of load, be careful.
    NOTE: Library must be initialized, ENDPOINT_URL should be set

    :return None
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_health_check, "cb"):
        logger.debug("vcx_health_check: Creating callback")
        vcx_health_check.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

    result = await do_call('vcx_health_check',
                           vcx_health_check.cb)

    logger.debug("vcx_health_check completed")
    return result


async def vcx_create_pairwise_agent() -> str:
    """
    Create pairwise agent which can be later used for connection establishing.

    You can pass `agent_info` into `vcx_connection_connect` function as field of `connection_options` JSON parameter.
    The passed Pairwise Agent will be used for connection establishing instead of creation a new one.

    :param status: optional, comma separated - query for messages with the specified status.
                                     Possible statuses:
                                     MS-101 - Created
                                     MS-102 - Sent
                                     MS-103 - Received
                                     MS-104 - Accepted
                                     MS-105 - Rejected
                                     MS-106 - Reviewed
    :param uids: optional, comma separated - query for messages with the specified uids
    :param pw_dids: optional, comma separated - DID's pointing to specific connection
    :return: message
        [{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}]
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_create_pairwise_agent, "cb"):
        logger.debug("vcx_create_pairwise_agent: Creating callback")
        vcx_create_pairwise_agent.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    result = await do_call('vcx_create_pairwise_agent',
                           vcx_create_pairwise_agent.cb)

    logger.debug("vcx_create_pairwise_agent completed")
    return result.decode()


async def vcx_extract_attached_message(message: str) -> str:
    """
    Extract content of Aries message containing attachment decorator.
    RFC: https://github.com/hyperledger/aries-rfcs/tree/main/features/0592-indy-attachments

    :param message: aries message containing attachment decorator
    :return: attached message as JSON string
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_extract_attached_message, "cb"):
        logger.debug("vcx_extract_attached_message: Creating callback")
        vcx_extract_attached_message.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    c_message = c_char_p(message.encode('utf-8'))

    result = await do_call('vcx_extract_attached_message',
                           c_message,
                           vcx_extract_attached_message.cb)

    logger.debug("vcx_extract_attached_message completed")
    return result


async def vcx_extract_thread_id(message: str) -> str:
    """
    Extract thread id for message

    :param message: aries message containing attachment decorator
    :return: thread id
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_extract_thread_id, "cb"):
        logger.debug("vcx_extract_thread_id: Creating callback")
        vcx_extract_thread_id.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    c_message = c_char_p(message.encode('utf-8'))

    result = await do_call('vcx_extract_thread_id',
                           c_message,
                           vcx_extract_thread_id.cb)

    logger.debug("vcx_extract_thread_id completed")
    return result


async def vcx_resolve_message_by_url(url: str) -> str:
    """
    Resolve message by the given URL.
    Supported cases:
      1. Message inside of query parameters (c_i, oob, d_m, m) as base64 encoded string
      2. Message inside response `location` header for GET request
      3. Message inside response for GET request

    :param url: url to fetch message
    :return: resolved message
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_resolve_message_by_url, "cb"):
        logger.debug("vcx_resolve_message_by_url: Creating callback")
        vcx_resolve_message_by_url.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

    c_url = c_char_p(url.encode('utf-8'))

    result = await do_call('vcx_resolve_message_by_url',
                           c_url,
                           vcx_resolve_message_by_url.cb)

    logger.debug("vcx_resolve_message_by_url completed")
    return result
