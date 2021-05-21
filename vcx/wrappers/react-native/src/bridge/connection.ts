import { NativeModules } from 'react-native'
import { v4 as uuidv4 } from 'uuid'

const { RNIndy } = NativeModules

interface IConnectionCreateData {
  goal?: string | null
  goalCode?: string | null
  handshake: boolean
  attachment?: string | null
}

interface IConnectionCreateWithInvitationData {
  invitation: string
}

interface IConnectionCreateWithOutofbandInvitationData {
  invitation: string
}

interface IConnectionConnectData {
  handle: number
  options?: string | undefined | number
}

interface IConnectionDeleteData {
  handle: number
}

interface IConnectionSerializeData {
  handle: number
}

interface IConnectionGetStateData {
  handle: number
}

interface IConnectionUpdateStateData {
  handle: number
}

interface IConnectionUpdateStateWithMessageData {
  handle: number
  message: string
}

interface IConnectionDeserializeData {
  serialized: string
}

interface IConnectionSendMessageData {
  handle: number
  message: string
  options: string
}

interface IConnectionSignData {
  handle: number
  data: string
  base64EncodingOption: string
  encodeBeforeSigning: boolean
}

interface ISignDataResult {
  data: string
  signature: string
}

interface IConnectionVerifySignatureData {
  handle: number
  data: string
  signature: string
}

interface IConnectionGetInvitationData {
  handle: number
  abbr: boolean
}

interface IConnectionSendReuseData {
  handle: number
  invitation: string
}

interface IConnectionRedirectData {
  handle: number
  existingConnectionHandle: number
}

interface IConnectionSendAnswerData {
  handle: number
  question: string
  answer: string
}

interface IConnectionSendInviteActionData {
  handle: number
  data: string
}

interface IConnectionGetData {
  handle: number
}

interface IConnectionSendPing {
  handle: number
  comment: string
}

export class Connection {
  /**
   * Create a Connection object that provides a pairwise connection for an institution's user.
   *
   * @return              handle that should be used to perform actions with the Connection object.
   *
   * @throws VcxException If an exception occurred in Libvcx library.
   */
  public static async createConnectionInvitation(): Promise<number> {
    return await RNIndy.createConnection(uuidv4())
  }

  /**
   * Create a Connection object that provides an Out-of-Band Connection for an institution's user.
   * NOTE: this method can be used when `aries` protocol is set.
   * NOTE: this method is EXPERIMENTAL
   * WARN: `requestAttach` field is not fully supported in the current library state.
   *        You can use simple messages like Question but it cannot be used
   *        for Credential Issuance and Credential Presentation.
   *
   * @param  goalCode     a self-attested code the receiver may want to display to
   *                      the user or use in automatically deciding what to do with the out-of-band message.
   * @param  goal         a self-attested string that the receiver may want to display to the user about
   *                      the context-specific goal of the out-of-band message.
   * @param  handshake    whether Inviter wants to establish regular connection using `connections` handshake protocol.
   *                      if false, one-time connection channel will be created.
   * @param  attachment  An additional message as JSON that will be put into attachment decorator
   *                        that the receiver can using in responding to the message (for example Question message).
   *
   * @return              handle that should be used to perform actions with the Connection object.
   *
   * @throws VcxException If an exception occurred in Libvcx library.
   */
  public static async createOutOfBandConnectionInvitation({
    goal,
    goalCode,
    handshake,
    attachment,
  }: IConnectionCreateData): Promise<number> {
    return await RNIndy.createOutOfBandConnection(uuidv4(), goalCode, goal, handshake, attachment)
  }

