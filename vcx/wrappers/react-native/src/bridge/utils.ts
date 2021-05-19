import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface ISetActiveTxnAuthorAgreementMetaData {
  text: string,
  version: string,
  taaDigest: string,
  mechanism: string,
  timestamp: number,
}

interface IGetAcceptanceMechanismsData {
  submitterDid: string,
  timestamp: number,
  version: string,
}

interface IAppendTxnAuthorAgreementData {
  requestJson: string,
  text: string,
  version: string,
  taaDigest: string,
  mechanism: string,
  timestamp: number,
}

interface IGetGenesisPathData {
  poolConfig: string,
  fileName: string,
}

interface IToBase64FromUtf8Data {
  data: string,
  base64EncodingOption: string,
}

interface IToUtf8FromBase64Data {
  data: string,
  base64EncodingOption: string,
}

interface IGenerateThumbprintData {
  data: string,
  base64EncodingOption: string,
}

interface IGetColorData {
  imagePath: string,
}

interface IGetRequestRedirectionUrlData {
  url: string,
}

interface IAddTxnAuthorAgreement {
  submitterDid: string,
  text: string,
  version: string,
}

interface IAddAcceptanceMechanisms {
  submitterDid: string,
  aml: string,
  version: string,
  amlContext: string,
}

interface IAnonDecrypt {
  handle: number,
  recipientVk: string,
  encryptedMsg: string,
}

interface ISignDataResult {
  data: string,
  signature: string,
}

interface ICreateOneTimeInfo {
  config: string,
}

interface IVcxGetRequestPrice {
  config: string,
  requesterInfoJson: string
}

interface IVcxEndorseTransaction {
  requesterInfoJson: string
}

