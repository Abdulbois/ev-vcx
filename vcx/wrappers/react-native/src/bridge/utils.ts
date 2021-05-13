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

export class Utils {
  public static async getLedgerFees(): Promise<string> {
    return await RNIndy.getLedgerFees()
  }

  public static async getLedgerAuthorAgreement(): Promise<string> {
    return await RNIndy.getTxnAuthorAgreement()
  }

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
}
