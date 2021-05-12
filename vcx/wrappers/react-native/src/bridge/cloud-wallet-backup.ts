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
  public static async create({
    backupKey,
  }: IWalletCreateBackupData): Promise<number> {
    return await RNIndy.createWalletBackup(
      uuidv4(),
      backupKey,
    )
  }

  public static async send({
    handle,
    path,
  }: IWalletSendBackupData): Promise<number> {
    return await RNIndy.backupWalletBackup(
      handle,
      path,
    )
  }

  public static async updateState({
    handle,
  }: IWalletUpdateBackupStateData): Promise<number> {
    return await RNIndy.updateWalletBackupState(
      handle,
    )
  }

  public static async updateStateWithMessage({
    handle,
    message,
  }: IWalletUpdateBackupStateWithMessageData): Promise<number> {
    return await RNIndy.updateWalletBackupStateWithMessage(
      handle,
      message,
    )
  }

  public static async restore({
    config,
  }: IWalletRestoreData): Promise<number> {
    return await RNIndy.restoreWallet(
      config,
    )
  }

  public static async serialize({
    handle,
  }: IWalletSerializeBackupData): Promise<string> {
    return await RNIndy.serializeBackupWallet(
      handle,
    )
  }

  public static async deserialize({
    serialized,
  }: IWalletDeserializeBackupData): Promise<number> {
    return await RNIndy.deserializeBackupWallet(
      serialized,
    )
  }
}