  /**
   * Create a Connection object from the given Invitation that provides a pairwise connection.
   *
   * @param  invitation A string representing a json object which is provided by an entity that wishes to make a connection.
   *                       The format depends on used communication protocol:
   *                          proprietary:
   *                              "{"targetName": "", "statusMsg": "message created", "connReqId": "mugIkrWeMr", "statusCode": "MS-101", "threadId": null, "senderAgencyDetail": {"endpoint": "http://localhost:8080", "verKey": "key", "DID": "did"}, "senderDetail": {"agentKeyDlgProof": {"agentDID": "8f6gqnT13GGMNPWDa2TRQ7", "agentDelegatedKey": "5B3pGBYjDeZYSNk9CXvgoeAAACe2BeujaAkipEC7Yyd1", "signature": "TgGSvZ6+/SynT3VxAZDOMWNbHpdsSl8zlOfPlcfm87CjPTmC/7Cyteep7U3m9Gw6ilu8SOOW59YR1rft+D8ZDg=="}, "publicDID": "7YLxxEfHRiZkCMVNii1RCy", "name": "Faber", "logoUrl": "http://robohash.org/234", "verKey": "CoYZMV6GrWqoG9ybfH3npwH3FnWPcHmpWYUF8n172FUx", "DID": "Ney2FxHT4rdEyy6EDCCtxZ"}}"
   *                          aries:
   *                              "{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation","label":"Alice","recipientKeys":["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"],"serviceEndpoint":"https://example.com/endpoint","routingKeys":["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K"]}"
   *
   * @return               handle that should be used to perform actions with the Connection object.
   *
   * @throws VcxException  If an exception occurred in Libvcx library.
   */
  public static async createWithInvitation({ invitation }: IConnectionCreateWithInvitationData): Promise<number> {
    return await RNIndy.createConnectionWithInvite(uuidv4(), invitation)
  }

  /**
   * Create a Connection object from the given Out-of-Band Invitation.
   * Depending on the format of Invitation there are two way of follow interaction:
   *     * Invitation contains `handshake_protocols`: regular Connection process will be ran.
   *         Follow steps as for regular Connection establishment.
   *     * Invitation does not contain `handshake_protocols`: one-time completed Connection object will be created.
   *         You can use `vcx_connection_send_message` or specific function to send a response message.
   *         Note that on repeated message sending an error will be thrown.
   *
   * NOTE: this method can be used when `aries` protocol is set.
   *
   * WARN: The user has to analyze the value of "request~attach" field yourself and
   *       create/handle the correspondent state object or send a reply once the connection is established.
   *
   * @param  invitation    A JSON string representing Out-of-Band Invitation provided by an entity that wishes interaction.
   *                  {
   *                      "@type": "https://didcomm.org/out-of-band/%VER/invitation",
   *                      "@id": "<id used for context as pthid>", -  the unique ID of the message.
   *                      "label": Optional<string>, - a string that the receiver may want to display to the user,
   *                                                  likely about who sent the out-of-band message.
   *                      "goal_code": Optional<string>, - a self-attested code the receiver may want to display to
   *                                                      the user or use in automatically deciding what to do with the out-of-band message.
   *                      "goal": Optional<string>, - a self-attested string that the receiver may want to display to the user
   *                                                  about the context-specific goal of the out-of-band message.
   *                      "handshake_protocols": Optional<[string]>, - an array of protocols in the order of preference of the sender
   *                                                                  that the receiver can use in responding to the message in order to create or reuse a connection with the sender.
   *                                                                  One or both of handshake_protocols and request~attach MUST be included in the message.
   *                      "request~attach": Optional<[
   *                          {
   *                              "@id": "request-0",
   *                              "mime-type": "application/json",
   *                              "data": {
   *                                  "json": "<json of protocol message>"
   *                              }
   *                          }
   *                      ]>, - an attachment decorator containing an array of request messages in order of preference that the receiver can using in responding to the message.
   *                            One or both of handshake_protocols and request~attach MUST be included in the message.
   *                      "service": [
   *                          {
   *                              "id": string
   *                              "type": string,
   *                              "recipientKeys": [string],
   *                              "routingKeys": [string],
   *                              "serviceEndpoint": string
   *                          }
   *                      ] - an item that is the equivalent of the service block of a DIDDoc that the receiver is to use in responding to the message.
   *                  }
   *
   * @return               handle that should be used to perform actions with the Connection object.
   *
   * @throws VcxException  If an exception occurred in Libvcx library.
   */
  public static async createWithOutofbandInvitation({
    invitation,
  }: IConnectionCreateWithOutofbandInvitationData): Promise<number> {
    return await RNIndy.createConnectionWithOutofbandInvite(uuidv4(), invitation)
  }

