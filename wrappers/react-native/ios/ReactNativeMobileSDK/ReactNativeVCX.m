#import "ReactNativeVCX.h"

@implementation ReactNativeVCX

RCT_EXPORT_MODULE();

RCT_EXPORT_METHOD(getModuleList: (RCTResponseSenderBlock)callback)
{
   NSArray *nativeModuleList = @[@"react-native-fbsdk", @"react-native-camera", @"react-native-maps"];
   callback(@[[NSNull null], nativeModuleList]);
}

@end
