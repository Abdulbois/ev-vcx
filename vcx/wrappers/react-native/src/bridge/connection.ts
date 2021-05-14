import { NativeModules } from 'react-native'
import { v4 as uuidv4 } from 'uuid'

const { RNIndy } = NativeModules

interface IConnectionCreateData {
  goal: string,
  goalCode: string,
  handshake: boolean,
  attachment?: string | null,
}

interface IConnectionCreateWithInvitationData {
  invitation: string,
}

interface IConnectionCreateWithOutofbandInvitationData {
  invitation: string,
}

interface IConnectionConnectData {
  handle: number,
  options?: string | undefined | number,
}

interface IConnectionDeleteData {
  handle: number,
}

interface IConnectionSerializeData {
  handle: number,
}

interface IConnectionGetStateData {
  handle: number,
}

interface IConnectionUpdateStateData {
  handle: number,
}

interface IConnectionUpdateStateWithMessageData {
  handle: number,
  message: string,
}

interface IConnectionDeserializeData {
  serialized: string,
}

interface IConnectionSendMessageData {
  handle: number,
  message: string,
}

interface IConnectionSignData {
  handle: number,
  data: string,
  base64EncodingOption: string,
  encodeBeforeSigning: boolean
}

interface ISignDataResult {
  data: string,
  signature: string,
}

interface IConnectionVerifySignatureData {
  handle: number,
  data: string,
  signature: string,
}

interface IConnectionGetInvitationData {
  handle: number,
  abbr: boolean,
}

interface IConnectionSendReuseData {
  handle: number,
  invitation: string,
}

interface IConnectionRedirectData {
  handle: number,
  existingConnectionHandle: number,
}

interface IConnectionSendAnswerData {
  handle: number,
  question: string,
  answer: string,
}

interface IConnectionSendInviteActionData {
  handle: number,
  data: string,
}

interface IConnectionGetData {
  handle: number,
}

interface IConnectionAcceptConnectionInvite {
  invitationId: string,
  invite: string,
  options: string,
}

interface IConnectionRelease {
  handle: number,
}

interface IConnectionSendPing {
  handle: number,
  comment: string,
}

interface IConnectionSendDiscoveryFeatures {
  handle: number,
  comment: string,
  query: string,
}

interface IConnectionGetProblemReport {
  handle: number,
}

export class Connection {
  public static async createConnectionInvitation(): Promise<number> {
    return await RNIndy.createConnection(
      uuidv4(),
    )
  }

  public static async createOutOfBandConnectionInvitation({
    goal,
    goalCode,
    handshake,
    attachment,
  }: IConnectionCreateData): Promise<number> {
    return await RNIndy.createOutOfBandConnection(
      uuidv4(),
      goalCode,
      goal,
      handshake,
      attachment,
    )
  }

  public static async createWithInvitation({ invitation }: IConnectionCreateWithInvitationData): Promise<number> {
    return await RNIndy.createConnectionWithInvite(
      uuidv4(),
      invitation,
    )
  }

  public static async createWithOutofbandInvitation({ invitation }: IConnectionCreateWithOutofbandInvitationData): Promise<number> {
    return await RNIndy.createConnectionWithOutofbandInvite(
      uuidv4(),
      invitation,
    )
  }

  public static async connect({ handle, options }: IConnectionConnectData): Promise<number> {
    return await RNIndy.connectionConnect(
      handle,
      options,
    )
  }

  public static async getState({ handle }: IConnectionGetStateData): Promise<number> {
    return await RNIndy.connectionGetState(
      handle,
    )
  }

  public static async updateState({ handle }: IConnectionUpdateStateData): Promise<number> {
    return await RNIndy.connectionUpdateState(
      handle,
    )
  }

  public static async updateStateWithMessage({
    handle,
    message,
  }: IConnectionUpdateStateWithMessageData): Promise<number> {
    return await RNIndy.connectionUpdateStateWithMessage(
      handle,
      message,
    )
  }

  public static async sendMessage({ handle, message }: IConnectionSendMessageData): Promise<string> {
    return await RNIndy.connectionSendMessage(
      handle,
      message,
    )
  }

