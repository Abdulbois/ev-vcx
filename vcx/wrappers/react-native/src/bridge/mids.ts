import { NativeModules } from 'react-native'

const { MIDSDocumentVerification } = NativeModules

interface IMidsSdkInit {
  sdkToken: string
  apiDataCenter: string
}

interface IMidsGetDocumentTypes {
  country: string
}

interface IMidsScanStart {
  document: string
}

export class DocumentVerification {
  public static async midsSdkInit({ sdkToken, apiDataCenter }: IMidsSdkInit): Promise<void> {
    return new Promise((resolve, reject) => {
      MIDSDocumentVerification.initMIDSSDK(
        sdkToken,
        apiDataCenter,
        resolve,
        reject
      )
    })
  }

  public static async midsGetCountryList(): Promise<string> {
    return new Promise((resolve, reject) => {
      MIDSDocumentVerification.getCountryList(resolve, reject)
    })
  }

  public static async midsGetDocumentTypes({ country }: IMidsGetDocumentTypes): Promise<string> {
    return new Promise((resolve, reject) => {
      MIDSDocumentVerification.getDocumentTypes(
        country,
        resolve,
        reject
      )
    })
  }

  public static async midsScanStart({ document }: IMidsScanStart): Promise<string> {
    return new Promise((resolve, reject) => {
      MIDSDocumentVerification.startMIDSSDKScan(
        document,
        '1.0.0',
        resolve,
        reject
      )
    })
  }

  public static async midsSdkTerminate(): Promise<void> {
    return new Promise((resolve, reject) => {
      MIDSDocumentVerification.terminateSDK(resolve, reject)
    })
  }
}
