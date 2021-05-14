import { NativeModules } from 'react-native'
import { v4 as uuidv4 } from 'uuid'

const { RNIndy } = NativeModules

interface IVerifierCreateData {
  requestedAttrs: string,
  requestedPredicates: string,
  revocationInterval: string,
  name: string,
}

interface IVerifierCreateWithProposalData {
  presentationProposal: string,
  name: string,
}

interface IVerifierGetStateData {
  handle: number,
}

interface IVerifierUpdateStateData {
  handle: number,
}

interface IVerifierUpdateStateWithMessageData {
  handle: number,
  message: string,
}

interface IVerifierRequestProofData {
  handle: number,
  connectionHandle: number,
}

interface IVerifierGetData {
  handle: number,
}

interface IVerifierSerializeData {
  handle: number,
}

interface IVerifierDeserializeData {
  serialized: string,
}

interface IVerifierRequestPresentation {
  handle: number,
  connectionHandle: number,
  requestedAttrs: string,
  requestedPredicates: string,
  revocationInterval: string,
  name: string,
}

interface IVerifierGetProofProposal {
  handle: number,
}

interface IVerifierProofAccepted {
  handle: number,
  data: string,
}

interface IVerifierProofRelease {
  handle: number,
}

export class Verifier {
  public static async create({
    requestedAttrs,
    requestedPredicates,
    revocationInterval,
    name,
  }: IVerifierCreateData): Promise<number> {
    return await RNIndy.createProofVerifier(
      uuidv4(),
      requestedAttrs,
      requestedPredicates,
      revocationInterval,
      name,
    )
  }

  public static async createWithProposal({
    presentationProposal,
    name,
  }: IVerifierCreateWithProposalData): Promise<number> {
    return await RNIndy.createProofVerifierWithProposal(
      uuidv4(),
      presentationProposal,
      name,
    )
  }

  public static async getState({ handle }: IVerifierGetStateData): Promise<number> {
    return await RNIndy.proofVerifierGetState(
      handle,
    )
  }

  public static async updateState({ handle }: IVerifierUpdateStateData): Promise<number> {
    return await RNIndy.proofVerifierUpdateState(
      handle,
    )
  }

  public static async updateStateWithMessage({
    handle,
    message,
  }: IVerifierUpdateStateWithMessageData): Promise<number> {
    return await RNIndy.proofVerifierUpdateStateWithMessage(
      handle,
      message,
    )
  }

  public static async sendProofRequest({ handle, connectionHandle }: IVerifierRequestProofData): Promise<void> {
    return await RNIndy.proofVerifierSendRequest(
      handle,
      connectionHandle,
    )
  }

  public static async getProofRequestMessage({
    handle,
  }: IVerifierGetData): Promise<string> {
    return await RNIndy.proofVerifierGetPresentationRequest(
      handle,
    )
  }

  public static async getProofMessage({
    handle,
  }: IVerifierGetData): Promise<string> {
    return await RNIndy.proofVerifierGetProofMessage(
      handle,
    )
  }

  public static async getProblemReportMessage({
    handle,
  }: IVerifierGetData): Promise<string> {
    return await RNIndy.proofVerifierGetProblemReport(
      handle,
    )
  }

  public static async serialize({ handle }: IVerifierSerializeData): Promise<string> {
    return await RNIndy.proofVerifierSerialize(
      handle,
    )
  }

  public static async deserialize({ serialized }: IVerifierDeserializeData): Promise<number> {
    return await RNIndy.proofVerifierDeserialize(
      serialized,
    )
  }

  public static async requestPresentation({
    handle,
    connectionHandle,
    requestedAttrs,
    requestedPredicates,
    revocationInterval,
    name,
  }: IVerifierRequestPresentation): Promise<void> {
    return await RNIndy.proofVerifierRequestPresentation(
      handle,
      connectionHandle,
      requestedAttrs,
      requestedPredicates,
      revocationInterval,
      name
    )
  }

  public static async getProofProposal({
    handle
  }: IVerifierGetProofProposal): Promise<string> {
    return await RNIndy.proofVerifierGetProofProposal(handle)
  }

  public static async proofAccepted({
    handle,
    data,
  }: IVerifierProofAccepted): Promise<number> {
    return await RNIndy.proofVerifierProofAccepted(
      handle,
      data
    )
  }

  public static async release({
    handle
  }: IVerifierProofRelease): Promise<void> {
    return await RNIndy.proofVerifierProofRelease(handle)
  }
}
