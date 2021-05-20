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
  /**
   * Get token that can be used for provisioning an agent
   * NOTE: Can be used only for Evernym's applications
   *
   * @param  config           provisioning configuration.
   * {
   *     vcx_config: VcxConfig // Same config passed to agent provision
   *                           // See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
   *     sponsee_id: String,
   *     sponsor_id: String,
   *     com_method: {
   *         type: u32 // 1 means push notifications, 4 means forward to sponsor app
   *         id: String,
   *         value: String,
   *     },
   * }
   *
   * @return                provisioning token as JSON
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   *
   **/
  public static async getProvisionToken({
    agencyConfig,
  }: IGetProvisionTokenData): Promise<string> {
    return await RNIndy.getProvisionToken(
      agencyConfig,
    )
  }

  /**
   * Provision an agent in the agency, populate configuration and wallet for this agent.
   *
   * @param  conf           Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
   *
   * @return                populated config that can be used for library initialization.
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async provision({
    agencyConfig,
  }: IProvisionData): Promise<string> {
    return await RNIndy.createOneTimeInfo(
      agencyConfig,
    )
  }

  /**
   * Provision an agent in the agency, populate configuration and wallet for this agent.
   *
   * @param  config         Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
   * @param  token          provisioning token.
   *      {
   *          This can be a push notification endpoint to contact the sponsee or
   *          an id that the sponsor uses to reference the sponsee in its backend system
   *          "sponseeId": String,
   *          "sponsorId": String, //Persistent Id of the Enterprise sponsoring the provisioning
   *          "nonce": String,
   *          "timestamp": String,
   *          "sig": String, // Base64Encoded(sig(nonce + timestamp + id))
   *          "sponsorVerKey": String,
   *          "attestationAlgorithm": Optional[String], // device attestation signature algorithm. Can be one of: SafetyNet | DeviceCheck
   *          "attestationData": Optional[String], // device attestation signature matching to specified algorithm
   *        }
   *
   * @return                populated config that can be used for library initialization.
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   *
   **/
  public static async provisionWithToken({
    agencyConfig,
    token,
  }: IProvisionWithTokenData): Promise<string> {
    return await RNIndy.createOneTimeInfoWithToken(
      agencyConfig,
      token,
    )
  }

  /**
   * Update information on the agent (ie, comm method and type)
   *
   * @param  config         New agent updated configuration as JSON
   *                        "{"id":"123","value":"value"}"
   *
   * @return                void
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async updateInfo({
    config,
  }: IUpdateAgentInfoData): Promise<void> {
    return await RNIndy.vcxUpdatePushToken(
      config,
    )
  }

  /**
   * Retrieve messages from the agent
   *
   * @param  messageStatus  optional, comma separated - query for messages with the specified status.
   *                             Statuses:
   *                                  MS-101 - Created
   *                                  MS-102 - Sent
   *                                  MS-103 - Received
   *                                  MS-104 - Accepted
   *                                  MS-105 - Rejected
   *                                  MS-106 - Reviewed
   *                        "MS-103,MS-106"
   * @param  uids           optional, comma separated - query for messages with the specified uids
   *                        "s82g63,a2h587"
   * @param  pwdids         optional, comma separated - DID's pointing to specific connection
   *                        "did1,did2"
   *
   * @return                The list of all found messages
   *                        "[{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}]"
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
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

  /**
   * Retrieve messages from the Cloud Agent
   *
   * @param  messageStatus  optional, comma separated - query for messages with the specified status.
   *                             Statuses:
   *                                  MS-101 - Created
   *                                  MS-102 - Sent
   *                                  MS-103 - Received
   *                                  MS-104 - Accepted
   *                                  MS-105 - Rejected
   *                                  MS-106 - Reviewed
   *                        "MS-103,MS-106"
   * @param  uids           optional, comma separated - query for messages with the specified uids
   *                        "s82g63,a2h587"
   *
   * @return                The list of all found messages
   *                        "[{"pairwiseDID":"did","msgs":[{"statusCode":"MS-106","payload":null,"senderDID":"","uid":"6BDkgc3z0E","type":"aries","refMsgId":null,"deliveryDetails":[],"decryptedPayload":"{"@msg":".....","@type":{"fmt":"json","name":"aries","ver":"1.0"}}"}]}]"
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async downloadAgentMessages({
    messageStatus,
    uids,
  }: IDownloadAgentMessagesData): Promise<string> {
    return await RNIndy.vcxGetAgentMessages(
      messageStatus,
      uids,
    )
  }

  /**
   * Update the status of messages from the specified connection
   *
   * @param  messageStatus  message status to set
   *                             Statuses:
   *                                  MS-101 - Created
   *                                  MS-102 - Sent
   *                                  MS-103 - Received
   *                                  MS-104 - Accepted
   *                                  MS-105 - Rejected
   *                                  MS-106 - Reviewed
   *                        "MS-103,MS-106"
   * @param  msgJson        list of messages to update
   *                        [{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]},...]
   *
   * @return               void
   *
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async updateMessages({
    messageStatus,
    pwdids,
  }: IUpdateMessagesData): Promise<number> {
    return await RNIndy.updateMessages(
      messageStatus,
      pwdids,
    )
  }

  /**
   * Create pairwise agent which can be later used for connection establishing.
   *
   * You can pass `agent_info` into `vcx_connection_connect` function as field of `connection_options` JSON parameter.
   * The passed Pairwise Agent will be used for connection establishing instead of creation a new one.
   *
   * @return   Agent info as JSON string:
   *     {
   *         "pw_did": string,
   *         "pw_vk": string,
   *         "agent_did": string,
   *         "agent_vk": string,
   *     }
   * @throws VcxException   If an exception occurred in Libvcx library.
   */
  public static async createPairwiseAgent(): Promise<string> {
    return await RNIndy.createPairwiseAgent()
  }
}
