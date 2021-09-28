import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface IWalletKeyData {
  lengthOfKey: number
}

interface IWalletExportData {
  exportPath: string
  encryptionKey: string
}

interface IWalletImportData {
  config: string
}

interface IWalletAddRecordData {
  type: string
  key: string
  value: string
}

interface IWalletGetItemData {
  type: string
  key: string
}

interface IWalletDeleteRecordData {
  type: string
  key: string
}

interface IWalletUpdateRecordData {
  type: string
  key: string
  value: string
}

interface IWalletAddRecordTagsData {
  type: string
  key: string
  tags: string
}

interface IWalletUpdateRecordTagsData {
  type: string
  key: string
  tags: string
}

interface IWalletDeleteRecordTagsData {
  type: string
  key: string
  tags: string
}

interface IWalletOpenSearchData {
  type: string
  query: string
  options: string
}

interface IWalletSearchNextRecordsData {
  handle: number
  count: number
}

interface IWalletCloseSearchData {
  handle: number
}

interface IWalletGetTokenInfoData {
  paymentHandle: number
}

interface IWalletSendTokensData {
  paymentHandle: number
  tokens: string
  recipient: string
}

interface IWalletCreatePaymentAddressData {
  seed: string
}

export class Wallet {
  /**
   * Generate key of the specific length
   *
   * @param  lengthOfKey      Length of the key to generate
   *
   * @return                  Key as a string
   *
   * @throws VcxException     Thrown if an error occurs when calling the underlying SDK.
   */
  public static async creatKey({ lengthOfKey }: IWalletKeyData): Promise<string> {
    return await RNIndy.createWalletKey(lengthOfKey)
  }

