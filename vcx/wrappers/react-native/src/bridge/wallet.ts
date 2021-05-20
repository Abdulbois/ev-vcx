import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface IWalletKeyData {
  lengthOfKey: number,
}

interface IWalletExportData {
  exportPath: string,
  encryptionKey: string,
}

interface IWalletImportData {
  config: string,
}

interface IWalletSetItemData {
  key: string,
  value: string,
}

interface IWalletGetItemData {
  key: string,
}

interface IWalletDeleteItemData {
  key: string,
}

interface IWalletUpdateItemData {
  key: string,
  value: string,
}

interface IWalletGetTokenInfoData {
  paymentHandle: number,
}

interface IWalletSendTokensData {
  paymentHandle: number,
  tokens: string,
  recipient: string,
}

interface IWalletCreatePaymentAddressData {
  seed: string,
}

interface ISignDataResult {
  data: string,
  signature: string,
}

interface ISignWithAddress {
  address: string,
  message: ISignDataResult,
}

interface IVerifyWithAddress {
  address: string,
  message: ISignDataResult,
  signature: ISignDataResult,
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
  public static async creatKey({
    lengthOfKey,
  }: IWalletKeyData): Promise<string> {
    return await RNIndy.createWalletKey(
      lengthOfKey,
    )
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
  public static async export({
    exportPath,
    encryptionKey,
  }: IWalletExportData): Promise<number> {
    return await RNIndy.exportWallet(
      exportPath,
      encryptionKey,
    )
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
  public static async import({
    config,
  }: IWalletImportData): Promise<number> {
    return await RNIndy.decryptWalletFile(
      config,
    )
  }

  /**
   * Adds a record to the wallet
   *
   * @param key          the id ("key") of the record.
   * @param value       value of the record with the associated id.
   *
   * @return                  void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async setItem({
    key,
    value,
  }: IWalletSetItemData): Promise<number> {
    return await RNIndy.setWalletItem(
      key,
      value,
    )
  }

  /**
   * Gets a record from Wallet.
   *
   * @param key          the id ("key") of the record.
   *
   * @return                  received record as JSON
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async getItem({
    key,
  }: IWalletGetItemData): Promise<string> {
    return await RNIndy.getWalletItem(
      key,
    )
  }

  /**
   * Deletes an existing record.
   *
   * @param key          the id ("key") of the record.
   *
   * @return                  void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async deleteItem({
    key,
  }: IWalletDeleteItemData): Promise<number> {
    return await RNIndy.deleteWalletItem(
      key,
    )
  }

  /**
   * Updates the value of a record already in the wallet.
   *
   * @param key          the id ("key") of the record.
   * @param value       new value of the record with the associated id.
   *
   * @return                  void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async updateItem({
    key,
    value,
  }: IWalletUpdateItemData): Promise<number> {
    return await RNIndy.updateWalletItem(
      key,
      value,
    )
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
  public static async getTokenInfo({
    paymentHandle,
  }: IWalletGetTokenInfoData): Promise<string> {
    return await RNIndy.getTokenInfo(
      paymentHandle,
    )
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
  public static async sendTokens({
    paymentHandle,
    tokens,
    recipient,
  }: IWalletSendTokensData): Promise<boolean> {
    return await RNIndy.sendTokens(
      paymentHandle,
      tokens,
      recipient,
    )
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
  public static async createPaymentAddress({
    seed,
  }: IWalletCreatePaymentAddressData): Promise<string> {
    return await RNIndy.createPaymentAddress(
      seed,
    )
  }

  /**
   * Signs a message with a payment address.
   *
   * @param address:  Payment address of message signer.
   * @param message   The message to be signed
   *
   * @return A future that resolves to a signature string.
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async sign({
    address,
    message,
  }: ISignWithAddress): Promise<string> {
    return await RNIndy.signWithAddress(
      address,
      message
    )
  }

  /**
   * Verify a signature with a payment address.
   *
   * @param address   Payment address of the message signer
   * @param message   Message that has been signed
   * @param signature A signature to be verified
   * @return A future that resolves to true if signature is valid, otherwise false.
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async verity({
    address,
    message,
    signature,
  }: IVerifyWithAddress): Promise<string> {
    return await RNIndy.verifyWithAddress(
      address,
      message,
      signature
    )
  }
}
