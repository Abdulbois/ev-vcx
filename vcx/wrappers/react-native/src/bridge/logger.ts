import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface ISetLoggerData {
  logLevel: string
  uniqueIdentifier: string
  maxAllowedFileBytes: number
}

interface IGetLogLevelData {
  levelName: string
}

interface IEncryptVcxLogData {
  logFilePath: string
  key: string
}

interface IWriteToVcxLogData {
  loggerName: string
  logLevel: string
  message: string
  logFilePath: string
}

export class Logger {
  public static async setLogger({ logLevel, uniqueIdentifier, maxAllowedFileBytes }: ISetLoggerData): Promise<string> {
    return await RNIndy.setVcxLogger(logLevel, uniqueIdentifier, maxAllowedFileBytes)
  }

  public static async getLogLevel({ levelName }: IGetLogLevelData): Promise<number> {
    return await RNIndy.getLogLevel(levelName)
  }

  public static async encryptLog({ logFilePath, key }: IEncryptVcxLogData): Promise<string> {
    return await RNIndy.encryptVcxLog(logFilePath, key)
  }

  public static async writeToLog({ loggerName, logLevel, message, logFilePath }: IWriteToVcxLogData): Promise<void> {
    return await RNIndy.writeToVcxLog(loggerName, logLevel, message, logFilePath)
  }
}
