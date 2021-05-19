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

interface ICredentialGetRequestMsg {
  handle: number,
  myPwDid: string,
  theirPwDid: string,
  paymentHandle: number,
}

interface ICredentialAcceptCredentialOffer {
  offer: string,
  handle: number,
}

interface ICredentialGetProblemReport {
  handle: number,
}

export class Credential {
  /**
   * Create a Credential object that requests and receives a credential for an institution
   *
   * @param  sourceId         Institution's personal identification for the credential, should be unique.
   * @param  credentialOffer  Received Credential Offer message.
   *                          The format of Credential Offer depends on communication method:
   *                          <pre>
   *                          {@code
   *                              proprietary:
     *                                      "[{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]"
   *                              aries:
   *                                      "{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential", "@id":"<uuid-of-offer-message>", "comment":"somecomment", "credential_preview":<json-ldobject>, "offers~attach":[{"@id":"libindy-cred-offer-0", "mime-type":"application/json", "data":{"base64":"<bytesforbase64>"}}]}"
   *                          }
   *                          </pre>
   *
   * @return                      handle that should be used to perform actions with the Credential object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async createWithOffer({
    offer,
  }: ICreateWithOfferData): Promise<number> {
    return await RNIndy.credentialCreateWithOffer(
      uuidv4(),
      offer,
    )
  }

  /**
   * Queries agency for Credential Offer messages from the given connection.
   *
   * @param  connectionHandle     handle pointing to Connection object to query for credential offers.
   *
   * @return                      List of received Credential Offers as JSON string.
   *                              "[[{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]]"
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getOffers({ connectionHandle }: IGetOffersData): Promise<string> {
    return await RNIndy.credentialGetOffers(
      connectionHandle,
    )
  }

  /**
   * Get the current state of the Credential object
   * Credential states:
   *     2 - Credential Request Sent
   *     3 - Credential Offer Received
   *     4 - Credential Accepted
   *
   * @param  credentialHandle     handle pointing to a Credential object.
   *
   * @return                      the most current state of the Credential object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getState({ handle }: ICredentialUpdateStateData): Promise<number> {
    return await RNIndy.getClaimOfferState(
      handle,
    )
  }

  /**
   * Query the agency for the received messages.
   * Checks for any messages changing state in the Credential object and updates the state attribute.
   * If it detects a credential it will store the credential in the wallet.
   *
   * @param  credentialHandle     handle pointing to a Credential object.
   *
   * @return                      the most current state of the Credential object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async updateState({ handle }: ICredentialGetStateData): Promise<number> {
    return await RNIndy.updateClaimOfferState(
      handle,
    )
  }

  /**
   * Update the state of the Credential object based on the given message.
   *
   * @param  credentialHandle     handle pointing to a Credential object.
   * @param  message              message to process for any Credential state transitions.
   *
   * @return                      the most current state of the Credential object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async updateStateWithMessage({
    handle,
    message,
  }: ICredentialUpdateStateWithMessageData): Promise<number> {
    return await RNIndy.updateClaimOfferStateWithMessage(
      handle,
      message,
    )
  }

  /**
   * Approves the Credential Offer and submits a Credential Request.
   *
   * @param  credentialHandle     handle pointing to a Credential object.
   * @param  connectionHandle     handle pointing to a Connection object.
   * @param  paymentHandle        deprecated parameter (use 0).
   *
   * @return                      void
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
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

  /**
   * Send a Credential rejection to the connection.
   * It can be called once Credential Offer or Credential messages are received.
   *
   * Note that this function can be used for `aries` communication protocol.
   * In other cases it returns ActionNotSupported error.
   *
   * @param  credentialHandle     handle pointing to a Credential object.
   * @param  connectionHandle     handle pointing to a Connection identifying pairwise connection..
   * @param  comment              (Optional) human-friendly message to insert into Reject message.
   *
   * @return                      void
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
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

  /**
   * Delete a Credential associated with the state object from the Wallet and release handle of the state object.
   *
   * @param  credentialHandle     handle pointing to credential state object to delete.
   *
   * @return                      void
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async delete({
    handle,
  }: ICredentialDeleteData): Promise<void> {
    return await RNIndy.deleteCredential(
      handle,
    )
  }

  /**
   * Retrieve information about a stored credential in user's wallet, including credential id and the credential itself.
   *
   * @param  credentialHandle     handle pointing to a Credential object.
   *
   * @return                      Credential message as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getCredentialMessage({
    handle,
  }: ICredentialGetCredentialMessageData): Promise<string> {
    return await RNIndy.getClaimVcx(
      handle,
    )
  }

  /**
   * Build Presentation Proposal message for revealing Credential data.
   *
   * Presentation Proposal is an optional message that can be sent by the Prover to the Verifier to
   * initiate a Presentation Proof process.
   *
   * Presentation Proposal Format: https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof#propose-presentation
   *
   * EXPERIMENTAL
   *
   * @param  credentialHandle     handle pointing to Credential to use for Presentation Proposal message building
   *
   * @return                      Presentation Proposal message as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getPresentationProposalMessage({
    handle,
  }: ICredentialGetPresentationProposalData): Promise<string> {
    return await RNIndy.credentialGetPresentationProposal(
      handle,
    )
  }

  /**
   * Get Problem Report message for object in Failed or Rejected state.
   *
   * @param  credentialHandle handle pointing to Credential state object.
   *
   * @return                  Problem Report as JSON string or null
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async getProblemReportMessage({
    handle,
  }: ICredentialGetData): Promise<string> {
    return await RNIndy.connectionGetProblemReport(
      handle,
    )
  }

  /**
   * Get JSON string representation of Credential object.
   *
   * @param  credentialHandle     handle pointing to a Credential object.
   *
   * @return                      Credential object as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async serialize({ handle }: ICredentialSerializeData): Promise<string> {
    return await RNIndy.serializeClaimOffer(
      handle,
    )
  }

  /**
   * Takes a json string representing a Credential object and recreates an object matching the JSON.
   *
   * @param  serializedCredential JSON string representing a Credential object.
   *
   * @return                      handle that should be used to perform actions with the Credential object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async deserialize({ serialized }: ICredentialDeserializeData): Promise<number> {
    return await RNIndy.deserializeClaimOffer(
      serialized,
    )
  }

  /**
   * Approves the Credential Offer and gets the Credential Request message that can be sent to the specified connection
   *
   * @param  credentialHandle     handle pointing to a Credential object.
   * @param  myPwDid              pairwise DID used for Connection.
   * @param  theirPwDid           pairwise DID of the remote side used for Connection.
   * @param  paymentHandle        deprecated parameter (use 0).
   *
   * @return                      Credential Request message as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getRequestMsg({
    handle,
    myPwDid,
    theirPwDid,
    paymentHandle,
  }: ICredentialGetRequestMsg): Promise<string> {
    return await RNIndy.credentialGetRequestMsg(
      handle,
      myPwDid,
      theirPwDid,
      paymentHandle
    )
  }

  /**
   * Get Problem Report message for object in Failed or Rejected state.
   *
   * @param  credentialHandle handle pointing to Credential state object.
   *
   * @return                  Problem Report as JSON string or null
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async getProblemReport({ handle }: ICredentialGetProblemReport): Promise<string> {
    return await RNIndy.credentialGetProblemReport(handle)
  }
}
