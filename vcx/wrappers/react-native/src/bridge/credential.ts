import { NativeModules } from 'react-native'
import { v4 as uuidv4 } from 'uuid'

const { RNIndy } = NativeModules

interface ICreateWithOfferData {
  offer: string,
}

interface IGetOffersData {
  connectionHandle: number,
}

interface ICredentialGetStateData {
  handle: number,
}

interface ICredentialUpdateStateData {
  handle: number,
}

interface ICredentialUpdateStateWithMessageData {
  handle: number,
  message: string,
}

interface ICredentialSendRequestData {
  handle: number,
  connectionHandle: number,
  paymentHandle?: number,
}

interface ICredentialGetCredentialMessageData {
  handle: number,
}

interface ICredentialRejectData {
  handle: number,
  connectionHandle: number,
  comment?: string | undefined | number,
}

interface ICredentialDeleteData {
  handle: number,
}

interface ICredentialGetPresentationProposalData {
  handle: number,
}

interface ICredentialGetData {
  handle: number,
}

interface ICredentialSerializeData {
  handle: number,
}

interface ICredentialDeserializeData {
  serialized: string,
}

export class Credential {
  public static async createWithOffer({
    offer,
  }: ICreateWithOfferData): Promise<number> {
    return await RNIndy.credentialCreateWithOffer(
      uuidv4(),
      offer,
    )
  }

  public static async getOffers({ connectionHandle }: IGetOffersData): Promise<string> {
    return await RNIndy.credentialGetOffers(
      connectionHandle,
    )
  }

  public static async getState({ handle }: ICredentialUpdateStateData): Promise<number> {
    return await RNIndy.getClaimOfferState(
      handle,
    )
  }

  public static async updateState({ handle }: ICredentialGetStateData): Promise<number> {
    return await RNIndy.updateClaimOfferState(
      handle,
    )
  }

  public static async updateStateWithMessage({
    handle,
    message,
  }: ICredentialUpdateStateWithMessageData): Promise<number> {
    return await RNIndy.updateClaimOfferStateWithMessage(
      handle,
      message,
    )
  }

  public static async sendRequest({
    handle,
    connectionHandle,
    paymentHandle,
  }: ICredentialSendRequestData): Promise<void> {
    return await RNIndy.sendClaimRequest(
      handle,
      connectionHandle,
      paymentHandle,
    )
  }

  public static async reject({
    handle,
    connectionHandle,
    comment,
  }: ICredentialRejectData): Promise<void> {
    return await RNIndy.credentialReject(
      handle,
      connectionHandle,
      comment,
    )
  }

  public static async delete({
    handle,
  }: ICredentialDeleteData): Promise<void> {
    return await RNIndy.deleteCredential(
      handle,
    )
  }

  public static async getCredentialMessage({
    handle,
  }: ICredentialGetCredentialMessageData): Promise<string> {
    return await RNIndy.getClaimVcx(
      handle,
    )
  }

  public static async getPresentationProposalMessage({
    handle,
  }: ICredentialGetPresentationProposalData): Promise<string> {
    return await RNIndy.credentialGetPresentationProposal(
      handle,
    )
  }

  public static async getProblemReportMessage({
    handle,
  }: ICredentialGetData): Promise<string> {
    return await RNIndy.connectionGetProblemReport(
      handle,
    )
  }

  public static async serialize({ handle }: ICredentialSerializeData): Promise<string> {
    return await RNIndy.serializeClaimOffer(
      handle,
    )
  }

  public static async deserialize({ serialized }: ICredentialDeserializeData): Promise<number> {
    return await RNIndy.deserializeClaimOffer(
      serialized,
    )
  }
}