  public static async signData({
    handle,
    data,
    base64EncodingOption,
    encodeBeforeSigning,
  }: IConnectionSignData): Promise<ISignDataResult> {
    return await RNIndy.connectionSignData(
      handle,
      data,
      base64EncodingOption,
      encodeBeforeSigning,
    )
  }

  public static async verifySignature({
    handle,
    data,
    signature,
  }: IConnectionVerifySignatureData): Promise<boolean> {
    return await RNIndy.connectionVerifySignature(
      handle,
      data,
      signature,
    )
  }

  public static async getInvitation({
    handle,
    abbr,
  }: IConnectionGetInvitationData): Promise<string> {
    return await RNIndy.getConnectionInvite(
      handle,
      abbr,
    )
  }

  public static async reuse({
    handle,
    invitation,
  }: IConnectionSendReuseData): Promise<void> {
    return await RNIndy.connectionReuse(
      handle,
      invitation,
    )
  }

  public static async redirect({
    handle,
    existingConnectionHandle,
  }: IConnectionRedirectData): Promise<void> {
    return await RNIndy.connectionRedirect(
      handle,
      existingConnectionHandle,
    )
  }

  public static async sendAnswer({
    handle,
    question,
    answer,
  }: IConnectionSendAnswerData): Promise<string> {
    return await RNIndy.connectionSendAnswer(
      handle,
      question,
      answer,
    )
  }

  public static async sendInviteAction({
    handle,
    data,
  }: IConnectionSendInviteActionData): Promise<string> {
    return await RNIndy.connectionSendInviteAction(
      handle,
      data,
    )
  }

  public static async getRedirectDetails({
    handle,
  }: IConnectionGetData): Promise<string> {
    return await RNIndy.getRedirectDetails(
      handle,
    )
  }

  public static async getPwDid({
    handle,
  }: IConnectionGetData): Promise<string> {
    return await RNIndy.connectionGetPwDid(
      handle,
    )
  }

  public static async getTheirPwDid({
    handle,
  }: IConnectionGetData): Promise<string> {
    return await RNIndy.connectionGetTheirPwDid(
      handle,
    )
  }

  public static async getInfo({
    handle,
  }: IConnectionGetData): Promise<string> {
    return await RNIndy.connectionGetInfo(
      handle,
    )
  }

  public static async getProblemReportMessage({
    handle,
  }: IConnectionGetData): Promise<string> {
    return await RNIndy.connectionGetProblemReport(
      handle,
    )
  }

  public static async delete({ handle }: IConnectionDeleteData): Promise<void> {
    return await RNIndy.deleteConnection(
      handle,
    )
  }

  public static async serialize({ handle }: IConnectionSerializeData): Promise<string> {
    return await RNIndy.getSerializedConnection(
      handle,
    )
  }

  public static async deserialize({ serialized }: IConnectionDeserializeData): Promise<number> {
    return await RNIndy.deserializeConnection(
      serialized,
    )
  }

  public static async acceptInvite({
    invitationId,
    invite,
    options,
  }: IConnectionAcceptConnectionInvite): Promise<number> {
    return await RNIndy.vcxConnectionAcceptConnectionInvite(
      invitationId,
      invite,
      options
    )
  }

  public static async release({
    handle,
  }: IConnectionRelease): Promise<number> {
    return await RNIndy.connectionRelease(handle)
  }

  public static async sendPing({
    handle,
    comment,
  }: IConnectionSendPing): Promise<void> {
    return await RNIndy.connectionSendPing(handle, comment)
  }

  public static async sendDiscoveryFeatures({
    handle,
    comment,
    query,
  }: IConnectionSendDiscoveryFeatures): Promise<void> {
    return await RNIndy.connectionSendDiscoveryFeatures(handle, comment, query)
  }

  public static async getTheirDid({ handle }: IConnectionGetData): Promise<string> {
    return await RNIndy.connectionGetTheirDid(handle)
  }

  public static async info({ handle }: IConnectionGetData): Promise<string> {
    return await RNIndy.connectionInfo(handle)
  }

  public static async getProblemReport({ handle }: IConnectionGetProblemReport): Promise<string> {
    return await RNIndy.connectionGetProblemReport(handle)
  }
}
