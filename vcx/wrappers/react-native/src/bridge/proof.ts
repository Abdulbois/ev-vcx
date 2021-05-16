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
  /**
   * Create a DisclosedProof object for fulfilling a corresponding proof request.
   *
   * @param  sourceId         Institution's personal identification for the proof, should be unique.
   * @param  proofRequest     received Proof Request message. The format of Proof Request depends on communication method:
   *                          <pre>
   *                          {@code
   *                              proprietary:
     *                                  "{"@topic":{"mid":9,"tid":1},"@type":{"name":"PROOF_REQUEST","version":"1.0"},"msg_ref_id":"ymy5nth","proof_request_data":{"name":"AccountCertificate","nonce":"838186471541979035208225","requested_attributes":{"business_2":{"name":"business"},"email_1":{"name":"email"},"name_0":{"name":"name"}},"requested_predicates":{},"version":"0.1"}}"
   *                              aries:
   *                                  "{"@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/request-presentation","@id": "<uuid-request>","comment": "some comment","request_presentations~attach": [{"@id": "libindy-request-presentation-0","mime-type": "application/json","data":  {"base64": "<bytes for base64>"}}]}"
   *                          }
   *                          </pre>
   *
   * @return                  handle that should be used to perform actions with the DisclosedProof object.
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async createWithRequest({
    proofRequest,
  }: IProofCreateWithRequestData): Promise<number> {
    return await RNIndy.proofCreateWithRequest(
      uuidv4(),
      proofRequest,
    )
  }

  /**
   * Queries agency for Proof Request messages from the given connection.
   *
   * @param  connectionHandle     handle pointing to Connection object to query for Proof Request messages.
   *
   * @return                      List of received Proof Request messages as JSON string.
   *                              "[{"@topic":{"mid":9,"tid":1},"@type":{"name":"PROOF_REQUEST","version":"1.0"},"msg_ref_id":"ymy5nth","proof_request_data":{"name":"AccountCertificate","nonce":"838186471541979035208225","requested_attributes":{"business_2":{"name":"business"},"email_1":{"name":"email"},"name_0":{"name":"name"}},"requested_predicates":{},"version":"0.1"}}]"
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getRequests({ connectionHandle }: IProofGetRequestsData): Promise<string> {
    return await RNIndy.proofGetRequests(
      connectionHandle,
    )
  }

  /**
   * Get credentials from wallet matching to the proof request associated with proof object
   *
   * @param  proofHandle          handle pointing to a DisclosedProof object.
   *
   * @return                      the list of credentials that can be used for proof generation
   *                              "{'attrs': {'attribute_0': [{'cred_info': {'schema_id': 'id', 'cred_def_id': 'id', 'attrs': {'attr_name': 'attr_value', ...}, 'referent': '914c7e11'}}]}}"
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getCredentialsForProofRequest({ handle }: IProofGetCredentialsData): Promise<string> {
    return await RNIndy.proofRetrieveCredentials(
      handle,
    )
  }

  /**
   * Get the current state of the DisclosedProof object
   * Credential states:
   *         3 - Proof Request Received
   *         4 - Proof Sent
   *
   * @param  proofHandle          handle pointing to a DisclosedProof object.
   *
   * @return                      the most current state of the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getState({ handle }: IProofGetStateData): Promise<number> {
    return await RNIndy.proofGetState(
      handle,
    )
  }

  /**
   * Query the agency for the received messages.
   * Checks for any messages changing state in the DisclosedProof object and updates the state attribute.
   *
   * @param  proofHandle          handle pointing to a DisclosedProof object.
   *
   * @return                      the most current state of the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async updateState({ handle }: IProofUpdateStateData): Promise<number> {
    return await RNIndy.proofUpdateState(
      handle,
    )
  }

  /**
   * Query the agency for the received messages.
   * Checks for any messages changing state in the DisclosedProof object and updates the state attribute.
   *
   * @param  proofHandle          handle pointing to a DisclosedProof object.
   * @param  message              message.
   *
   * @return                      the most current state of the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async updateStateWithMessage({
    handle,
    message,
  }: IProofUpdateStateWithMessageData): Promise<number> {
    return await RNIndy.proofUpdateStateWithMessage(
      handle,
      message,
    )
  }

  /**
   * Accept Proof Request associated with DisclosedProof object and generates a Proof from the selected credentials and self attested attributes
   *
   * @param  proofHandle              handle pointing to a DisclosedProof object.
   * @param  selectedCredentials      a json string with a credential for each proof request attribute.
   * @param  selfAttestedAttributes   a json string with attributes self attested by user
   *
   * @return                          void
   *
   * @throws VcxException             If an exception occurred in Libvcx library.
   */
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

  /**
   * Send a Proof to the connection, called after having received a proof request
   *
   * @param  proofHandle              handle pointing to a DisclosedProof object.
   * @param  connectionHandle         handle pointing to a Connection object to use for sending message (pass 0 in case of ephemeral proof)..
   *
   * @return                          void
   *
   * @throws VcxException             If an exception occurred in Libvcx library.
   */
  public static async sendProof({ handle, connectionHandle }: IProofSendProofData): Promise<void> {
    return await RNIndy.proofSend(
      handle,
      connectionHandle,
    )
  }

  /**
   * Send a Proof Rejection message to the connection, called after having received a Proof Request
   *
   * @param  proofHandle              handle pointing to a DisclosedProof object.
   * @param  connectionHandle         handle pointing to a Connection object to use for sending message.
   *
   * @return                          void
   *
   * @throws VcxException             If an exception occurred in Libvcx library.
   */
  public static async reject({ handle, connectionHandle }: IProofRejectData): Promise<void> {
    return await RNIndy.proofReject(
      handle,
      connectionHandle,
    )
  }

  /**
   * Declines Presentation Request.
   * There are two ways of following interaction:
   *     - Prover wants to propose using a different presentation - pass `proposal` parameter.
   *     - Prover doesn't want to continue interaction - pass `reason` parameter.
   * <p>
   * Note that only one of these parameters can be passed.
   * <p>
   * Note that proposing of different presentation is supported for `aries` protocol only.
   *
   * @param  proofHandle              handle pointing to a DisclosedProof object.
   * @param  connectionHandle         handle pointing to a Connection object to use for sending message.
   * @param  reason                   (Optional) human-readable string that explain the reason of decline.
   * @param  proposal                 (Optional) the proposed format of presentation request.
   *
   * @return                          void
   *
   * @throws VcxException             If an exception occurred in Libvcx library.
   */
  public static async decline({ handle, connectionHandle, reason, proposal }: IProofDeclineData): Promise<void> {
    return await RNIndy.proofDeclineRequest(
      handle,
      connectionHandle,
      reason,
      proposal,
    )
  }

  /**
   * Get Problem Report message for object in Failed or Rejected state.
   *
   * @param  proofHandle      handle pointing to Disclosed Proof state object.
   *
   * @return                  Problem Report as JSON string or null
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async getProblemReportMessage({
    handle,
  }: IProofGetData): Promise<string> {
    return await RNIndy.proofGetProblemReport(
      handle,
    )
  }

  /**
   * Get JSON string representation of DisclosedProof object.
   *
   * @param  proofHandle          handle pointing to a DisclosedProof object.
   *
   * @return                      DisclosedProof object as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async serialize({ handle }: IProofSerializeData): Promise<string> {
    return await RNIndy.proofSerialize(
      handle,
    )
  }

  /**
   * Takes a json string representing a DisclosedProof object and recreates an object matching the JSON.
   *
   * @param  serializedProof      JSON string representing a DisclosedProof object.
   *
   * @return                      handle that should be used to perform actions with the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async deserialize({ serialized }: IProofDeserializeData): Promise<number> {
    return await RNIndy.proofDeserialize(
      serialized,
    )
  }
}
