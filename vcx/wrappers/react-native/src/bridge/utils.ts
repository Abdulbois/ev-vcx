import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface IGetRequestRedirectionUrlData {
  url: string
}

interface IExtractAttachedMessage {
  message: string
}

interface IExtractThreadId {
  message: string
}

interface IResolveMessageByUrl {
  url: string
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

  /**
   * Extract thread id for message
   *
   * @return               string - thread id
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async extractThreadId({ message }: IExtractThreadId): Promise<string> {
    return await RNIndy.extractThreadId(message)
  }

  /**
   * Resolve message by the given URL.
   * Supported cases:
   *   1. Message inside of query parameters (c_i, oob, d_m, m) as base64 encoded string
   *   2. Message inside response `location` header for GET request
   *   3. Message inside response for GET request
   *
   * @return                  string - resolved message as JSON string
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async resolveMessageByUrl({ url }: IResolveMessageByUrl): Promise<string> {
    return await RNIndy.resolveMessageByUrl(url)
  }
}