export class Utils {
  /**
   * Get ledger fees from the network
   *
   * @return                the fee structure for the sovrin network
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async getLedgerFees(): Promise<string> {
    return await RNIndy.getLedgerFees()
  }

  /**
   * Retrieve author agreement and acceptance mechanisms set on the Ledger
   *
   * @return               transaction author agreement set on the ledger
   *                       "{"text":"Default agreement", "version":"1.0.0", "aml": {"label1": "description"}}"
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async getLedgerAuthorAgreement(): Promise<string> {
    return await RNIndy.getTxnAuthorAgreement()
  }

  /**
   * Builds a GET_TXN_AUTHR_AGRMT_AML request. Request to get a list of  acceptance mechanisms from the ledger
   * valid for specified time or the latest one.
   *
   * EXPERIMENTAL
   *
   * @param submitterDid (Optional) DID of the request sender.
   * @param timestamp - time to get an active acceptance mechanisms. Pass -1 to get the latest one.
   * @param version - (Optional) version of acceptance mechanisms.
   *
   * NOTE: timestamp and version cannot be specified together.
   *
   * @return A future resolving to a request result as json.
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async getLedgerAcceptanceMechanisms({
    submitterDid,
    timestamp,
    version,
  }: IGetAcceptanceMechanismsData): Promise<string> {
    return await RNIndy.getAcceptanceMechanisms(
      submitterDid,
      timestamp,
      version,
    )
  }

  /**
   * Set some accepted agreement as active.
   * <p>
   * Either combination text/version ot hash must be passed.
   *
   * @param  text                 Optional(string) text of transaction agreement
   * @param  version              Optional(string) version of transaction agreement
   * @param  hash                 Optional(string) hash on text and version. This parameter is required if text and version parameters are ommited.
   * @param  accMechType          mechanism how user has accepted the TAA
   * @param  timeOfAcceptance     UTC timestamp when user has accepted the TAA
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async setActiveTxnAuthorAgreement({
    text,
    version,
    taaDigest,
    mechanism,
    timestamp,
  }: ISetActiveTxnAuthorAgreementMetaData): Promise<string> {
    return await RNIndy.setActiveTxnAuthorAgreementMeta(
      text,
      version,
      taaDigest,
      mechanism,
      timestamp
    )
  }

  /**
   * Append transaction author agreement acceptance data to a request.
   * This function should be called before signing and sending a request
   * if there is any transaction author agreement set on the Ledger.
   *
   * EXPERIMENTAL
   *
   * This function may calculate digest by itself or consume it as a parameter.
   * If all text, version and taaDigest parameters are specified, a check integrity of them will be done.
   *
   * @param requestJson original request data json.
   * @param text - (Optional) raw data about TAA from ledger.
   * @param version - (Optional) raw version about TAA from ledger.
   *     `text` and `version` parameters should be passed together.
   *     `text` and `version` parameters are required if taaDigest parameter is omitted.
   * @param taaDigest - (Optional) digest on text and version. This parameter is required if text and version parameters are omitted.
   * @param mechanism - mechanism how user has accepted the TAA
   * @param time - UTC timestamp when user has accepted the TAA. Note that the time portion will be discarded to avoid a privacy risk.
   *
   * @return A future resolving to an updated request result as json.
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async appendTxnAuthorAgreement({
    requestJson,
    text,
    version,
    taaDigest,
    mechanism,
    timestamp,
  }: IAppendTxnAuthorAgreementData): Promise<string> {
    return await RNIndy.appendTxnAuthorAgreement(
      requestJson,
      text,
      version,
      taaDigest,
      mechanism,
      timestamp
    )
  }

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
  public static async fetchPublicEntities(): Promise<void> {
    return await RNIndy.fetchPublicEntities()
  }

  public static async getGenesisPathWithConfig({
    poolConfig,
    fileName,
  }: IGetGenesisPathData): Promise<string> {
    return await RNIndy.getGenesisPathWithConfig(
      poolConfig,
      fileName,
    )
  }

  public static async getRequestRedirectionUrl({
    url,
  }: IGetRequestRedirectionUrlData): Promise<string> {
    return await RNIndy.getRequestRedirectionUrl(
      url,
    )
  }

  /**
   * Builds a TXN_AUTHR_AGRMT request. Request to add a new version of Transaction Author Agreement to the ledger.
   *
   * EXPERIMENTAL
   *
   * @param submitterDid DID of the request sender.
   * @param text -  a content of the TTA.
   * @param version -  a version of the TTA (unique UTF-8 string).
   *
   * @return A future resolving to a request result as json.
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async addTxnAuthorAgreement({
    submitterDid,
    text,
    version,
  }: IAddTxnAuthorAgreement): Promise<string> {
    return await RNIndy.addTxnAuthorAgreement(
      submitterDid,
      text,
      version,
    )
  }

  /**
   * Builds a SET_TXN_AUTHR_AGRMT_AML request. Request to add a new list of acceptance mechanisms for transaction author agreement.
   * Acceptance Mechanism is a description of the ways how the user may accept a transaction author agreement.
   *
   * EXPERIMENTAL
   *
   * @param submitterDid DID of the request sender.
   * @param aml - a set of new acceptance mechanisms:
   * <pre>
   * {@code
   * {
   *     "<acceptance mechanism label 1>": { acceptance mechanism description 1},
   *     "<acceptance mechanism label 2>": { acceptance mechanism description 2},
   *     ...
   * }
   * }
   * </pre>
   *
   * @param version - a version of new acceptance mechanisms. (Note: unique on the Ledger).
   * @param amlContext - (Optional) common context information about acceptance mechanisms (may be a URL to external resource).
   *
   * @return A future resolving to a request result as json.
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async addAcceptanceMechanisms({
    submitterDid,
    aml,
    version,
    amlContext,
  }: IAddAcceptanceMechanisms): Promise<string> {
    return await RNIndy.addAcceptanceMechanisms(
      submitterDid,
      aml,
      version,
      amlContext
    )
  }

  /**
   * Decrypts a message by anonymous-encryption scheme.
   *
   * Sealed boxes are designed to anonymously send messages to a Recipient given its public key.
   * Only the Recipient can decrypt these messages, using its private key.
   * While the Recipient can verify the integrity of the message, it cannot verify the identity of the Sender.
   *
   * Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
   * for specific DID.
   *
   * @param walletHandle       The walletHandle.
   * @param recipientVk  Id (verkey) of my key. The key must be created by calling createKey or createAndStoreMyDid
   * @param encryptedMsg encrypted message
   * @return A future that resolves to a decrypted message as an array of bytes.
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async anonDecrypt({
    handle,
    recipientVk,
    encryptedMsg,
  }: IAnonDecrypt): Promise<ISignDataResult> {
    return await RNIndy.anonDecrypt(
      handle,
      recipientVk,
      encryptedMsg,
    )
  }

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
  public static async vcxGetRequestPrice({ config, requesterInfoJson }: IVcxGetRequestPrice): Promise<number> {
    return await RNIndy.vcxGetRequestPrice(config, requesterInfoJson)
  }

  /**
   * Endorse transaction to the ledger preserving an original author
   *
   * @param  transactionJson  transaction to endorse
   *
   * @return                  void
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async vcxEndorseTransaction({ requesterInfoJson }: IVcxEndorseTransaction): Promise<void> {
    return await RNIndy.vcxEndorseTransaction(requesterInfoJson)
  }

  /**
   * This function allows you to check the health of LibVCX and EAS/CAS instance.
   * It will return error in case of any problems on EAS or will resolve pretty long if VCX is thread-hungry.
   * WARNING: this call may take a lot of time returning answer in case of load, be careful.
   * NOTE: Library must be initialized, ENDPOINT_URL should be set
   * @return                  void
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async healthCheck(): Promise<void> {
    return await RNIndy.healthCheck()
  }
}
