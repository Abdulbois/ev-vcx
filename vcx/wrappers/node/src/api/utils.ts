import { Callback } from 'ffi-napi'

import { VCXInternalError } from '../errors'
import { initRustAPI, rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { IInitVCXOptions } from './common'
// import { resolve } from 'url';

// tslint:disable-next-line:interface-name
export interface PtrBuffer extends Buffer {
  // Buffer.deref typing provided by @types/ref-napi is wrong, so we overwrite the typing/
  // An issue is currently dealing with fixing it https://github.com/DefinitelyTyped/DefinitelyTyped/pull/44004#issuecomment-744497037
  deref: () => PtrBuffer
}

export async function provisionAgent (configAgent: string, options: IInitVCXOptions = {}): Promise<string> {
  /**
   * Provision an agent in the agency, populate configuration and wallet for this agent.
   *
   * Params:
   *  configAgent - Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
   *
   * Example:
   * ```
   * enterpriseConfig = {
   *     'agency_url': 'https://enym-eagency.pdev.evernym.com',
   *     'agency_did': 'YRuVCckY6vfZfX9kcQZe3u',
   *     'agency_verkey': "J8Yct6FwmarXjrE2khZesUXRVVSVczSoa9sFaGe6AD2v",
   *     'wallet_name': 'LIBVCX_SDK_WALLET',
   *     'agent_seed': '00000000000000000000000001234561',
   *     'enterprise_seed': '000000000000000000000000Trustee1',
   *     'wallet_key': '1234'
   *  }
   * vcxConfig = await provisionAgent(JSON.stringify(enterprise_config))
   */
  try {
    initRustAPI(options.libVCXPath)
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_agent_provision_async(0, configAgent, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, config: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(config)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function provisionAgentWithToken (configAgent: string, token: string, options: IInitVCXOptions = {}): Promise<string> {
  /**
   * Provision an agent in the agency, populate configuration and wallet for this agent.
   *
   * Params:
   *  configAgent - Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
   *  token -       Provisioning token provided by sponsor
   *
   * Example:
   * ```
   * enterpriseConfig = {
   *     'agency_url': 'https://enym-eagency.pdev.evernym.com',
   *     'agency_did': 'YRuVCckY6vfZfX9kcQZe3u',
   *     'agency_verkey': "J8Yct6FwmarXjrE2khZesUXRVVSVczSoa9sFaGe6AD2v",
   *     'wallet_name': 'LIBVCX_SDK_WALLET',
   *     'agent_seed': '00000000000000000000000001234561',
   *     'enterprise_seed': '000000000000000000000000Trustee1',
   *     'wallet_key': '1234'
   *  }
   * vcxConfig = await provisionAgent(JSON.stringify(enterprise_config))
   */

    /**   Token Example:
    *    {
    *       "sponseeId": string,
    *       "sponsorId": string, //name of enterprise sponsoring the provisioning
    *       "nonce": string,
    *       "timestamp": string,
    *       "sig": string, // base64encoded(sig(nonce + timestamp + id))
    *       "sponsorVerKey": string,
    *       "attestationAlgorithm": Optional<string>, // device attestation signature algorithm. Can be one of: SafetyNet | DeviceCheck
    *       "attestationData": Optional<string>, // device attestation signature matching to specified algorithm
    *     }
    **/
  try {
    initRustAPI(options.libVCXPath)
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_provision_agent_with_token(0, configAgent, token, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, config: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(config)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function provisionAgentWithTokenAsync (configAgent: string, token: string, options: IInitVCXOptions = {}): Promise<string> {
  /**
   * Provision an agent in the agency, populate configuration and wallet for this agent.
   *
   * Params:
   *  configAgent - Configuration JSON. See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
   *  token -       Provisioning token provided by sponsor
   *
   * Example:
   * ```
   * configAgent = {
   *     'agency_url': 'https://enym-eagency.pdev.evernym.com',
   *     'agency_did': 'YRuVCckY6vfZfX9kcQZe3u',
   *     'agency_verkey': "J8Yct6FwmarXjrE2khZesUXRVVSVczSoa9sFaGe6AD2v",
   *     'wallet_name': 'LIBVCX_SDK_WALLET',
   *     'agent_seed': '00000000000000000000000001234561',
   *     'enterprise_seed': '000000000000000000000000Trustee1',
   *     'wallet_key': '1234'
   *  }
   * token =
   *    {
   *       "sponseeId": string,
   *       "sponsorId": string, //name of enterprise sponsoring the provisioning
   *       "nonce": string,
   *       "timestamp": string,
   *       "sig": string, // base64encoded(sig(nonce + timestamp + id))
   *       "sponsorVerKey": string,
   *       "attestationAlgorithm": Optional<string>, // device attestation signature algorithm. Can be one of: SafetyNet | DeviceCheck
   *       "attestationData": Optional<string>, // device attestation signature matching to specified algorithm
   *     }
   **/
  try {
    initRustAPI(options.libVCXPath)
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_provision_agent_with_token_async(0, configAgent, token, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, config: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(config)
        })
)
} catch (err) {
throw new VCXInternalError(err)
  }
}

export async function getProvisionToken (config: string, options: IInitVCXOptions = {}): Promise<string> {
  /**
   * Get token that can be used for provisioning an agent
   * NOTE: Can be used only for Evernym's applications
   * Config:
   * {
   *     vcx_config: VcxConfig // Same config passed to agent provision
   *                           // See: https://github.com/evernym/mobile-sdk/blob/master/docs/Configuration.md#agent-provisioning-options
   *     sponsee_id: String,
   *     sponsor_id: String,
   * }
   */
  try {
    initRustAPI(options.libVCXPath)
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_get_provision_token(0, config, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, token: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(token)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function updateAgentInfo (options: string): Promise<void> {
  /**
   * Update information on the agent (ie, comm method and type)
   */
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_agent_update_info(0, options, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32'],
        (xhandle: number, err: number) => {
          if (err) {
            reject(err)
            return
          }
          resolve()
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export function getVersion (): string {
  return rustAPI().vcx_version()
}

export async function getLedgerFees (): Promise<string> {
  /**
   * Get ledger fees from the sovrin network
   */
  try {
    const ledgerFees = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_ledger_get_fees(0, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, fees: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(fees)
        })
    )
    return ledgerFees
  } catch (err) {
    throw new VCXInternalError(err)
  }
}
export function shutdownVcx (deleteWallet: boolean): number {
  return rustAPI().vcx_shutdown(deleteWallet)
}

export interface IUpdateInstitutionConfigs {
  name: string,
  logoUrl: string
}
export function updateInstitutionConfigs ({ name, logoUrl }: IUpdateInstitutionConfigs): number {
  const rc = rustAPI().vcx_update_institution_info(name, logoUrl)
  if (rc) {
    throw new VCXInternalError(rc)
  }
  return rc
}

export interface IDownloadMessagesConfigs {
  status: string,
  uids: string,
  pairwiseDids: string
}

export async function downloadMessages
({ status, uids, pairwiseDids }: IDownloadMessagesConfigs): Promise<string> {
  /**
   *  Retrieve messages from the agency
   */
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_messages_download(0, status, uids, pairwiseDids, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, messages: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(messages)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export interface IUpdateMessagesConfigs {
  msgJson: string
}

export async function updateMessages ({ msgJson }: IUpdateMessagesConfigs): Promise<number> {
  /**
   * Update the status of messages from the specified connection
   */
  try {
    return await createFFICallbackPromise<number>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_messages_update_status(0, 'MS-106', msgJson, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32'],
        (xhandle: number, err: number) => {
          if (err) {
            reject(err)
            return
          }
          resolve(err)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function endorseTransaction (transaction: string): Promise<void> {
  /**
   * Endorse transaction to the ledger preserving an original author
   */
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_endorse_transaction(0, transaction, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32'],
        (xhandle: number, err: number) => {
          if (err) {
            reject(err)
            return
          }
          resolve()
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export interface IDownloadMessage {
  uid: string, // id of the message to query.
}

export async function downloadMessage ({ uid }: IDownloadMessage): Promise<string> {
  /**
   *  Retrieves single message from the agency by the given uid.
   */
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_download_message(0, uid, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, message: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(message)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function fetchPublicEntities (): Promise<void> {
  /**
   * Fetch and Cache public entities from the Ledger associated with stored in the wallet credentials.
   * This function performs two steps:
   *     1) Retrieves the list of all credentials stored in the opened wallet.
   *     2) Fetch and cache Schemas / Credential Definitions / Revocation Registry Definitions
   *        correspondent to received credentials from the connected Ledger.
   *
   * This helper function can be used, for instance as a background task, to refresh library cache.
   * This allows us to reduce the time taken for Proof generation by using already cached entities
   * instead of queering the Ledger.
   *
   * NOTE: Library must be already initialized (wallet and pool must be opened).
   */
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_fetch_public_entities(0, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32'],
        (xhandle: number, err: number) => {
          if (err) {
            reject(err)
            return
          }
          resolve()
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function healthCheck (): Promise<void> {
  /**
   * This function allows you to check the health of LibVCX and EAS/CAS instance.
   * It will return error in case of any problems on EAS or will resolve pretty long if VCX is thread-hungry.
   * WARNING: this call may take a lot of time returning answer in case of load, be careful.
   * NOTE: Library must be initialized, ENDPOINT_URL should be set
   */
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_health_check(0, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32'],
        (xhandle: number, err: number) => {
          if (err) {
            reject(err)
            return
          }
          resolve()
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function createPairwiseAgent(): Promise<string> {
  /**
   *  Create pairwise agent which can be later used for connection establishing.
   *
   *  You can pass `agent_info` into `vcx_connection_connect` function as field of `connection_options` JSON parameter.
   *  The passed Pairwise Agent will be used for connection establishing instead of creation a new one.
   */
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_create_pairwise_agent(0, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, agentInfo: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(agentInfo)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export interface IExtractAttachedMessage {
  message: string, // aries message containing attachment decorator
}

export async function extractAttachedMessage({ message }: IExtractAttachedMessage): Promise<string> {
  /**
   *  Extract content of Aries message containing attachment decorator.
   */
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_extract_attached_message(0, message, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, messages: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(messages)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export async function extractThreadId({ message }: IExtractAttachedMessage): Promise<string> {
  /**
   *  Extract content of Aries message containing attachment decorator.
   */
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_extract_thread_id(0, message, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, messages: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(messages)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}

export interface IResolveMessageByUrl {
  url: string, // url to fetch message
}

export async function resolveMessageByUrl({ url }: IResolveMessageByUrl): Promise<string> {
  /**
   *  Resolve message by the given URL.
   */
  try {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_resolve_message_by_url(0, url, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback(
        'void',
        ['uint32','uint32','string'],
        (xhandle: number, err: number, messages: string) => {
          if (err) {
            reject(err)
            return
          }
          resolve(messages)
        })
    )
  } catch (err) {
    throw new VCXInternalError(err)
  }
}
