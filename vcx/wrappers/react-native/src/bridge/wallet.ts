import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface IWalletKeyData {
  lengthOfKey: number,
}

interface IWalletExportData {
  exportPath: string,
  encryptionKey: string,
}

interface IWalletCopyToPathData {
  uri: string,
  zipPath: string,
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

export class Wallet {
  public static async creatKey({
    lengthOfKey,
  }: IWalletKeyData): Promise<string> {
    return await RNIndy.createWalletKey(
      lengthOfKey,
    )
  }

  public static async copyToPath({
    uri,
    zipPath,
  }: IWalletCopyToPathData): Promise<number> {
    return await RNIndy.copyToPath(
      uri,
      zipPath,
    )
  }

  public static async export({
    exportPath,
    encryptionKey,
  }: IWalletExportData): Promise<number> {
    return await RNIndy.exportWallet(
      exportPath,
      encryptionKey,
    )
  }

  public static async import({
    config,
  }: IWalletImportData): Promise<number> {
    return await RNIndy.decryptWalletFile(
      config,
    )
  }

  public static async setItem({
    key,
    value,
  }: IWalletSetItemData): Promise<number> {
    return await RNIndy.setWalletItem(
      key,
      value,
    )
  }

  public static async getItem({
    key,
  }: IWalletGetItemData): Promise<string> {
    return await RNIndy.getWalletItem(
      key,
    )
  }

  public static async deleteItem({
    key,
  }: IWalletDeleteItemData): Promise<number> {
    return await RNIndy.deleteWalletItem(
      key,
    )
  }

  public static async updateItem({
    key,
    value,
  }: IWalletUpdateItemData): Promise<number> {
    return await RNIndy.updateWalletItem(
      key,
      value,
    )
  }

  public static async getTokenInfo({
    paymentHandle,
  }: IWalletGetTokenInfoData): Promise<string> {
    return await RNIndy.getTokenInfo(
      paymentHandle,
    )
  }

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

  public static async createPaymentAddress({
    seed,
  }: IWalletCreatePaymentAddressData): Promise<string> {
    return await RNIndy.createPaymentAddress(
      seed,
    )
  }
}
