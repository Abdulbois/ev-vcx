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

export class Verifier {
  /**
   * Create a new DisclosedProof object that requests a proof for an enterprise
   *
   * @param  requestedAttrs       Describes the list of requested attribute
   *     [{
   *         "name": Optional(string), // attribute name, (case insensitive and ignore spaces)
   *         "names": Optional([string, string]), // attribute names, (case insensitive and ignore spaces)
   *                                              // NOTE: should either be "name" or "names", not both and not none of them.
   *                                              // Use "names" to specify several attributes that have to match a single credential.
   *         "restrictions":  Optional(wql query) - set of restrictions applying to requested credentials. (see below)
   *         "non_revoked": {
   *             "from": Optional(u64) Requested time represented as a total number of seconds from Unix Epoch, Optional
   *             "to": Optional(u64)
   *                 //Requested time represented as a total number of seconds from Unix Epoch, Optional
   *         }
   *     }]
   * @param  requestedPredicates  predicate specifications prover must provide claim for.
   *     <pre>
   *     {@code
   *     [
   *        { // set of requested predicates
   *           "name": attribute name, (case insensitive and ignore spaces)
   *           "p_type": predicate type (Currently ">=" only)
   *           "p_value": int predicate value
   *           "restrictions":  Optional(wql query) -  set of restrictions applying to requested credentials. (see below)
   *           "non_revoked": Optional({
   *               "from": Optional(u64) Requested time represented as a total number of seconds from Unix Epoch, Optional
   *               "to": Optional(u64) Requested time represented as a total number of seconds from Unix Epoch, Optional
   *           })
   *       }
   *    ]
   *    }
   *    </pre>
   *
   * @param  revocationInterval  Optional timestamps to request revocation proof
   * @param  name                label for proof request.
   *
   * @return                      handle that should be used to perform actions with the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
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

  /**
   * Create a new DisclosedProof object based on the given Presentation Proposal message
   *
   * @param  presentationProposal Message sent by the Prover to the verifier to initiate a proof presentation process:
   *         {
   *             "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/propose-presentation",
   *             "@id": "<uuid-propose-presentation>",
   *             "comment": "some comment",
   *             "presentation_proposal": {
   *                 "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation-preview",
   *                 "attributes": [
   *                     {
   *                         "name": "<attribute_name>", - name of the attribute.
   *                         "cred_def_id": "<cred_def_id>", - maps to the credential definition identifier of the credential with the current attribute
   *                         "mime-type": Optional"<type>", - optional type of value. if mime-type is missing (null), then value is a string.
   *                         "value": "<value>", - value of the attribute to reveal in presentation
   *                     },
   *                     // more attributes
   *                   ],
   *                  "predicates": [
   *                     {
   *                         "name": "<attribute_name>", - name of the attribute.
   *                         "cred_def_id": "<cred_def_id>", - maps to the credential definition identifier of the credential with the current attribute
   *                         "predicate": "<predicate>", - predicate operator: "<", "<=", ">=", ">"
   *                         "threshold": <threshold> - threshold value for the predicate.
   *                     },
   *                     // more predicates
   *                 ]
   *             }
   *         }
   *
   * @param  name                 label for proof request.
   *
   * @return                      handle that should be used to perform actions with the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
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

  /**
   * Get the current state of the DisclosedProof object
   * DisclosedProof states:
   *     1 - Initialized
   *     2 - DisclosedProof Request Sent
   *     3 - DisclosedProof Received
   *     4 - DisclosedProof Accepted
   *
   * @param  handle               handle pointing to a DisclosedProof object.
   *
   * @return                      the most current state of the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getState({ handle }: IVerifierGetStateData): Promise<number> {
    return await RNIndy.proofVerifierGetState(
      handle,
    )
  }

  /**
   * Query the agency for the received messages.
   * Checks for any messages changing state in the DisclosedProof object and updates the state attribute.
   *
   * @param  handle               handle pointing to a DisclosedProof object.
   *
   * @return                      the most current state of the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async updateState({ handle }: IVerifierUpdateStateData): Promise<number> {
    return await RNIndy.proofVerifierUpdateState(
      handle,
    )
  }

  /**
   * Update the state of the DisclosedProof object based on the given message.
   *
   * @param  handle               handle pointing to a DisclosedProof object.
   * @param  message              message to process for any DisclosedProof state transitions.
   *
   * @return                      the most current state of the DisclosedProof object.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async updateStateWithMessage({
    handle,
    message,
  }: IVerifierUpdateStateWithMessageData): Promise<number> {
    return await RNIndy.proofVerifierUpdateStateWithMessage(
      handle,
      message,
    )
  }

  /**
   * Sends a DisclosedProof Request message to pairwise connection.
   *
   * @param  handle               handle pointing to a DisclosedProof object.
   * @param  connectionHandle     handle pointing to a Connection object to use for sending message.
   *
   * @return                      void
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async sendProofRequest({ handle, connectionHandle }: IVerifierRequestProofData): Promise<void> {
    return await RNIndy.proofVerifierSendRequest(
      handle,
      connectionHandle,
    )
  }

  /**
   * Get DisclosedProof Request message that can be sent to the pairwise connection.
   *
   * @param  handle               handle pointing to a DisclosedProof object.
   *
   * @return                      Proof Request message as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getProofRequestMessage({
    handle,
  }: IVerifierGetData): Promise<string> {
    return await RNIndy.proofVerifierGetPresentationRequest(
      handle,
    )
  }

  /**
   * Get DisclosedProof message that can be sent to the specified connection.
   *
   * @param  handle               handle pointing to a DisclosedProof object.
   *
   * @return                      Proof message as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getProofMessage({
    handle,
  }: IVerifierGetData): Promise<string> {
    return await RNIndy.proofVerifierGetProofMessage(
      handle,
    )
  }

  /**
   * Get Problem Report message for object in Failed or Rejected state.
   *
   * @param  handle           handle pointing to DisclosedProof state object.
   *
   * @return                  Problem Report as JSON string or null
   *
   * @throws VcxException     If an exception occurred in Libvcx library.
   */
  public static async getProblemReportMessage({
    handle,
  }: IVerifierGetData): Promise<string> {
    return await RNIndy.proofVerifierGetProblemReport(
      handle,
    )
  }

