import { NativeModules } from 'react-native'
import { v4 as uuidv4 } from 'uuid'

const { RNIndy } = NativeModules

interface IWalletCreateBackupData {
  backupKey: string,
}

interface IWalletSendBackupData {
  handle: number,
  path: string,
}

interface IWalletUpdateBackupStateData {
  handle: number,
}

interface IWalletUpdateBackupStateWithMessageData {
  handle: number,
  message: string,
}

interface IWalletSerializeBackupData {
  handle: number,
}

interface IWalletDeserializeBackupData {
  serialized: string,
}

interface IWalletRestoreData {
  config: string,
}

export class CloudWalletBackup {
  /**
   * Create a Wallet Backup object that provides a Cloud wallet backup and provision's backup protocol with Agent
   *
   * @param sourceID        institution's personal identification for the user
   * @param backupKey       String representing the User's Key for securing (encrypting) the exported Wallet.
   *
   * @return                handle that should be used to perform actions with the WalletBackup object.
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async create({
    backupKey,
  }: IWalletCreateBackupData): Promise<number> {
    return await RNIndy.createWalletBackup(
      uuidv4(),
      backupKey,
    )
  }

  /**
   * Wallet Backup to the Cloud
   *
   * @param walletBackupHandle  handle pointing to WalletBackup object.
   * @param path                path to export wallet to User's File System. (This instance of the export
   *
   * @return                    void
   *
   * @throws VcxException Thrown if an error occurs when calling the underlying SDK.
   */
  public static async send({
    handle,
    path,
  }: IWalletSendBackupData): Promise<number> {
    return await RNIndy.backupWalletBackup(
      handle,
      path,
    )
  }

  /**
   * Checks for any state change and updates the the state attribute
   *
   * @param  walletBackupHandle  handle pointing to WalletBackup object.
   *
   * @return                      the most current state of the WalletBackup object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async updateState({
    handle,
  }: IWalletUpdateBackupStateData): Promise<number> {
    return await RNIndy.updateWalletBackupState(
      handle,
    )
  }

  /**
   * Update the state of the WalletBackup object based on the given message.
   *
   * @param  walletBackupHandle  handle pointing to WalletBackup object.
   * @param  message              message to process for any WalletBackup state transitions.
   *
   * @return                      the most current state of the WalletBackup object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async updateStateWithMessage({
    handle,
    message,
  }: IWalletUpdateBackupStateWithMessageData): Promise<number> {
    return await RNIndy.updateWalletBackupStateWithMessage(
      handle,
      message,
    )
  }

  /**
   * Requests a recovery of a backup previously stored with a cloud agent
   *
   * @param  config          config to use for wallet backup restoring
   *                         "{
   *                              "wallet_name":string, - new wallet name
   *                              "wallet_key":string, - key to use for encryption of the new wallet
   *                              "exported_wallet_path":string, - path to exported wallet
   *                              "backup_key":string, - key used for export
   *                              "key_derivation":Option(string) - key derivation method to use for new wallet
   *                         }"
   *
   * @return                 void
   *
   * @throws VcxException    If an exception occurred in Libvcx library.
   */
  public static async restore({
    config,
  }: IWalletRestoreData): Promise<number> {
    return await RNIndy.restoreWallet(
      config,
    )
  }

  /**
   * Get JSON string representation of WalletBackup object.
   *
   * @param  walletBackupHandle  handle pointing to WalletBackup object.
   *
   * @return                      WalletBackup object as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async serialize({
    handle,
  }: IWalletSerializeBackupData): Promise<string> {
    return await RNIndy.serializeBackupWallet(
      handle,
    )
  }

  /**
   * Takes a json string representing a WalletBackup object and recreates an object matching the JSON.
   *
   * @param  walletBackupStr JSON string representing a WalletBackup object.
   *
   * @return                 handle that should be used to perform actions with the WalletBackup object.
   *
   * @throws VcxException    If an exception occurred in Libvcx library.
   */
  public static async deserialize({
    serialized,
  }: IWalletDeserializeBackupData): Promise<number> {
    return await RNIndy.deserializeBackupWallet(
      serialized,
    )
  }
}
