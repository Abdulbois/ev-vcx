from ctypes import *
import logging
from vcx.common import do_call, do_call_sync, create_cb

__all__ = ["vcx_init", "vcx_init_with_config"]

async def vcx_init(config_path: str) -> None:
    """
    Initializes VCX with config file.
    The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md

    :param config_path: String - path to a config file to populate config attributes
    Example:
    await vcx_init('/home/username/vcxconfig.json')
    :return:
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_init, "cb"):
        logger.debug("vcx_init: Creating callback")
        vcx_init.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

    c_config_path = c_char_p(config_path.encode('utf-8'))

    result = await do_call('vcx_init',
                           c_config_path,
                           vcx_init.cb)

    logger.debug("vcx_init completed")
    return result


async def vcx_init_with_config(config: str) -> None:
    """
    Initializes VCX with config settings

    :param config: config as json.
    The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md

    Example:
    config = {
      "agency_did": "VsKV7grR1BUE29mG2Fm2kX",
      "agency_verkey": "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
      "agency_endpoint": "http://localhost:8080",
      "genesis_path":"/var/lib/indy/verity-staging/pool_transactions_genesis",
      "institution_name": "institution",
      "institution_logo_url": "http://robohash.org/234",
      "institution_did": "EwsFhWVoc3Fwqzrwe998aQ",
      "institution_verkey": "8brs38hPDkw5yhtzyk2tz7zkp8ijTyWnER165zDQbpK6",
      "remote_to_sdk_did": "EtfeMFytvYTKnWwqTScp9D",
      "remote_to_sdk_verkey": "8a7hZDyJK1nNCizRCKMr4H4QbDm8Gg2vcbDRab8SVfsi",
      "sdk_to_remote_did": "KacwZ2ndG6396KXJ9NDDw6",
      "sdk_to_remote_verkey": "B8LgZGxEPcpTJfZkeqXuKNLihM1Awm8yidqsNwYi5QGc"
    }
    await vcx_init_with_config(config)
    :return:
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_init_with_config, "cb"):
        logger.debug("vcx_init_with_config: Creating callback")
        vcx_init_with_config.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

    c_config = c_char_p(config.encode('utf-8'))

    result = await do_call('vcx_init_with_config',
                           c_config,
                           vcx_init_with_config.cb)

    logger.debug("vcx_init_with_config completed")
    return result


async def vcx_init_pool(pool_config: str) -> None:
    """
    Connect to a Pool Ledger

    You can deffer connecting to the Pool Ledger during library initialization (vcx_init or vcx_init_with_config)
    to decrease the taken time by omitting `genesis_path` field in config JSON.
    Next, you can use this function (for instance as a background task) to perform a connection to the Pool Ledger.

    Note: Pool must be already initialized before sending any request to the Ledger.

    Note: EXPERIMENTAL

    :param pool_config: String - the configuration JSON containing pool related settings:
                                {
                                    genesis_path: string - path to pool ledger genesis transactions,
                                    pool_name: Optional[string] - name of the pool ledger configuration will be created.
                                                                  If no value specified, the default pool name pool_name will be used.
                                    pool_config: Optional[string] - runtime pool configuration json:
                                            {
                                                "timeout": int (optional), timeout for network request (in sec).
                                                "extended_timeout": int (optional), extended timeout for network request (in sec).
                                                "preordered_nodes": array<string> -  (optional), names of nodes which will have a priority during request sending:
                                                        ["name_of_1st_prior_node",  "name_of_2nd_prior_node", .... ]
                                                        This can be useful if a user prefers querying specific nodes.
                                                        Assume that `Node1` and `Node2` nodes reply faster.
                                                        If you pass them Libindy always sends a read request to these nodes first and only then (if not enough) to others.
                                                        Note: Nodes not specified will be placed randomly.
                                                "number_read_nodes": int (optional) - the number of nodes to send read requests (2 by default)
                                                        By default Libindy sends a read requests to 2 nodes in the pool.
                                            }
                                    network: Optional[string] - Network identifier used for fully-qualified DIDs.
                                }

    Example:
    await vcx_init_pool('/home/username/docker.txn')

    :return: None
    """
    logger = logging.getLogger(__name__)

    if not hasattr(vcx_init_pool, "cb"):
        logger.debug("vcx_init_pool: Creating callback")
        vcx_init_pool.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

    c_pool_config = c_char_p(pool_config.encode('utf-8'))

    result = await do_call('vcx_init_pool',
                           c_pool_config,
                           vcx_init_pool.cb)

    logger.debug("vcx_init_pool completed")
    return result