  /**
   * Exports opened wallet
   *
   * @param  exportPath       Path to export wallet to User's File System.
   * @param  encryptionKey    String representing the User's Key for securing (encrypting) the exported Wallet.
   *
   * @return                  void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async export({ exportPath, encryptionKey }: IWalletExportData): Promise<number> {
    return await RNIndy.exportWallet(exportPath, encryptionKey)
  }

  /**
   * Creates a new secure wallet and then imports its content
   * according to fields provided in import_config
   * Cannot be used if wallet is already opened (Especially if vcx_init has already been used).
   *
   * @param  config           Configuration JSON for importing wallet
   *                          "{"wallet_name":"","wallet_key":"","exported_wallet_path":"","backup_key":"","key_derivation":""}"
   *
   * @return                  void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async import({ config }: IWalletImportData): Promise<number> {
    return await RNIndy.decryptWalletFile(config)
  }

  /**
   * Adds a record to the wallet
   *
   * @param type          type of record. (e.g. 'data', 'string', 'foobar', 'image')
   * @param key          the id ("key") of the record.
   * @param value       value of the record with the associated id.
   *
   * @return                  void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async addRecord({ type, key, value }: IWalletAddRecordData): Promise<number> {
    return await RNIndy.addWalletRecord(type, key, value)
  }

  /**
   * Gets a record from Wallet.
   *
   * @param type          type of record. (e.g. 'data', 'string', 'foobar', 'image')
   * @param key          the id ("key") of the record.
   *
   * @return                  received record as JSON
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async getRecord({ type, key }: IWalletGetRecordData): Promise<string> {
    return await RNIndy.getWalletRecord(type, key)
  }

  /**
   * Deletes an existing record.
   *
   * @param type          type of record. (e.g. 'data', 'string', 'foobar', 'image')
   * @param key          the id ("key") of the record.
   *
   * @return                  void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async deleteRecord({ type, key }: IWalletDeleteRecordData): Promise<number> {
    return await RNIndy.deleteWalletRecord(type, key)
  }

  /**
   * Updates the value of a record already in the wallet.
   *
   * @param type          type of record. (e.g. 'data', 'string', 'foobar', 'image')
   * @param key          the id ("key") of the record.
   * @param value       new value of the record with the associated id.
   *
   * @return                  void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async updateRecord({ type, key, value }: IWalletUpdateRecordData): Promise<number> {
    return await RNIndy.updateWalletRecord(type, key, value)
  }

  /**
   * Add tags to a record in the wallet.
   * Assumes there is an open wallet and that a record with specified type and id pair already exists.
   *
   * @param type         type of record. (e.g. 'data', 'string', 'foobar', 'image')
   * @param key          the id ("key") of the record.
   * @param tags         tags for the record with the associated id and type.
   *
   * @return             void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async addRecordTags({ type, key, tags }: IWalletAddRecordTagsData): Promise<number> {
    return await RNIndy.addWalletRecordTags(type, key, tags)
  }

  /**
   * Updates tags of a record in the wallet.
   * Assumes there is an open wallet and that a record with specified type and id pair already exists.
   *
   * @param type         type of record. (e.g. 'data', 'string', 'foobar', 'image')
   * @param key          the id ("key") of the record.
   * @param tags         tags for the record with the associated id and type.
   *
   * @return             void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async updateRecordTags({ type, key, tags }: IWalletUpdateRecordTagsData): Promise<number> {
    return await RNIndy.updateWalletRecordTags(type, key, tags)
  }

  /**
   * Deletes tags from a record in the wallet.
   * Assumes there is an open wallet and that a record with specified type and id pair already exists.
   *
   * @param type         type of record. (e.g. 'data', 'string', 'foobar', 'image')
   * @param key          the id ("key") of the record.
   * @param tags         tags to delete for the record with the associated id and type.
   *
   * @return             void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async deleteRecordTags({ type, key, tags }: IWalletDeleteRecordTagsData): Promise<number> {
    return await RNIndy.deleteWalletRecordTags(type, key, tags)
  }

  /**
   * Search for records in the wallet.
   *
   * @param type         type of record. (e.g. 'data', 'string', 'foobar', 'image')
   * @param query        MongoDB style query to wallet record tags:
   *                {
   *                    "tagName": "tagValue",
   *                    $or: {
   *                        "tagName2": { $regex: 'pattern' },
   *                        "tagName3": { $gte: 123 },
   *                    },
   *                }
   * @param options
   *            {
   *               retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
   *               retrieveTotalCount: (optional, false by default) Calculate total count,
   *               retrieveType: (optional, false by default) Retrieve record type,
   *               retrieveValue: (optional, true by default) Retrieve record value,
   *               retrieveTags: (optional, true by default) Retrieve record tags,
   *            }
   *
   * @return    search handle
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async openSearch({ type, query, options }: IWalletOpenSearchData): Promise<number> {
    return await RNIndy.openWalletSearch(type, key, tags)
  }

  /**
   * Fetch next records for wallet search.
   *
   * @param handle         handle pointing to search (returned by openSearch).
   * @param count          number of records to fetch
   *
   * @return                wallet records json:
   *                        {
   *                            totalCount: <int>, // present only if retrieveTotalCount set to true
   *                            records: [{ // present only if retrieveRecords set to true
   *                                id: "Some id",
   *                                type: "Some type", // present only if retrieveType set to true
   *                                value: "Some value", // present only if retrieveValue set to true
   *                                tags: <tags json>, // present only if retrieveTags set to true
   *                            }],
   *                        }
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async searchNextRecords({ handle, count }: IWalletSearchNextRecordsData): Promise<string> {
    return await RNIndy.searchNextWalletRecords(handle, count)
  }

  /**
   * Close a search
   *
   * @param handle         handle pointing to search (returned by openSearch).
   *
   * @return               void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async closeSearch({ handle }: IWalletCloseSearchData): Promise<number> {
    return await RNIndy.closeWalletSearch(handle)
  }

  /**
   * Get the total balance from all addresses contained in the configured wallet.
   *
   * @param  paymentHandle            unused parameter (pass 0)
   * @return                          payment information stored in the wallet
   *                                  "{"balance":6,"balance_str":"6","addresses":[{"address":"pay:null:9UFgyjuJxi1i1HD","balance":3,"utxo":[{"source":"pay:null:1","paymentAddress":"pay:null:zR3GN9lfbCVtHjp","amount":1,"extra":"yqeiv5SisTeUGkw"}]}]}"
   *
   * @throws VcxException             If an exception occurred in Libvcx library.
   */
  public static async getTokenInfo({ paymentHandle }: IWalletGetTokenInfoData): Promise<string> {
    return await RNIndy.getTokenInfo(paymentHandle)
  }

  /**
   * Send tokens to a specific address
   *
   * @param  paymentHandle            unused parameter (pass 0)
   * @param  tokens                   number of tokens to send
   * @param  recipient                address of recipient
   *
   * @return                          receipt of token transfer
   *
   * @throws VcxException             If an exception occurred in Libvcx library.
   */
  public static async sendTokens({ paymentHandle, tokens, recipient }: IWalletSendTokensData): Promise<boolean> {
    return await RNIndy.sendTokens(paymentHandle, tokens, recipient)
  }

  /**
   * Add a payment address to the wallet
   *
   * @param  seed            Seed to use for creation
   *
   * @return                 generated payment address
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async createPaymentAddress({ seed }: IWalletCreatePaymentAddressData): Promise<string> {
    return await RNIndy.createPaymentAddress(seed)
  }
}
