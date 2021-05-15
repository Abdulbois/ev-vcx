import { NativeModules } from 'react-native'

const { RNIndy } = NativeModules

interface IInitData {
  config: string,
}

interface IInitPoolData {
  config: string,
}

interface IShutdownData {
  deleteWallet?: boolean,
}

interface IInitWithConfigPath {
  configPath: string
}

export class Library {
  public static async init({
    config,
  }: IInitData): Promise<boolean> {
    return await RNIndy.init(
      config,
    )
  }

  public static async initPool({
    config,
  }: IInitPoolData): Promise<boolean> {
    return await RNIndy.vcxInitPool(
      config,
    )
  }

  public static async shutdown({
    deleteWallet,
  }: IShutdownData): Promise<void> {
    return await RNIndy.shutdownVcx(
      deleteWallet,
    )
  }

  public static async initWithConfigPath({ configPath }: IInitWithConfigPath): Promise<number> {
    return await RNIndy.initWithConfigPath(configPath)
  }
}