  /**
   * Establishes connection between institution and its user.
   *
   * @param  connectionHandle  handle pointing to a Connection object.
   * @param  options: details about establishing connection
   *     {
   *         "connection_type": Option<"string"> - one of "SMS", "QR",
   *         "phone": "string": Option<"string"> - phone number in case "connection_type" is set into "SMS",
   *         "update_agent_info": Option<bool> - whether agent information needs to be updated.
   *                                             default value for `update_agent_info`=true
   *                                             if agent info does not need to be updated, set `update_agent_info`=false
   *         "use_public_did": Option<bool> - whether to use public DID for an establishing connection
   *                                          default value for `use_public_did`=false
   *         "pairwise_agent_info": Optional<JSON object> - pairwise agent to use instead of creating a new one.
   *                                                        Can be received by calling `vcx_create_pairwise_agent` function.
   *                                                         {
   *                                                             "pw_did": string,
   *                                                             "pw_vk": string,
   *                                                             "agent_did": string,
   *                                                             "agent_vk": string,
   *                                                         }
   *     }
   *
   * @return                   Connection Invite as JSON string.
   *
   * @throws VcxException      If an exception occurred in Libvcx library.
   */
  public static async connect({ handle, options }: IConnectionConnectData): Promise<number> {
    return await RNIndy.connectionConnect(handle, options)
  }

  /**
   * Returns the current internal state of the Connection object.
   * Possible states:
   *         1 - Initialized
   *         2 - Connection Request Sent
   *         3 - Connection Response Received
   *         4 - Connection Accepted
   *
   * @param  connectionHandle handle pointing to a Connection object.
   *
   * @return                  the most current state of the Connection object.
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async getState({ handle }: IConnectionGetStateData): Promise<number> {
    return await RNIndy.connectionGetState(handle)
  }

  /**
   * Query the agency for the received messages.
   * Checks for any messages changing state in the Connection and updates the state attribute.
   *
   * @param  connectionHandle  handle pointing to a Connection object.
   *
   * @return                   the most current state of the Connection object.
   *
   * @throws VcxException      If an exception occurred in Libvcx library.
   */
  public static async updateState({ handle }: IConnectionUpdateStateData): Promise<number> {
    return await RNIndy.connectionUpdateState(handle)
  }

  /**
   * Update the state of the Connection object based on the given message.
   *
   * @param  connectionHandle  handle pointing to a Connection object.
   * @param  message           message to process for any Connection state transitions.
   *
   * @return                   the most current state of the Connection object.
   *
   * @throws VcxException      If an exception occurred in Libvcx library.
   */
  public static async updateStateWithMessage({
    handle,
    message,
  }: IConnectionUpdateStateWithMessageData): Promise<number> {
    return await RNIndy.connectionUpdateStateWithMessage(handle, message)
  }

  /**
   * Send a generic message to the pairwise connection.
   *
   * @param  connectionHandle     handle pointing to a Connection object.
   * @param  message              actual message to send
   *
   * @return                      id of sent message
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async sendMessage({ handle, message, options }: IConnectionSendMessageData): Promise<string> {
    return await RNIndy.connectionSendMessage(handle, message, options)
  }

  /**
   * Generate a signature for the specified data using Connection pairwise keys.
   *
   * @param  connectionHandle     handle pointing to a Connection object.
   * @param  data                 raw data buffer for signature
   * @param  data          length of data buffer
   *
   * @return                      generated signature bytes
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async signData({
    handle,
    data,
    base64EncodingOption,
    encodeBeforeSigning,
  }: IConnectionSignData): Promise<ISignDataResult> {
    return await RNIndy.connectionSignData(handle, data, base64EncodingOption, encodeBeforeSigning)
  }

  /**
   * Verify the signature is valid for the specified data using Connection pairwise keys.
   *
   * @param  connectionHandle     handle pointing to a Connection object.
   * @param  data                 raw data buffer for signature
   * @param  signature            signature generate for raw data
   *
   * @return                      bool whether the signature was valid or not
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async verifySignature({ handle, data, signature }: IConnectionVerifySignatureData): Promise<boolean> {
    return await RNIndy.connectionVerifySignature(handle, data, signature)
  }

  /**
   * Get the invite details that were sent or can be sent to the remote side.
   *
   * @param  connectionHandle handle pointing to a Connection object.
   * @param  abbr     abbreviated connection details for QR codes or not (applicable for `proprietary` communication method only)
   *
   * @return                  Connection Invitation as JSON string
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async getInvitation({ handle, abbr }: IConnectionGetInvitationData): Promise<string> {
    return await RNIndy.getConnectionInvite(handle, abbr)
  }

  /**
   * Send a message to reuse existing Connection instead of setting up a new one as response on received Out-of-Band Invitation.
   * <p>
   * Note that this function works in case `aries` communication method is used.
   * In other cases it returns ActionNotSupported error.
   *
   * @param  connectionHandle handle pointing to a Connection object to awaken and send reuse message.
   * @param  invitation           A JSON string representing Out-of-Band Invitation provided by an entity that wishes interaction.
   *                          {
   *                              "@type": "https://didcomm.org/out-of-band/%VER/invitation",
   *                              "@id": "<id used for context as pthid>", -  the unique ID of the message.
   *                              "label": Optional<string>, - a string that the receiver may want to display to the user,
   *                                                          likely about who sent the out-of-band message.
   *                              "goal_code": Optional<string>, - a self-attested code the receiver may want to display to
   *                                                              the user or use in automatically deciding what to do with the out-of-band message.
   *                              "goal": Optional<string>, - a self-attested string that the receiver may want to display to the user
   *                                                          about the context-specific goal of the out-of-band message.
   *                              "handshake_protocols": Optional<[string]>, - an array of protocols in the order of preference of the sender
   *                                                                          that the receiver can use in responding to the message in order to create or reuse a connection with the sender.
   *                                                                          One or both of handshake_protocols and request~attach MUST be included in the message.
   *                              "request~attach": Optional<[
   *                                  {
   *                                      "@id": "request-0",
   *                                      "mime-type": "application/json",
   *                                      "data": {
   *                                          "json": "<json of protocol message>"
   *                                      }
   *                                  }
   *                              ]>, - an attachment decorator containing an array of request messages in order of preference that the receiver can using in responding to the message.
   *                                  One or both of handshake_protocols and request~attach MUST be included in the message.
   *                              "service": [
   *                                  {
   *                                      "id": string
   *                                      "type": string,
   *                                      "recipientKeys": [string],
   *                                      "routingKeys": [string],
   *                                      "serviceEndpoint": string
   *                                  }
   *                              ] - an item that is the equivalent of the service block of a DIDDoc that the receiver is to use in responding to the message.
   *                          }
   *
   * @return                  void
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async reuse({ handle, invitation }: IConnectionSendReuseData): Promise<void> {
    return await RNIndy.connectionReuse(handle, invitation)
  }

  /**
   * Redirect Connection
   *
   * @param  connectionHandle            handle pointing to a Connection object.
   * @param  existingConnectionHandle    handle pointing to a new Connection object.
   *
   * @return                   void
   *
   * @throws VcxException      If an exception occurred in Libvcx library.
   */
  public static async redirect({ handle, existingConnectionHandle }: IConnectionRedirectData): Promise<void> {
    return await RNIndy.connectionRedirect(handle, existingConnectionHandle)
  }

