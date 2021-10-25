import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface ISetActiveTxnAuthorAgreementMetaData {
  text: string
  version: string
  taaDigest: string
  mechanism: string
  timestamp: number
}

interface IGetAcceptanceMechanismsData {
  submitterDid: string
  timestamp: number
  version: string
}

interface IAppendTxnAuthorAgreementData {
  requestJson: string
  text: string
  version: string
  taaDigest: string
  mechanism: string
  timestamp: number
}

interface IGetRequestRedirectionUrlData {
  url: string
}

interface IExtractAttachedMessage {
  message: string
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
    return await RNIndy.getAcceptanceMechanisms(submitterDid, timestamp, version)
  }

  /**
   * Set some accepted agreement as active.
   * <p>
   * Either combination text/version ot hash must be passed.
   *
   * @param  text                 Optional(string) text of transaction agreement
   * @param  version              Optional(string) version of transaction agreement
   * @param  hash                 Optional(string) hash on text and version. This parameter is required if text and version parameters are ommited.
   * @param  mechanism          mechanism how user has accepted the TAA
   * @param  timestamp     UTC timestamp when user has accepted the TAA
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
    return await RNIndy.setActiveTxnAuthorAgreementMeta(text, version, taaDigest, mechanism, timestamp)
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
    return await RNIndy.appendTxnAuthorAgreement(requestJson, text, version, taaDigest, mechanism, timestamp)
  }

  /**
   * Fetch and Cache public entities from the Ledger associated with stored in the wallet credentials.
   * This function performs two steps:
   *     1) Retrieves the list of all credentials stored in the opened wallet.
   *     2) Fetch and cache Schemas / Credential Definitions / Revocation Registry Definitions
   *        correspondent to received credentials from the connected Ledger.
   *
   * This helper function can be used, for instance as a background task, to refresh library cache.
   * This allows us to reduce the time taken for DisclosedProof generation by using already cached entities instead of queering the Ledger.
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

  public static async getRequestRedirectionUrl({ url }: IGetRequestRedirectionUrlData): Promise<string> {
    return await RNIndy.getRequestRedirectionUrl(url)
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

  /**
   * Extract content of Aries message containing attachment decorator.
   * RFC: https://github.com/hyperledger/aries-rfcs/tree/main/features/0592-indy-attachments
   * @return                  string - attached message as JSON string
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async extractAttachedMessage({ message }: IExtractAttachedMessage): Promise<string> {
    return await RNIndy.extractAttachedMessage(message)
  }
}
