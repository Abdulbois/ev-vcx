import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface IGetProvisionTokenData {
  agencyConfig: string,
}

interface IProvisionData {
  agencyConfig: string,
}

interface IProvisionWithTokenData {
  agencyConfig: string,
  token: string,
}

interface IUpdateAgentInfoData {
  config: string,
}

interface IDownloadAgentMessagesData {
  messageStatus: string,
  uids: string,
}

interface IDownloadMessagesData {
  messageStatus: string,
  uids: string,
  pwdids: string,
}

interface IUpdateMessagesData {
  messageStatus: string,
  pwdids: string,
}

export class Agent {
  public static async getProvisionToken({
    agencyConfig,
  }: IProvisionData): Promise<string> {
    return await RNIndy.getProvisionToken(
      agencyConfig,
    )
  }

  public static async provision({
    agencyConfig,
  }: IProvisionWithTokenData): Promise<string> {
    return await RNIndy.createOneTimeInfo(
      agencyConfig,
    )
  }

  public static async provisionWithToken({
    agencyConfig,
    token,
  }: IProvisionWithTokenData): Promise<string> {
    return await RNIndy.createOneTimeInfoWithToken(
      agencyConfig,
      token,
    )
  }

  public static async updateInfo({
    config,
  }: IUpdateAgentInfoData): Promise<void> {
    return await RNIndy.vcxUpdatePushToken(
      config,
    )
  }

  public static async downloadMessages({
    messageStatus,
    uids,
    pwdids,
  }: IDownloadMessagesData): Promise<string> {
    return await RNIndy.downloadMessages(
      messageStatus,
      uids,
      pwdids,
    )
  }

  public static async downloadAgentMessages({
    messageStatus,
    uids,
  }: IDownloadAgentMessagesData): Promise<string> {
    return await RNIndy.vcxGetAgentMessages(
      messageStatus,
      uids,
    )
  }

  public static async updateMessages({
    messageStatus,
    pwdids,
  }: IUpdateMessagesData): Promise<number> {
    return await RNIndy.updateMessages(
      messageStatus,
      pwdids,
    )
  }

  public static async createPairwiseAgent(): Promise<string> {
    return await RNIndy.createPairwiseAgent()
  }
}