  /**
   * Send answer on received question message according to Aries question-answer protocol.
   * <p>
   * Note that this function works in case `aries` communication method is used.
   * In other cases it returns ActionNotSupported error.
   *
   * @param  connectionHandle handle pointing to a Connection object to send answer message.
   * @param  question         A JSON string representing Question received via pairwise connection.
   *                          {
   *                              "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/questionanswer/1.0/question",
   *                              "@id": "518be002-de8e-456e-b3d5-8fe472477a86",
   *                              "question_text": "Alice, are you on the phone with Bob from Faber Bank right now?",
   *                              "question_detail": "This is optional fine-print giving context to the question and its various answers.",
   *                              "nonce": "<valid_nonce>",
   *                              "signature_required": true,
   *                              "valid_responses" : [
   *                                  {"text": "Yes, it's me"},
   *                                  {"text": "No, that's not me!"}],
   *                              "~timing": {
   *                                  "expires_time": "2018-12-13T17:29:06+0000"
   *                              }
   *                          }
   * @param  answer           An answer to use which is a JSON string representing chosen `valid_response` option from Question message.
   *                          {
   *                              "text": "Yes, it's me"
   *                          }
   *
   * @return                 Sent message as JSON string.
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async sendAnswer({ handle, question, answer }: IConnectionSendAnswerData): Promise<string> {
    return await RNIndy.connectionSendAnswer(handle, question, answer)
  }

  /**
   * Send a message to invite another side to take a particular action.
   * The action is represented as a `goal_code` and should be described in a way that can be automated.
   * <p>
   * The related protocol can be found here:
   *     https://github.com/hyperledger/aries-rfcs/blob/ecf4090b591b1d424813b6468f5fc391bf7f495b/features/0547-invite-action-protocol
   *
   * @param  connectionHandle handle pointing to a Connection object to send answer message.
   * @param  data             JSON containing information to build message
   *      {
   *          goal_code: string - A code the receiver may want to display to the user or use in automatically deciding what to do after receiving the message.
   *          ack_on: Optional<array<string>> - Specify when ACKs message need to be sent back from invitee to inviter:
   *              * not needed - None or empty array
   *              * at the time the invitation is accepted - ["ACCEPT"]
   *              * at the time out outcome for the action is known - ["OUTCOME"]
   *              * both - ["ACCEPT", "OUTCOME"]
   *      }
   *
   * @return                  s
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async sendInviteAction({ handle, data }: IConnectionSendInviteActionData): Promise<string> {
    return await RNIndy.connectionSendInviteAction(handle, data)
  }

  /**
   * Get Connection redirect details.
   *
   * @param  connectionHandle  handle pointing to a Connection object.
   *
   * @return                   Connection redirect details as JSON string.
   *
   * @throws VcxException      If an exception occurred in Libvcx library.
   */
  public static async getRedirectDetails({ handle }: IConnectionGetData): Promise<string> {
    return await RNIndy.getRedirectDetails(handle)
  }

