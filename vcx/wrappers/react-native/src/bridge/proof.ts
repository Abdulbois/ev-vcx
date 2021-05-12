import { NativeModules } from 'react-native'
import { v4 as uuidv4 } from 'uuid'

const { RNIndy } = NativeModules

interface IProofCreateWithRequestData {
  proofRequest: string,
}

interface IProofGetRequestsData {
  connectionHandle: number,
}

interface IProofGetCredentialsData {
  handle: number,
}

interface IProofSendProofData {
  handle: number,
  connectionHandle: number,
}

interface IProofRejectData {
  handle: number,
  connectionHandle: number,
}

interface IProofGetStateData {
  handle: number,
}

interface IProofUpdateStateData {
  handle: number,
}

interface IProofUpdateStateWithMessageData {
  handle: number,
  message: string,
}

interface IProofGenerateData {
  handle: number,
  selectedCredentials: string,
  selfAttestedAttributes: string,
}

interface IProofDeclineData {
  handle: number,
  connectionHandle: number,
  reason?: string,
  proposal?: string,
}

interface IProofGetData {
  handle: number,
}

interface IProofSerializeData {
  handle: number,
}

interface IProofDeserializeData {
  serialized: string,
}

export class Proof {
  public static async createWithRequest({
    proofRequest,
  }: IProofCreateWithRequestData): Promise<number> {
    return await RNIndy.proofCreateWithRequest(
      uuidv4(),
      proofRequest,
    )
  }

  public static async getRequests({ connectionHandle }: IProofGetRequestsData): Promise<string> {
    return await RNIndy.proofGetRequests(
      connectionHandle,
    )
  }

  public static async getCredentialsForProofRequest({ handle }: IProofGetCredentialsData): Promise<string> {
    return await RNIndy.proofRetrieveCredentials(
      handle,
    )
  }

  public static async getState({ handle }: IProofGetStateData): Promise<number> {
    return await RNIndy.proofGetState(
      handle,
    )
  }

  public static async updateState({ handle }: IProofUpdateStateData): Promise<number> {
    return await RNIndy.proofUpdateState(
      handle,
    )
  }

  public static async updateStateWithMessage({
    handle,
    message,
  }: IProofUpdateStateWithMessageData): Promise<number> {
    return await RNIndy.proofUpdateStateWithMessage(
      handle,
      message,
    )
  }

  public static async generateProof({
    handle,
    selectedCredentials,
    selfAttestedAttributes,
  }: IProofGenerateData): Promise<void> {
    return await RNIndy.proofGenerate(
      handle,
      selectedCredentials,
      selfAttestedAttributes,
    )
  }

  public static async sendProof({ handle, connectionHandle }: IProofSendProofData): Promise<void> {
    return await RNIndy.proofSend(
      handle,
      connectionHandle,
    )
  }

  public static async reject({ handle, connectionHandle }: IProofRejectData): Promise<void> {
    return await RNIndy.proofReject(
      handle,
      connectionHandle,
    )
  }

  public static async decline({ handle, connectionHandle, reason, proposal }: IProofDeclineData): Promise<void> {
    return await RNIndy.proofDeclineRequest(
      handle,
      connectionHandle,
      reason,
      proposal,
    )
  }

  public static async getProblemReportMessage({
    handle,
  }: IProofGetData): Promise<string> {
    return await RNIndy.proofGetProblemReport(
      handle,
    )
  }

  public static async serialize({ handle }: IProofSerializeData): Promise<string> {
    return await RNIndy.proofSerialize(
      handle,
    )
  }

  public static async deserialize({ serialized }: IProofDeserializeData): Promise<number> {
    return await RNIndy.proofDeserialize(
      serialized,
    )
  }
}
