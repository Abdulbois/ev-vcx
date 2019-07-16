//
//  IndySdk.m
//  vcx
//
//  Created by Norman Jarvis on 2/18/19.
//  Copyright © 2019 GuestUser. All rights reserved.
//


#include "IndySdk.h"
#include "IndyCallbacks.h"
//#include "vcx.h"
#include "indy_types.h"
#include "indy_crypto.h"
#include "indy_ledger.h"

@implementation IndySdk



+ (void)addTxnAuthorAgreement:(NSString *)text
                  withVersion:(NSString *)version
                fromRequester:(NSString *)requesterDID
                completion:(void (^)(NSError *error, NSString *jsonResult))completion
{
    indy_handle_t handle = [[IndyCallbacks sharedInstance] createCommandHandleFor:completion];
    
    indy_error_t ret = indy_build_txn_author_agreement_request(handle,
                                                               [requesterDID UTF8String],
                                                               [text UTF8String],
                                                               [version UTF8String],
                                                               IndyWrapperCommonStringCallback);
    
    [[IndyCallbacks sharedInstance] completeStr:completion forHandle:handle ifError:ret];
}


+ (void)getTxnAuthorAgreement:(NSString *)taaFilter
                fromRequester:(NSString *)requesterDID
                completion:(void (^)(NSError *error, NSString *jsonResult))completion
{
    indy_handle_t handle = [[IndyCallbacks sharedInstance] createCommandHandleFor:completion];

    indy_error_t ret = indy_build_get_txn_author_agreement_request(handle,
                                                                   [requesterDID UTF8String],
                                                                   [taaFilter UTF8String],
                                                                   IndyWrapperCommonStringCallback);
    
    [[IndyCallbacks sharedInstance] completeStr:completion forHandle:handle ifError:ret];
}


+ (void)addAcceptanceMechanisms:(NSString *)aml
                    withVersion:(NSString *)version
                    withContext:(NSString *)amlContext
                  fromRequester:(NSString *)requesterDID
                     completion:(void (^)(NSError *error, NSString *jsonResult))completion
{
    indy_handle_t handle = [[IndyCallbacks sharedInstance] createCommandHandleFor:completion];
    
    indy_error_t ret = indy_build_acceptance_mechanisms_request(handle,
                                                                [requesterDID UTF8String],
                                                                [aml UTF8String],
                                                                [version UTF8String],
                                                                [amlContext UTF8String],
                                                                IndyWrapperCommonStringCallback);
    
    [[IndyCallbacks sharedInstance] completeStr:completion forHandle:handle ifError:ret];
}


+ (void)getAcceptanceMechanisms:(NSNumber *)timestamp
                    withVersion:(NSString *)version
                  fromRequester:(NSString *)requesterDID
                     completion:(void (^)(NSError *error, NSString *jsonResult))completion
{
    indy_handle_t handle = [[IndyCallbacks sharedInstance] createCommandHandleFor:completion];
    
    indy_error_t ret = indy_build_get_acceptance_mechanisms_request(handle,
                                                               [requesterDID UTF8String],
                                                               [timestamp longLongValue],
                                                               [version UTF8String],
                                                               IndyWrapperCommonStringCallback);
    
    [[IndyCallbacks sharedInstance] completeStr:completion forHandle:handle ifError:ret];

}


/// Append transaction author agreement acceptance data to a request.
/// This function should be called before signing and sending a request
/// if there is any transaction author agreement set on the Ledger.
///
/// This function may calculate hash by itself or consume it as a parameter.
/// If all text, version and taa_digest parameters are specified, a check integrity of them will be done.
///
/// #Params
/// request_json: original request data json.
/// text and version - (optional) raw data about TAA from ledger.
///     These parameters should be passed together.
///     These parameters are required if taa_digest parameter is omitted.
/// taa_digest - (optional) hash on text and version. This parameter is required if text and version parameters are omitted.
/// mechanism - mechanism how user has accepted the TAA
/// time - UTC timestamp when user has accepted the TAA
/// completion: Callback that takes command result as parameter.
///
/// #Returns
/// Updated request result as json.
///
/// #Errors
/// Common*
+ (void)appendTxnAuthorAgreement:(NSString *)requestJson
                   withAgreement:(NSString *)text
                     withVersion:(NSString *)version
                      withDigest:(NSString *)taaDigest
                   withMechanism:(NSString *)mechanism
                   withTimestamp:(NSNumber *)time
                   completion:(void (^)(NSError *error, NSString *jsonResult))completion
{
    indy_handle_t handle = [[IndyCallbacks sharedInstance] createCommandHandleFor:completion];
    
    indy_error_t ret = indy_append_txn_author_agreement_acceptance_to_request(handle,
                                                                              [requestJson UTF8String],
                                                                              [text UTF8String],
                                                                              [version UTF8String],
                                                                              [taaDigest UTF8String],
                                                                              [mechanism UTF8String],
                                                                              [time longLongValue],
                                                                              IndyWrapperCommonStringCallback);
    
    [[IndyCallbacks sharedInstance] completeStr:completion forHandle:handle ifError:ret];
    
}



+ (void)anonCrypt:(NSData *)message
         theirKey:(NSString *)theirKey
       completion:(void (^)(NSError *error, NSData *encryptedMsg))completion
{
    indy_handle_t handle = [[IndyCallbacks sharedInstance] createCommandHandleFor:completion];
    
    uint32_t messageLen = (uint32_t) [message length];
    uint8_t *messageRaw = (uint8_t *) [message bytes];
    
    indy_error_t ret = indy_crypto_anon_crypt(handle,
                                              [theirKey UTF8String],
                                              messageRaw,
                                              messageLen,
                                              IndyWrapperCommonDataCallback);
    
    [[IndyCallbacks sharedInstance] completeData:completion forHandle:handle ifError:ret];
}

+ (void)anonDecrypt:(NSData *)encryptedMessage
              myKey:(NSString *)myKey
       walletHandle:(IndyHandle)walletHandle
         completion:(void (^)(NSError *error, NSData *decryptedMessage))completion
{
    
    indy_handle_t handle = [[IndyCallbacks sharedInstance] createCommandHandleFor:completion];
    
    uint32_t messageLen = (uint32_t) [encryptedMessage length];
    uint8_t *messageRaw = (uint8_t *) [encryptedMessage bytes];
    
    indy_error_t ret = indy_crypto_anon_decrypt(handle,
                                                walletHandle,
                                                [myKey UTF8String],
                                                messageRaw,
                                                messageLen,
                                                IndyWrapperCommonDataCallback);
    
    [[IndyCallbacks sharedInstance] completeData:completion forHandle:handle ifError:ret];
}


@end