  //   /**
  //    * Get the information about the established Connection.
  //    * <p>
  //    * Note: This method can be used for `aries` communication method only.
  //    * For other communication method it returns ActionNotSupported error.
  //    *
  //    * @param  connectionHandle     handle pointing to a Connection object.
  //    *
  //    * @return                      Connection Information as JSON string.
  //    *                              {
  //    *                                  "current": {
  //    *                                      "did": string, - DID of current connection side
  //    *                                      "recipientKeys": array[string], - Recipient keys
  //    *                                      "routingKeys": array[string], - Routing keys
  //    *                                      "serviceEndpoint": string, - Endpoint
  //    *                                      "protocols": array[string], - The set of protocol supported by current side.
  //    *                                  },
  //    *                                  "remote: Optional({ - details about remote connection side
  //    *                                      "did": string - DID of remote side
  //    *                                      "recipientKeys": array[string] - Recipient keys of remote side
  //    *                                      "routingKeys": array[string] - Routing keys of remote side
  //    *                                      "serviceEndpoint": string - Endpoint of remote side
  //    *                                      "protocols": array[string] - The set of protocol supported by side. Is filled after DiscoveryFeatures process was completed.
  //    *                                  })
  //    *                              }
  //    *
  //    * @throws VcxException         If an exception occurred in Libvcx library.
  //    */
  //   public static async getInfo({
  //     handle,
  //   }: IConnectionGetData): Promise<string> {
  //     return await RNIndy.connectionGetInfo(
  //       handle,
  //     )
  //   }

  /**
   * Get Problem Report message for object in Failed or Rejected state.
   *
   * @param  connectionHandle handle pointing to Connection state object.
   *
   * @return                  Problem Report as JSON string or null
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async getProblemReportMessage({ handle }: IConnectionGetData): Promise<string> {
    return await RNIndy.connectionGetProblemReport(handle)
  }

  /**
   * Delete a Connection object from the agency and release its handle.
   * <p>
   * NOTE: This eliminates the connection and any ability to use it for any communication.
   *
   * @param  connectionHandle handle pointing to a Connection object.
   *
   * @return                  void
   *
   * @throws VcxException    If an exception occurred in Libvcx library.
   */
  public static async delete({ handle }: IConnectionDeleteData): Promise<void> {
    return await RNIndy.deleteConnection(handle)
  }

  /**
   * Get JSON string representation of Connection object.
   *
   * @param  connectionHandle  handle pointing to a Connection object.
   *
   * @return                   Connection object as JSON string.
   *
   * @throws VcxException      If an exception occurred in Libvcx library.
   */
  public static async serialize({ handle }: IConnectionSerializeData): Promise<string> {
    return await RNIndy.getSerializedConnection(handle)
  }

  /**
   * Takes a json string representing a Connection object and recreates an object matching the JSON.
   *
   * @param  connectionData  JSON string representing a Connection object.
   *
   * @return                 handle that should be used to perform actions with the Connection object.
   *
   * @throws VcxException    If an exception occurred in Libvcx library.
   */
  public static async deserialize({ serialized }: IConnectionDeserializeData): Promise<number> {
    return await RNIndy.deserializeConnection(serialized)
  }

  /**
   * Send trust ping message to the specified connection to prove that two agents have a functional pairwise channel.
   * <p>
   * Note that this function works in case `aries` communication method is used.
   * In other cases it returns ActionNotSupported error.
   *
   * @param  connectionHandle handle pointing to a Connection object.
   * @param  comment          (Optional) human-friendly description of the ping.
   *
   * @return                  void
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async sendPing({ handle, comment }: IConnectionSendPing): Promise<void> {
    return await RNIndy.connectionSendPing(handle, comment)
  }
}
