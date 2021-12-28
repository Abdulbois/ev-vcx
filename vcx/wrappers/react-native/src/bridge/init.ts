import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface IInitData {
  config: string
}

interface IInitPoolData {
  config: string
}

interface IShutdownData {
  deleteWallet?: boolean
}

export class Library {
  /**
   * Initializes VCX with config
   * An example file is at libvcx/sample_config/config.json
   * The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
   *
   * @param  config           config as JSON string to use for library initialization
   *
   * @return                  void
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async init({ config }: IInitData): Promise<boolean> {
    return await RNIndy.init(config)
  }

  /**
   * Connect to a Pool Ledger
   *
   * You can deffer connecting to the Pool Ledger during library initialization (vcx_init or vcx_init_with_config)
   * to decrease the taken time by omitting `genesis_path` field in config JSON.
   * Next, you can use this function (for instance as a background task) to perform a connection to the Pool Ledger.
   *
   * Note: Pool must be already initialized before sending any request to the Ledger.
   *
   * EXPERIMENTAL
   *
   * @param  config           the configuration JSON containing pool related settings:
   *                          {
   *                              genesis_path: string - path to pool ledger genesis transactions,
   *                              pool_name: Optional[string] - name of the pool ledger configuration will be created.
   *                                                   If no value specified, the default pool name pool_name will be used.
   *                              pool_config: Optional[string] - runtime pool configuration json:
   *                                  {
   *                                      "timeout": int (optional), timeout for network request (in sec).
   *                                      "extended_timeout": int (optional), extended timeout for network request (in sec).
   *                                      "preordered_nodes": array<string> -  (optional), names of nodes which will have a priority during request sending:
   *                                         ["name_of_1st_prior_node",  "name_of_2nd_prior_node", .... ]
   *                                         This can be useful if a user prefers querying specific nodes.
   *                                         Assume that `Node1` and `Node2` nodes reply faster.
   *                                         If you pass them Libindy always sends a read request to these nodes first and only then (if not enough) to others.
   *                                         Note: Nodes not specified will be placed randomly.
   *                                      "number_read_nodes": int (optional) - the number of nodes to send read requests (2 by default)
   *                                         By default Libindy sends a read requests to 2 nodes in the pool.
   *                                  }
   *                              network: Optional[string] - Network identifier used for fully-qualified DIDs.
   *                          }
   *
   *                          Note: You can also pass a list of network configs.
   *                                In this case library will connect to multiple ledger networks and will look up public data in each of them.
   *                                [{ "genesis_path": string, "pool_name": string, ... }]
   *
   * @return                  void
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async initPool({ config }: IInitPoolData): Promise<boolean> {
    return await RNIndy.vcxInitPool(config)
  }

  /**
   * Reset libvcx to a pre-configured state, releasing/deleting any handles and freeing memory
   *
   * @param  deleteWallet     specify whether wallet/pool should be deleted
   *
   * @return                  void
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async shutdown({ deleteWallet }: IShutdownData): Promise<void> {
    return await RNIndy.shutdownVcx(deleteWallet)
  }
}