  /**
   * Get JSON string representation of DisclosedProof object.
   *
   * @param  proofHandle          handle pointing to a DisclosedProof object.
   *
   * @return                      Proof object as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async serialize({ handle }: IVerifierSerializeData): Promise<string> {
    return await RNIndy.proofVerifierSerialize(
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
  public static async deserialize({ serialized }: IVerifierDeserializeData): Promise<number> {
    return await RNIndy.proofVerifierDeserialize(
      serialized,
    )
  }

  /**
   * Sends a new DisclosedProof Request message to pairwise connection.
   * Used after receiving a proposal, to negotiate.
   *
   * @param  handle               handle pointing to a DisclosedProof object.
   * @param  connectionHandle     handle pointing to a Connection object to use for sending message.
   * @param  requestedAttrs       Describes the list of requested attribute
   *     [{
   *         "name": Optional(string), // attribute name, (case insensitive and ignore spaces)
   *         "names": Optional([string, string]), // attribute names, (case insensitive and ignore spaces)
   *                                              // NOTE: should either be "name" or "names", not both and not none of them.
   *                                              // Use "names" to specify several attributes that have to match a single credential.
   *         "restrictions":  Optional(wql query) - set of restrictions applying to requested credentials. (see below)
   *         "non_revoked": {
   *             "from": Optional(u64) Requested time represented as a total number of seconds from Unix Epoch, Optional
   *             "to": Optional(u64)
   *                 //Requested time represented as a total number of seconds from Unix Epoch, Optional
   *         }
   *     }]
   * @param  requestedPredicates  predicate specifications prover must provide claim for.
   *     <pre>
   *     {@code
   *     [
     *        { // set of requested predicates
     *           "name": attribute name, (case insensitive and ignore spaces)
     *           "p_type": predicate type (Currently ">=" only)
     *           "p_value": int predicate value
     *           "restrictions":  Optional(wql query) -  set of restrictions applying to requested credentials. (see below)
     *           "non_revoked": Optional({
   *               "from": Optional(u64) Requested time represented as a total number of seconds from Unix Epoch, Optional
     *               "to": Optional(u64) Requested time represented as a total number of seconds from Unix Epoch, Optional
     *           })
   *       }
   *    ]
   *    }
   *    </pre>
   *
   * @param  revocationInterval  Optional timestamps to request revocation proof
   * @param  name                label for proof request.
   *
   * @return                     void
   *
   * @throws VcxException        If an exception occurred in Libvcx library.
   */
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

  /**
   * Get Proof proposal received.
   *
   * @param  handle               handle pointing to a DisclosedProof object.
   *
   * @return                      Proof proposal as JSON string.
   *
   * @throws VcxException         If an exception occurred in Libvcx library.
   */
  public static async getProofProposalMessage({
    handle
  }: IVerifierGetProofProposal): Promise<string> {
    return await RNIndy.proofVerifierGetProofProposal(handle)
  }
}
