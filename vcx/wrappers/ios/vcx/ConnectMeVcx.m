//
//  init.m
//  vcx
//
//  Created by GuestUser on 4/30/18.
//  Copyright © 2018 GuestUser. All rights reserved.
//

#import <Foundation/Foundation.h>
#import "ConnectMeVcx.h"
#import "utils/NSError+VcxError.h"
#import "utils/VcxCallbacks.h"
#import "vcx.h"
#include "vcx.h"

void VcxWrapperCommonCallback(vcx_command_handle_t xcommand_handle,
                              vcx_error_t err) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *) = (void (^)(NSError *)) block;

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

void VcxWrapperCommonHandleCallback(vcx_command_handle_t xcommand_handle,
                                    vcx_error_t err,
                                    vcx_command_handle_t pool_handle) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, VcxHandle) = (void (^)(NSError *, VcxHandle)) block;

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, (VcxHandle) pool_handle);
        });
    }
}

void VcxWrapperCommonNumberCallback(vcx_command_handle_t xcommand_handle,
                                    vcx_error_t err,
                                    vcx_command_handle_t handle) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, vcx_command_handle_t) = (void (^)(NSError *, vcx_command_handle_t)) block;

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, handle);
        });
    }
}

void VcxWrapperCommonStringCallback(vcx_command_handle_t xcommand_handle,
                                    vcx_error_t err,
                                    const char *const arg1) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *) = (void (^)(NSError *, NSString *arg1)) block;
    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, sarg1);
        });
    }
}

void VcxWrapperCommonBoolCallback(vcx_command_handle_t xcommand_handle,
                                  vcx_error_t err,
                                  unsigned int arg1) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, BOOL) = (void (^)(NSError *, BOOL arg1)) block;

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, (BOOL) arg1);
        });
    }
}

void VcxWrapperCommonStringStringCallback(vcx_command_handle_t xcommand_handle,
                                          vcx_error_t err,
                                          const char *const arg1,
                                          const char *const arg2) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *arg1, NSString *arg2) = (void (^)(NSError *, NSString *arg1, NSString *arg2)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, sarg1, sarg2);
        });
    }
}

void VcxWrapperCommonStringOptStringCallback(vcx_command_handle_t xcommand_handle,
                                             vcx_error_t err,
                                             const char *const arg1,
                                             const char *const arg2) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *arg1, NSString *arg2) = (void (^)(NSError *, NSString *arg1, NSString *arg2)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, sarg1, sarg2);
        });
    }
}

void VcxWrapperCommonStringOptStringOptStringCallback(vcx_command_handle_t xcommand_handle,
                                                      vcx_error_t err,
                                                      const char *const arg1,
                                                      const char *const arg2,
                                                      const char *const arg3) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *arg1, NSString *arg2, NSString *arg3) = (void (^)(NSError *, NSString *arg1, NSString *arg2, NSString *arg3)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }
    NSString *sarg3 = nil;
    if (arg3) {
        sarg3 = [NSString stringWithUTF8String:arg3];
    }

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, sarg1, sarg2, sarg3);
        });
    }
}

void VcxWrapperCommonStringStringStringCallback(vcx_command_handle_t xcommand_handle,
                                                vcx_error_t err,
                                                const char *const arg1,
                                                const char *const arg2,
                                                const char *const arg3) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *arg1, NSString *arg2, NSString *arg3) = (void (^)(NSError *, NSString *arg1, NSString *arg2, NSString *arg3)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }
    NSString *sarg3 = nil;
    if (arg3) {
        sarg3 = [NSString stringWithUTF8String:arg3];
    }

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, sarg1, sarg2, sarg3);
        });
    }
}

/// Arguments arg1 and arg2 will be converted to nsdata
void VcxWrapperCommonDataCallback(vcx_command_handle_t xcommand_handle,
                                  vcx_error_t err,
                                  const uint8_t *const arg1,
                                  uint32_t arg2) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSData *arg) = (void (^)(NSError *, NSData *arg)) block;

    NSData *sarg = [NSData dataWithBytes:arg1 length:arg2];

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, sarg);
        });
    }
}

void VcxWrapperCommonStringDataCallback(vcx_command_handle_t xcommand_handle,
                                        vcx_error_t err,
                                        const char *const arg1,
                                        const uint8_t *const arg2,
                                        uint32_t arg3) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *, NSData *) = (void (^)(NSError *, NSString *, NSData *)) block;

    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSData *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSData dataWithBytes:arg2 length:arg3];
    }

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, sarg1, sarg2);
        });
    }
}

void VcxWrapperCommonStringStringLongCallback(vcx_command_handle_t xcommand_handle,
                                              vcx_error_t err,
                                              const char *arg1,
                                              const char *arg2,
                                              unsigned long long arg3) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, NSString *, NSString *, NSNumber *) = (void (^)(NSError *, NSString *arg1, NSString *arg2, NSNumber *arg3)) block;
    NSString *sarg1 = nil;
    if (arg1) {
        sarg1 = [NSString stringWithUTF8String:arg1];
    }
    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }
    NSNumber *sarg3 = [NSNumber numberWithInt:arg3];


    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, (NSString *) sarg1, (NSString *) sarg2, (NSNumber *) sarg3);
        });
    }
}

void VcxWrapperCommonNumberStringCallback(vcx_command_handle_t xcommand_handle,
                                          vcx_error_t err,
                                          vcx_command_handle_t handle,
                                          const char *const arg2
                                          ) {
    id block = [[VcxCallbacks sharedInstance] commandCompletionFor:xcommand_handle];
    [[VcxCallbacks sharedInstance] deleteCommandHandleFor:xcommand_handle];

    void (^completion)(NSError *, vcx_command_handle_t arg1, NSString *arg2) = (void (^)(NSError *, vcx_command_handle_t arg1, NSString *arg2)) block;

    NSString *sarg2 = nil;
    if (arg2) {
        sarg2 = [NSString stringWithUTF8String:arg2];
    }

    if (completion) {
        NSError *error = [NSError errorFromVcxError:err];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, handle, (NSString *) sarg2);
        });
    }
}


@implementation ConnectMeVcx

//- (int)initSovToken {
//    return sovtoken_init();
//}

//- (int)initNullPay {
//   return nullpay_init();
//}

- (void)initWithConfig:(NSString *)config
            completion:(void (^)(NSError *error))completion
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;
    vcx_error_t ret = vcx_init_with_config(handle, config_char, VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: initWithConfig: calling completion");
            completion(error);
        });
    }

}

- (void)initWithConfigPath:(NSString *)configPath
                completion:(void (^)(NSError *error))completion
{
    const char * config_path_char = [configPath cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;
    vcx_error_t ret = vcx_init(handle, config_path_char, VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: initWithConfig: calling completion");
            completion(error);
        });
    }

}


- (void)vcxSetLogMaxLevel:(NSInteger *)maxLvl
                completion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;
    vcx_error_t ret = vcx_set_log_max_lvl(handle, maxLvl, VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: initWithConfig: calling completion");
            completion(error);
        });
    }

}

- (void)initPool:(NSString *)poolConfig
            completion:(void (^)(NSError *error))completion
{
    const char *poolConfig_char = [poolConfig cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;
    vcx_error_t ret = vcx_init_pool(handle, poolConfig_char, VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: initPool: calling completion");
            completion(error);
        });
    }

}

- (void)agentProvisionAsync:(NSString *)config
               completion:(void (^)(NSError *error, NSString *config))completion
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;
    vcx_error_t ret = vcx_agent_provision_async(handle, config_char, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: agentProvision: calling completion");
            completion(error, false);
        });
    }

}

- (const char *)agentProvision:(NSString *)config
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];

    return vcx_provision_agent(config_char);
}

- (const char *)agentProvisionWithToken:(NSString *)config
                          token:(NSString *)token
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    const char *token_char = [token cStringUsingEncoding:NSUTF8StringEncoding];

    return vcx_provision_agent_with_token(config_char, token_char);
}

- (void)agentProvisionWithTokenAsync:(NSString *)config
                               token:(NSString *)token
                          completion:(void (^)(NSError *error, NSString *result))completion
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    const char *token_char = [token cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;
    vcx_error_t ret = vcx_provision_agent_with_token_async(handle, config_char, token_char, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }

}

- (const char *)vcxGetRequestPrice:(NSString *)config
                 requesterInfoJson:(NSString *)requesterInfoJson
                        completion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;

    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requester_info_json_char = [requesterInfoJson cStringUsingEncoding:NSUTF8StringEncoding];

    return vcx_get_request_price(handle, config_char, requester_info_json_char);
}

- (const char *)vcxEndorseTransaction:(NSString *)requesterInfoJson
                        completion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;
    const char *requester_info_json_char = [requesterInfoJson cStringUsingEncoding:NSUTF8StringEncoding];

    return vcx_endorse_transaction(handle, requester_info_json_char, VcxWrapperCommonStringCallback);
}

- (void)getProvisionToken:(NSString *)config
            completion:(void (^)(NSError *error, NSString *token))completion
{
    const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion] ;
    vcx_error_t ret = vcx_get_provision_token(handle, config_char, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"ERROR: getProvisionToken: calling completion");
            completion(error, nil);
        });
    }

}

- (void)connectionCreateInvite:(NSString *)sourceId
             completion:(void (^)(NSError *error, NSInteger connectionHandle)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_connection_create(handle, sourceId_char, VcxWrapperCommonHandleCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void)connectionCreateWithInvite:(NSString *)invitationId
                inviteDetails:(NSString *)inviteDetails
             completion:(void (^)(NSError *error, NSInteger connectionHandle)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *invitationId_char = [invitationId cStringUsingEncoding:NSUTF8StringEncoding];
   const char *inviteDetails_char = [inviteDetails cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_connection_create_with_invite(handle, invitationId_char, inviteDetails_char, VcxWrapperCommonHandleCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void)connectionCreateOutofband:(NSString *)sourceId
                         goalCode:(NSString *)goalCode
                             goal:(NSString *)goal
                        handshake:(BOOL *)handshake
                    requestAttach:(NSString *)requestAttach
                       completion:(void (^)(NSError *error, NSInteger connectionHandle))completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
   const char *goalCode_char = [goalCode cStringUsingEncoding:NSUTF8StringEncoding];
   const char *goal_char = [goal cStringUsingEncoding:NSUTF8StringEncoding];
   const char *requestAttach_char = [requestAttach cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_connection_create_outofband(handle, sourceId_char, goalCode_char, goal_char, handshake, requestAttach_char, VcxWrapperCommonHandleCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void)acceptConnectionWithInvite:(NSString *)invitationId
                inviteDetails:(NSString *)inviteDetails
                connectionType:(NSString *)connectionType
             completion:(void (^)(NSError *error, NSInteger connectionHandle, NSString *serializedConnection)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *invitationId_char = [invitationId cStringUsingEncoding:NSUTF8StringEncoding];
   const char *inviteDetails_char = [inviteDetails cStringUsingEncoding:NSUTF8StringEncoding];
   const char *connectionType_char = [connectionType cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_connection_accept_connection_invite(handle, invitationId_char, inviteDetails_char,  connectionType_char, VcxWrapperCommonNumberStringCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0, nil);
       });
   }
}

- (void)connectionCreateWithOutofbandInvite:(NSString *)invitationId
                                     invite:(NSString *)invite
                                 completion:(void (^)(NSError *error, NSInteger connectionHandle))completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *invitationId_char = [invitationId cStringUsingEncoding:NSUTF8StringEncoding];
   const char *invite_char = [invite cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_connection_create_with_outofband_invitation(handle, invitationId_char, invite_char, VcxWrapperCommonHandleCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void)connectionConnect:(VcxHandle)connectionHandle
           connectionType:(NSString *)connectionType
               completion:(void (^)(NSError *error, NSString *inviteDetails))completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *connectionType_char = [connectionType cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_connection_connect(handle, connectionHandle, connectionType_char, VcxWrapperCommonStringCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (void)connectionGetState:(NSInteger)connectionHandle
                completion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_connection_get_state(handle, connectionHandle, VcxWrapperCommonNumberCallback);

    if( ret != 0 ) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (void)connectionUpdateState:(NSInteger) connectionHandle
                   completion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_connection_update_state(handle, connectionHandle, VcxWrapperCommonNumberCallback);
    if( ret != 0 ) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (void)connectionSerialize:(NSInteger)connectionHandle
                  completion:(void (^)(NSError *error, NSString *serializedConnection))completion{
    vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_connection_serialize(handle, connectionHandle, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (void) getConnectionInviteDetails:(NSInteger) connectionHandle
                        abbreviated:(BOOL *) abbreviated
         withCompletion:(void (^)(NSError *error, NSString *inviteDetails))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_connection_invite_details(handle, connectionHandle, abbreviated, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)connectionDeserialize:(NSString *)serializedConnection
                    completion:(void (^)(NSError *error, NSInteger connectionHandle))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_connection=[serializedConnection cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_connection_deserialize(handle, serialized_connection, VcxWrapperCommonHandleCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (int)connectionRelease:(NSInteger) connectionHandle {
    return vcx_connection_release(connectionHandle);
}

- (void)deleteConnection:(VcxHandle)connectionHandle
          withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_delete_connection(handle, connectionHandle, VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSLog(@"deleteConnection: calling completion");
            completion(error);
        });
    }
}

- (void)connectionSendMessage:(VcxHandle)connectionHandle
                  withMessage:(NSString *)message
       withSendMessageOptions:(NSString *)sendMessageOptions
               withCompletion:(void (^)(NSError *error, NSString *msg_id))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *message_ctype = [message cStringUsingEncoding:NSUTF8StringEncoding];
    const char *sendMessageOptions_ctype = [sendMessageOptions cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_message(handle,
                                                  connectionHandle,
                                                  message_ctype,
                                                  sendMessageOptions_ctype,
                                                  VcxWrapperCommonStringCallback);
    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)connectionSendPing:(VcxHandle)connectionHandle
                   comment:(NSString *)comment
            withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *comment_ctype = [comment cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_ping(handle,
                                                  connectionHandle,
                                                  comment_ctype,
                                                  VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)connectionSendReuse:(VcxHandle)connectionHandle
                     invite:(NSString *)invite
             withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *invite_ctype = [invite cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_reuse(handle,
                                                connectionHandle,
                                                invite_ctype,
                                                VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)connectionSendAnswer:(VcxHandle)connectionHandle
                    question:(NSString *)question
                      answer:(NSString *)answer
             withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *question_ctype = [question cStringUsingEncoding:NSUTF8StringEncoding];
    const char *answer_ctype = [answer cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_answer(handle,
                                                connectionHandle,
                                                question_ctype,
                                                answer_ctype,
                                                VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)connectionSendInviteAction:(VcxHandle)connectionHandle
                              data:(NSString *)data
                    withCompletion:(void (^)(NSError *error, NSString *message))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *data_ctype = [data cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_send_invite_action(handle,
                                                        connectionHandle,
                                                        data_ctype,
                                                        VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)connectionSignData:(VcxHandle)connectionHandle
                  withData:(NSData *)dataRaw
            withCompletion:(void (^)(NSError *error, NSData *signature_raw, vcx_u32_t signature_len))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    uint8_t *data_raw = (uint8_t *) [dataRaw bytes];
    uint32_t data_length = (uint32_t) [dataRaw length];

    vcx_error_t ret = vcx_connection_sign_data(handle, connectionHandle, data_raw, data_length, VcxWrapperCommonDataCallback);
    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil, 0);
        });
    }
}

- (void)connectionVerifySignature:(VcxHandle)connectionHandle
                         withData:(NSData *)dataRaw
                withSignatureData:(NSData *)signatureRaw
                   withCompletion:(void (^)(NSError *error, vcx_bool_t valid))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    uint8_t *data_raw = (uint8_t *) [dataRaw bytes];
    uint32_t data_length = (uint32_t) [dataRaw length];

    uint8_t *signature_raw = (uint8_t *) [signatureRaw bytes];
    uint32_t signature_length = (uint32_t) [signatureRaw length];

    vcx_error_t ret = vcx_connection_verify_signature(handle,
                                                      connectionHandle,
                                                      data_raw,
                                                      data_length,
                                                      signature_raw,
                                                      signature_length,
                                                      VcxWrapperCommonBoolCallback);
    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, false);
        });
    }
}


- (void)connectionUpdateState:(VcxHandle) connectionHandle
               withCompletion:(void (^)(NSError *error, NSInteger state))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_connection_update_state(handle, connectionHandle, VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}


- (void)connectionUpdateStateWithMessage:(VcxHandle) connectionHandle
                                 message:(NSString *)message
                          withCompletion:(void (^)(NSError *error, NSInteger state))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * cMessage = [message cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_connection_update_state_with_message(handle, connectionHandle, cMessage, VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}

- (void)connectionGetState:(VcxHandle) connectionHandle
            withCompletion:(void (^)(NSError *error, NSInteger state))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_credential_update_state(handle, connectionHandle, VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}

- (void)connectionGetProblemReport:(NSInteger) connectionHandle
                        completion:(void (^)(NSError *error, NSString *message))completion
{
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_connection_get_problem_report(handle,
                                                        connectionHandle,
                                                        VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)connectionUpgrade:(NSInteger) connectionHandle
                     data:data
               completion:(void (^)(NSError *error, NSString *serialized))completion
{
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * cData = [data cStringUsingEncoding:NSUTF8StringEncoding];
    vcx_error_t ret = vcx_connection_upgrade(handle, connectionHandle, cData, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)connectionNeedUpgrade:(NSString *) serializedConnection
                   completion:(void (^)(NSError *error, vcx_bool_t need))completion
{
  vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

  const char * cSerializedConnection = [serializedConnection cStringUsingEncoding:NSUTF8StringEncoding];
  vcx_error_t ret = vcx_connection_need_upgrade(handle, cSerializedConnection, VcxWrapperCommonBoolCallback);
  if (ret != 0) {
      [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

      NSError *error = [NSError errorFromVcxError:ret];
      dispatch_async(dispatch_get_main_queue(), ^{
          completion(error, false);
      });
  }
}

- (void)agentUpdateInfo: (NSString *) config
            completion: (void (^)(NSError *error)) completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *config_char = [config cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_agent_update_info(handle, config_char, VcxWrapperCommonCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error);
       });
   }
}


- (void)getCredential:(NSInteger)credentialHandle
           completion:(void (^)(NSError *error, NSString *credential))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_get_credential(handle, credentialHandle, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)deleteCredential:(NSInteger )credentialHandle
              completion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_delete_credential(handle, credentialHandle, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error);
       });
    }
}

- (void)credentialParseOffer:(NSString *)offer
                  completion:(void (^)(NSError *error, NSString *info))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * cOffer=[offer cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_credential_parse_offer(handle, cOffer, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error,  nil);
       });
   }
}

- (void)credentialCreateWithOffer:(NSString *)sourceId
            offer:(NSString *)credentialOffer
           completion:(void (^)(NSError *error, NSInteger credentialHandle))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * credential_offer=[credentialOffer cStringUsingEncoding:NSUTF8StringEncoding];
   const char * source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_credential_create_with_offer(handle, source_id,credential_offer, VcxWrapperCommonNumberCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void)credentialCreateWithMsgid:(NSString *)sourceId
                 connectionHandle:(VcxHandle)connectionHandle
                            msgId:(NSString *)msgId
                       completion:(void (^)(NSError *error, NSInteger credentialHandle, NSString *credentialOffer))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char * msg_id= [msgId cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_credential_create_with_msgid(handle, source_id, connectionHandle, msg_id, VcxWrapperCommonNumberStringCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0, nil);
       });
    }
}

- (void)credentialAcceptCredentialOffer:(NSString *)sourceId
                                  offer:(NSString *)credentialOffer
                       connectionHandle:(VcxHandle)connectionHandle
                             completion:(void (^)(NSError *error, NSInteger credentialHandle, NSString *credentialSerialized))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * credential_offer=[credentialOffer cStringUsingEncoding:NSUTF8StringEncoding];
   const char * source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_credential_accept_credential_offer(handle, source_id, credential_offer, connectionHandle, VcxWrapperCommonNumberStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0, nil);
       });
   }
}

- (void)credentialSendRequest:(NSInteger)credentialHandle
             connectionHandle:(VcxHandle)connectionHandle
                paymentHandle:(vcx_payment_handle_t)paymentHandle
                   completion:(void (^)(NSError *))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_credential_send_request(handle, credentialHandle, connectionHandle, paymentHandle, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error);
       });
    }
}

- (void)credentialReject:(NSInteger)credentialHandle
        connectionHandle:(VcxHandle)connectionHandle
                 comment:(NSString *)comment
              completion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * c_comment = [comment cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_credential_reject(handle, credentialHandle, connectionHandle, c_comment, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error);
       });
    }
}

- (void)credentialGetState:(NSInteger)credentialHandle
                completion:(void (^)(NSError *error, NSInteger state))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_credential_get_state(handle, credentialHandle, VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}

- (void)credentialUpdateState:(NSInteger)credentialHandle
                   completion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_credential_update_state(handle, credentialHandle, VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}

- (void)credentialUpdateStateWithMessage:(VcxHandle) credentialHandle
                                 message:(NSString *)message
                          withCompletion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * cMessage = [message cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_credential_update_state_with_message(handle, credentialHandle, cMessage, VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}

- (void)credentialGetOffers:(VcxHandle)credentialHandle
                 completion:(void (^)(NSError *error, NSString *offers))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_credential_get_offers(handle, credentialHandle, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (void)credentialGetPresentationProposal:(NSInteger )credentialHandle
                               completion:(void (^)(NSError *error, NSString *presentationProposal))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_credential_get_presentation_proposal_msg(handle, credentialHandle, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)generateProof:(NSString *)proofRequestId
       requestedAttrs:(NSString *)requestedAttrs
  requestedPredicates:(NSString *)requestedPredicates
   revocationInterval:(NSString *)revocationInterval
            proofName:(NSString *)proofName
           completion:(void (^)(NSError *error, NSString *proofHandle))completion
{
    vcx_error_t ret;

    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *proofRequestId_char = [proofRequestId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requestedAttrs_char = [requestedAttrs cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requestedPredicates_char = [requestedPredicates cStringUsingEncoding:NSUTF8StringEncoding];
    const char *revocationInterval_char = [proofName cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proofName_char = [proofName cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_proof_create(handle, proofRequestId_char, requestedAttrs_char, requestedPredicates_char, revocationInterval_char, proofName_char, VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void) requestProof:(vcx_proof_handle_t)proof_handle
 withConnectionHandle:(vcx_connection_handle_t)connection_handle
       requestedAttrs:(NSString *)requestedAttrs
  requestedPredicates:(NSString *)requestedPredicates
            proofName:(NSString *)proofName
   revocationInterval:(NSString *)revocationInterval
       withCompletion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *requestedAttrs_char = [requestedAttrs cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requestedPredicates_char = [requestedPredicates cStringUsingEncoding:NSUTF8StringEncoding];
    const char *revocationInterval_char = [proofName cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proofName_char = [proofName cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_proof_request_proof(handle, proof_handle, connection_handle, requestedAttrs_char, requestedPredicates_char, revocationInterval_char, proofName_char, VcxWrapperCommonCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)proofGetPresentationProposal:(vcx_proof_handle_t)proof_handle
                          completion:(void (^)(NSError *error, NSString *presentationProposal))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_get_proof_proposal(handle, proof_handle, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)credentialSerialize:(NSInteger)credentialHandle
                  completion:(void (^)(NSError *error, NSString *state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_credential_serialize(handle, credentialHandle, VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)credentialDeserialize:(NSString *)serializedCredential
                    completion:(void (^)(NSError *error, NSInteger credentialHandle))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_credential = [serializedCredential cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_credential_deserialize(handle, serialized_credential, VcxWrapperCommonNumberCallback);

    if ( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}

- (void)credentialGetProblemReport:(NSInteger) credentialHandle
                        completion:(void (^)(NSError *error, NSString *message))completion
{
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_credential_get_problem_report(handle,
                                                        credentialHandle,
                                                        VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)credentialGetInfo:(NSInteger) credentialHandle
               completion:(void (^)(NSError *error, NSString *message))completion
{
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_credential_get_info(handle,
                                              credentialHandle,
                                              VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (int)credentialRelease:(NSInteger) credentialHandle {
    return vcx_credential_release(credentialHandle);
}

- (void)exportWallet:(NSString *)exportPath
            encryptWith:(NSString *)encryptionKey
           completion:(void (^)(NSError *error, NSInteger exportHandle))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * export_path=[exportPath cStringUsingEncoding:NSUTF8StringEncoding];
   const char * encryption_key = [encryptionKey cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_wallet_export(handle, export_path, encryption_key, VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void)importWallet:(NSString *)config
           completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_wallet_import(handle, [config cStringUsingEncoding:NSUTF8StringEncoding], VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error);
       });
   }
}

- (void)addRecordWallet:(NSString *)recordType
               recordId:(NSString *)recordId
            recordValue:(NSString *) recordValue
             completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_value =[recordValue cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_tag = "{}";
    ret = vcx_wallet_add_record(handle, record_type, record_id, record_value, record_tag, VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error);
       });
   }
}

- (void)getRecordWallet:(NSString *)recordType
            recordId:(NSString *)recordId
             completion:(void (^)(NSError *error, NSString* walletValue))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_tag = "{}";
    ret = vcx_wallet_get_record(handle, record_type, record_id, record_tag, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (int)vcxShutdown:(BOOL *) deleteWallet {
    int delete_wallet = deleteWallet;
    return vcx_shutdown(delete_wallet);
}

- (void)deleteRecordWallet:(NSString *)recordType
            recordId:(NSString *)recordId
           completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
   const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_wallet_delete_record(handle, record_type, record_id, VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error);
       });
   }
}

- (void)updateRecordWallet:(NSString *)recordType
              withRecordId:(NSString *)recordId
           withRecordValue:(NSString *) recordValue
            withCompletion:(void (^)(NSError *error))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_value =[recordValue cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_update_record_value(handle, record_type, record_id, record_value, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)proofGetRequests:(NSInteger)connectionHandle
                   completion:(void (^)(NSError *error, NSString *requests))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_get_requests(handle,connectionHandle, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (void) proofParseRequest:(NSString *)request
            withCompletion:(void (^)(NSError *error, NSString *info))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *cRequest = [request cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_disclosed_proof_parse_request(handle, cRequest, VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) proofCreateWithMsgId:(NSString *)sourceId
         withConnectionHandle:(vcx_connection_handle_t)connectionHandle
                    withMsgId:(NSString *)msgId
               withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle, NSString *proofRequest))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *source_id = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
    const char *msg_id = [msgId cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_disclosed_proof_create_with_msgid(handle, source_id, connectionHandle, msg_id, VcxWrapperCommonNumberStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0, nil);
        });
    }
}

- (void) proofRetrieveCredentials:(vcx_proof_handle_t)proofHandle
                   withCompletion:(void (^)(NSError *error, NSString *matchingCredentials))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_retrieve_credentials(handle, proofHandle, VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) proofGenerate:(vcx_proof_handle_t)proofHandle
withSelectedCredentials:(NSString *)selectedCredentials
 withSelfAttestedAttrs:(NSString *)selfAttestedAttributes
        withCompletion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *selected_credentials = [selectedCredentials cStringUsingEncoding:NSUTF8StringEncoding];
    const char *self_attested_attributes = [selfAttestedAttributes cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_disclosed_proof_generate_proof(handle, proofHandle, selected_credentials, self_attested_attributes, VcxWrapperCommonCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) proofSend:(vcx_proof_handle_t)proof_handle
withConnectionHandle:(vcx_connection_handle_t)connection_handle
    withCompletion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_send_proof(handle, proof_handle, connection_handle, VcxWrapperCommonCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) proofSendProposal:(vcx_proof_handle_t)proof_handle
      withConnectionHandle:(vcx_connection_handle_t)connection_handle
            withCompletion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_send_proposal(handle, proof_handle, connection_handle, VcxWrapperCommonCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)proofGetState:(NSInteger)proofHandle
                completion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_disclosed_proof_get_state(handle, proofHandle, VcxWrapperCommonNumberCallback);

    if( ret != 0 ) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (void)proofUpdateState:(NSInteger) proofHandle
                   completion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_disclosed_proof_update_state(handle, proofHandle, VcxWrapperCommonNumberCallback);
    if( ret != 0 ) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (void) proofReject: (vcx_proof_handle_t)proof_handle withConnectionHandle:(vcx_connection_handle_t)connection_handle
      withCompletion: (void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor: completion];

    ret = vcx_disclosed_proof_reject_proof(handle, proof_handle, connection_handle, VcxWrapperCommonCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) proofDeclinePresentationRequest:(vcx_proof_handle_t)proof_handle
                    withConnectionHandle:(vcx_connection_handle_t)connection_handle
                              withReason:(NSString *)reason
                            withProposal:(NSString *)proposal
                          withCompletion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *c_reason = [reason cStringUsingEncoding:NSUTF8StringEncoding];
    const char *c_proposal = [proposal cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_disclosed_proof_decline_presentation_request(handle, proof_handle, connection_handle, c_reason, c_proposal, VcxWrapperCommonCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) getProofMsg:(vcx_proof_handle_t) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *proofMsg))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_get_proof_msg(handle, proofHandle, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) getRejectMsg:(vcx_proof_handle_t) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *rejectMsg))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_get_reject_msg(handle, proofHandle, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) connectionRedirect: (vcx_connection_handle_t) redirect_connection_handle
        withConnectionHandle: (vcx_connection_handle_t) connection_handle
        withCompletion: (void (^)(NSError *error)) completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor: completion];

    ret = vcx_connection_redirect(handle, connection_handle, redirect_connection_handle, VcxWrapperCommonCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) getRedirectDetails: (vcx_connection_handle_t) connection_handle
        withCompletion: (void (^)(NSError *error, NSString *redirectDetails)) completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_connection_get_redirect_details(handle, connection_handle, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) proofCreateWithRequest:(NSString *) source_id
               withProofRequest:(NSString *) proofRequest
                 withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId = [source_id cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proof_request = [proofRequest cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_disclosed_proof_create_with_request(handle, sourceId, proof_request, VcxWrapperCommonNumberCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (void) proofCreateProposal:(NSString *) source_id
           withProofProposal:(NSString *) proofProposal
                 withComment:(NSString *) comment
              withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *sourceId = [source_id cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proof_proposal = [proofProposal cStringUsingEncoding:NSUTF8StringEncoding];
    const char *comment_char = [comment cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_disclosed_proof_create_proposal(handle, sourceId, proof_proposal, comment_char, VcxWrapperCommonNumberCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (void) proofSerialize:(vcx_proof_handle_t) proofHandle
         withCompletion:(void (^)(NSError *error, NSString *proof_request))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_disclosed_proof_serialize(handle, proofHandle, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) proofDeserialize:(NSString *) serializedProof
           withCompletion:(void (^)(NSError *error, vcx_proof_handle_t proofHandle)) completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_proof = [serializedProof cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_disclosed_proof_deserialize(handle, serialized_proof, VcxWrapperCommonNumberCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (int)proofRelease:(NSInteger) proofHandle {
    return vcx_disclosed_proof_release(proofHandle);
}
- (void)proofUpdateStateWithMessage:(VcxHandle) proofHandle
                            message:(NSString *)message
                     withCompletion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * cMessage = [message cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_disclosed_proof_update_state_with_message(handle, proofHandle, cMessage, VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}

- (void)proofGetProblemReport:(VcxHandle) proofHandle
                   completion:(void (^)(NSError *error, NSString *message))completion
{
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_disclosed_proof_get_problem_report(handle,
                                                             proofHandle,
                                                             VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
    }
}

- (void)createPaymentAddress:(NSString *)seed
              withCompletion:(void (^)(NSError *error, NSString *address))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    const char *c_seed = [seed cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_create_payment_address(handle, c_seed, VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)getTokenInfo:(vcx_payment_handle_t)payment_handle
      withCompletion:(void (^)(NSError *error, NSString *tokenInfo))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_wallet_get_token_info(handle, payment_handle, VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)sendTokens:(vcx_payment_handle_t)payment_handle
        withTokens:(NSString *)tokens
     withRecipient:(NSString *)recipient
    withCompletion:(void (^)(NSError *error, NSString *recipient))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    const char* c_recipient = [recipient cStringUsingEncoding:NSUTF8StringEncoding];
    const char* c_tokens = [tokens cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_send_tokens(handle, payment_handle, c_tokens, c_recipient, VcxWrapperCommonStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)downloadMessages:(NSString *)messageStatus
                    uid_s:(NSString *)uid_s
                  pwdids:(NSString *)pwdids
              completion:(void (^)(NSError *error, NSString* messages))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char * uids = [uid_s cStringUsingEncoding:NSUTF8StringEncoding];
    const char * pw_dids = [pwdids cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_messages_download(handle, message_status, uids, pw_dids, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)downloadMessage:(NSString *)uid
             completion:(void (^)(NSError *error, NSString* messages))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * uid_ = [uid cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_download_message(handle, uid_, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)downloadAgentMessages:(NSString *)messageStatus
                        uid_s:(NSString *)uid_s
                        completion:(void (^)(NSError *error, NSString* messages))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char * uids = [uid_s cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_download_agent_messages(handle, message_status, uids, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)updateMessages:(NSString *)messageStatus
                 pwdidsJson:(NSString *)pwdidsJson
              completion:(void (^)(NSError *error))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_status = [messageStatus cStringUsingEncoding:NSUTF8StringEncoding];
    const char * msg_json = [pwdidsJson cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_messages_update_status(handle, message_status, msg_json, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) getLedgerFees:(void(^)(NSError *error, NSString *fees)) completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_ledger_get_fees(handle, VcxWrapperCommonStringCallback);


    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)fetchPublicEntities:(void (^)(NSError *error))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_fetch_public_entities(handle, VcxWrapperCommonCallback);

    if( ret != 0 ) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)healthCheck:(void (^)(NSError *error))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_health_check(handle, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion([NSError errorFromVcxError: ret]);
        });
    }
}


//vcx_error_t vcx_wallet_backup_create(vcx_command_handle_t command_handle, const char *source_id, const char *backup_key,
//                                     void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_wallet_backup_handle_t));

- (void) createWalletBackup:(NSString *)sourceID
                 backupKey:(NSString *)backupKey
                 completion:(void (^)(NSError *error, NSInteger walletBackupHandle))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * source_id = [sourceID cStringUsingEncoding:NSUTF8StringEncoding];
    const char * backup_key = [backupKey cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_backup_create(handle, source_id, backup_key, VcxWrapperCommonNumberCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}
//vcx_error_t vcx_wallet_backup_backup(vcx_command_handle_t command_handle, vcx_wallet_backup_handle_t wallet_backup_handle, const char *path,
//    void (*cb)(vcx_command_handle_t, vcx_error_t));
- (void) backupWalletBackup:(vcx_wallet_backup_handle_t) walletBackupHandle
                   path:(NSString *)path
                   completion:(void(^)(NSError *error))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * new_path = [path cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_backup_backup(handle, walletBackupHandle, new_path, VcxWrapperCommonCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }

}

//vcx_error_t vcx_wallet_backup_update_state(vcx_command_handle_t command_handle, vcx_wallet_backup_handle_t wallet_backup_handle,
//                                           void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

- (void) updateWalletBackupState:(vcx_wallet_backup_handle_t) walletBackupHandle
                      completion:(void (^)(NSError *error, NSInteger state))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_wallet_backup_update_state(handle, walletBackupHandle, VcxWrapperCommonNumberCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }

}

//vcx_error_t vcx_wallet_backup_update_state_with_message(vcx_command_handle_t command_handle, vcx_wallet_backup_handle_t wallet_backup_handle, const char *message,
//                                                        void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_state_t));

- (void) updateWalletBackupStateWithMessage:(vcx_wallet_backup_handle_t) walletBackupHandle
                      message:(NSString *)message
                      completion:(void (^)(NSError *error, NSInteger state))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * new_message = [message cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_wallet_backup_update_state_with_message(handle, walletBackupHandle, new_message, VcxWrapperCommonNumberCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }

}

//vcx_error_t vcx_wallet_backup_serialize(vcx_command_handle_t command_handle, vcx_wallet_backup_handle_t wallet_backup_handle,
//                                        void (*cb)(vcx_command_handle_t, vcx_error_t, const char*));

- (void) serializeBackupWallet:(vcx_wallet_backup_handle_t) walletBackupHandle
                      completion:(void (^)(NSError *error, NSString *data))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_wallet_backup_serialize(handle, walletBackupHandle, VcxWrapperCommonStringCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

//vcx_error_t vcx_wallet_backup_deserialize(vcx_command_handle_t command_handle, const char *wallet_backup_str,
//                                          void (*cb)(vcx_command_handle_t, vcx_error_t, vcx_wallet_backup_handle_t));
- (void) deserializeBackupWallet:(NSString *) walletBackupStr
              completion:(void (^)(NSError *error, NSInteger walletBackupHandle))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * wallet_backup_str = [walletBackupStr cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_wallet_backup_deserialize(handle, wallet_backup_str, VcxWrapperCommonNumberCallback);

    if (ret != 0)
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }

}


- (void)restoreWallet:(NSString *)config
           completion:(void (^)(NSError *error))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_wallet_backup_restore(handle, [config cStringUsingEncoding:NSUTF8StringEncoding], VcxWrapperCommonCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error);
       });
   }
}

- (void) createPairwiseAgent:(void (^)(NSError *error, NSString *agentInfo))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    vcx_error_t ret = vcx_create_pairwise_agent(handle,
                                                VcxWrapperCommonStringCallback);
    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}


/*
* Verifier API
*/
- (void) createProofVerifier:(NSString *) sourceId
         requestedAttributes:(NSString *)requestedAttributes
         requestedPredicates:(NSString *)requestedPredicates
          revocationInterval:(NSString *)revocationInterval
                   proofName:(NSString *)proofName
                  completion:(void (^)(NSError *error, NSInteger handle))completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
   const char *requestedAttributes_char = [requestedAttributes cStringUsingEncoding:NSUTF8StringEncoding];
   const char *requestedPredicates_char = [requestedPredicates cStringUsingEncoding:NSUTF8StringEncoding];
   const char *revocationInterval_char = [revocationInterval cStringUsingEncoding:NSUTF8StringEncoding];
   const char *proofName_char = [proofName cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_proof_create(handle, sourceId_char, requestedAttributes_char, requestedPredicates_char,
        revocationInterval_char, proofName_char, VcxWrapperCommonHandleCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void) createProofVerifierWithProposal:(NSString *) sourceId
                    presentationProposal:(NSString *)presentationProposal
                                    name:(NSString *)name
                              completion:(void (^)(NSError *error, NSInteger handle))completion
{
   vcx_error_t ret;

   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *sourceId_char = [sourceId cStringUsingEncoding:NSUTF8StringEncoding];
   const char *presentationProposal_char = [presentationProposal cStringUsingEncoding:NSUTF8StringEncoding];
   const char *proofName_char = [name cStringUsingEncoding:NSUTF8StringEncoding];
   ret = vcx_proof_create_with_proposal(handle, sourceId_char, presentationProposal_char, proofName_char, VcxWrapperCommonHandleCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void) proofVerifierUpdateState:(NSInteger) proofHandle
                       completion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_proof_update_state(handle, proofHandle, VcxWrapperCommonNumberCallback);
    if( ret != 0 ) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (void) proofVerifierUpdateStateWithMessage:(NSInteger) proofHandle
                                    message:(NSString *)message
                                 completion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * cMessage = [message cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_proof_update_state_with_message(handle, proofHandle, cMessage, VcxWrapperCommonNumberCallback);

    if( ret != 0 )
    {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
    }
}

- (void) proofVerifierGetState:(NSInteger) proofHandle
                    completion:(void (^)(NSError *error, NSInteger state))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_proof_get_state(handle, proofHandle, VcxWrapperCommonNumberCallback);
    if( ret != 0 ) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0);
        });
    }
}

- (void) proofVerifierSerialize:(NSInteger) proofHandle
                     completion:(void (^)(NSError *error, NSString* serialized))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_proof_serialize(handle, proofHandle, VcxWrapperCommonStringCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (void) proofVerifierDeserialize:(NSString *) serialized
                       completion:(void (^)(NSError *error, NSInteger proofHandle))completion{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *serialized_char=[serialized cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_proof_deserialize(handle, serialized_char, VcxWrapperCommonHandleCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void) proofVerifierSendRequest:(NSInteger) proofHandle
                 connectionHandle:(NSInteger) connectionHandle
                       completion:(void (^)(NSError *error))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_proof_send_request(handle, proofHandle, connectionHandle , VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) proofVerifierRequestForProposal:(NSInteger) proofHandle
                        connectionHandle:(NSInteger) connectionHandle
                     requestedAttributes:(NSString *)requestedAttributes
                     requestedPredicates:(NSString *)requestedPredicates
                      revocationInterval:(NSString *)revocationInterval
                               proofName:(NSString *)proofName
                              completion:(void (^)(NSError *error))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *requestedAttributes_char=[requestedAttributes cStringUsingEncoding:NSUTF8StringEncoding];
    const char *requestedPredicates_char=[requestedPredicates cStringUsingEncoding:NSUTF8StringEncoding];
    const char *revocationInterval_char=[revocationInterval cStringUsingEncoding:NSUTF8StringEncoding];
    const char *proofName_char=[proofName cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_proof_request_proof(handle, proofHandle, connectionHandle, requestedAttributes_char,
        requestedPredicates_char, revocationInterval_char, proofName_char, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) proofVerifierGetProofRequestMessage:(NSInteger) proofHandle
                                 completion:(void (^)(NSError *error, NSString* message))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_proof_get_request_msg(handle, proofHandle, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) proofVerifierGetProofProposalMessage:(NSInteger) proofHandle
                                 completion:(void (^)(NSError *error, NSString* message))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_get_proof_proposal(handle, proofHandle, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) proofVerifierGetProofRequestAttach:(NSInteger) proofHandle
                                 completion:(void (^)(NSError *error, NSString* message))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_proof_get_request_attach(handle, proofHandle, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) proofVerifierGetProblemReportMessage:(NSInteger) proofHandle
                                   completion:(void (^)(NSError *error, NSString* message))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_proof_get_problem_report(handle, proofHandle, VcxWrapperCommonStringCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) proofVerifierSetConnection:(NSInteger) proofHandle
                            connectionHandle:(NSInteger) connectionHandle
                                 completion:(void (^)(NSError *error))completion
{
    vcx_error_t ret;
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    ret = vcx_proof_set_connection(handle, proofHandle, connectionHandle, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void) proofVerifierGetProofMessage:(NSInteger) proofHandle
                           completion:(void (^)(NSError *error, NSInteger proofState, NSString* message))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_get_proof_msg(handle, proofHandle, VcxWrapperCommonNumberStringCallback);

    if ( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, 0, nil);
        });
    }
}

- (int) proofVerifierProofRelease:(NSInteger) connectionHandle
                       completion:(void (^)(NSError *error))completion {
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

  return vcx_proof_release(handle, connectionHandle);
}

- (void)connectionSendDiscoveryFeatures:(VcxHandle)connectionHandle
                   comment:(NSString *)comment
                   query:(NSString *)query
            withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char *comment_ctype = [comment cStringUsingEncoding:NSUTF8StringEncoding];
    const char *query_ctype = [query cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_connection_send_discovery_features(handle,
                                                  connectionHandle,
                                                  comment_ctype,
                                                  query_ctype,
                                                  VcxWrapperCommonCallback);
    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)connectionGetPwDid:(VcxHandle)connectionHandle
            withCompletion:(void (^)(NSError *error, NSString* pwDid))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   ret = vcx_connection_get_pw_did(handle, connectionHandle, VcxWrapperCommonStringCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (void)connectionGetTheirDid:(VcxHandle)connectionHandle
               withCompletion:(void (^)(NSError *error, NSString* pwDid))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   ret = vcx_connection_get_their_pw_did(handle, connectionHandle, VcxWrapperCommonStringCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (void)connectionInfo:(VcxHandle)connectionHandle
        withCompletion:(void (^)(NSError *error, NSString* info))completion {
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

   ret = vcx_connection_info(handle, connectionHandle, VcxWrapperCommonStringCallback);
   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, nil);
       });
   }
}

- (void)credentialGetRequestMsg:(VcxHandle)credentialHandle
                        myPwDid:(NSString *)myPwDid
                     theirPwDid:(NSString *)theirPwDid
                  paymentHandle:(vcx_payment_handle_t)paymentHandle
                 withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle= [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * my_pw_did = [myPwDid cStringUsingEncoding:NSUTF8StringEncoding];
    const char * their_pw_did = [theirPwDid cStringUsingEncoding:NSUTF8StringEncoding];

    vcx_error_t ret = vcx_credential_get_request_msg(
                                                     handle,
                                                     credentialHandle,
                                                     my_pw_did, their_pw_did,
                                                     paymentHandle,
                                                     VcxWrapperCommonCallback
                                                    );

    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)anonDecrypt:(VcxHandle)walletHandle
        recipientVk:(NSString *)recipientVk
       encryptedMsg:(NSData *)encryptedMsg
     withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * recipient_vk_ctype = [recipientVk cStringUsingEncoding:NSUTF8StringEncoding];
    uint8_t * encrypted_msg_u8 = (uint8_t *) [encryptedMsg bytes];

    vcx_error_t ret = indy_crypto_anon_decrypt(
                                               handle,
                                               walletHandle,
                                               recipient_vk_ctype,
                                               encrypted_msg_u8,
                                               VcxWrapperCommonCallback
                                              );

    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)signWithAddress:(NSString *)address
                message:(NSData *)message
         withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * address_ctype = [address cStringUsingEncoding:NSUTF8StringEncoding];
    uint8_t * message_u8 = (uint8_t *) [message bytes];

    vcx_error_t ret = vcx_wallet_sign_with_address(
                                               handle,
                                               address_ctype,
                                               message_u8,
                                               sizeof(message_u8)/sizeof(uint8_t),
                                               VcxWrapperCommonCallback
                                              );

    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)verifyWithAddress:(NSString *)address
                  message:(NSData *)message
                signature:(NSData *)signature
           withCompletion:(void (^)(NSError *error))completion
{
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * address_ctype = [address cStringUsingEncoding:NSUTF8StringEncoding];
    uint8_t * message_u8 = (uint8_t *) [message bytes];
    uint8_t * signature_u8 = (uint8_t *) [signature bytes];

    vcx_error_t ret = vcx_wallet_verify_with_address(
                                               handle,
                                               address_ctype,
                                               message_u8,
                                               sizeof(message_u8)/sizeof(uint8_t),
                                               signature_u8,
                                               sizeof(signature_u8)/sizeof(uint8_t),
                                               VcxWrapperCommonCallback
                                              );

    if (ret != 0) {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor:handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)walletAddRecordTags:(NSString *)recordType
                   recordId:(NSString *)recordId
                 recordTags:(NSString *)recordTags
                 completion:(void (^)(NSError *error))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_tags =[recordTags cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_add_record_tags(handle, record_type, record_id, record_tags, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)walletUpdateRecordTags:(NSString *)recordType
                      recordId:(NSString *)recordId
                    recordTags:(NSString *)recordTags
                    completion:(void (^)(NSError *error))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_tags =[recordTags cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_update_record_tags(handle, record_type, record_id, record_tags, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)walletDeleteRecordTags:(NSString *)recordType
                      recordId:(NSString *)recordId
                    recordTags:(NSString *)recordTags
                    completion:(void (^)(NSError *error))completion {

    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * record_type =[recordType cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_id = [recordId cStringUsingEncoding:NSUTF8StringEncoding];
    const char * record_tags =[recordTags cStringUsingEncoding:NSUTF8StringEncoding];

    ret = vcx_wallet_delete_record_tags(handle, record_type, record_id, record_tags, VcxWrapperCommonCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)walletOpenSearch:(NSString *)type
                   query:(NSString *)query
                 options:(NSString *)options
              completion:(void (^)(NSError *error, NSInteger searchHandle)) completion
{
   vcx_error_t ret;
   vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
   const char *type_char = [type cStringUsingEncoding:NSUTF8StringEncoding];
   const char *query_char = [query cStringUsingEncoding:NSUTF8StringEncoding];
   const char *options_char = [options cStringUsingEncoding:NSUTF8StringEncoding];

   ret = vcx_wallet_open_search(handle, type_char, query_char, options_char, VcxWrapperCommonHandleCallback);

   if( ret != 0 )
   {
       [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

       NSError *error = [NSError errorFromVcxError:ret];
       dispatch_async(dispatch_get_main_queue(), ^{
           completion(error, 0);
       });
   }
}

- (void) walletSearchNextRecords:(NSInteger)searchHandle
                           count:(NSInteger)count
                      completion:(void (^)(NSError *error, NSString *records))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_wallet_search_next_records(handle, searchHandle, count, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void) walletCloseSearch:(NSInteger)searchHandle
                completion:(void (^)(NSError *error))completion {
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];

    ret = vcx_wallet_close_search(handle, searchHandle, VcxWrapperCommonHandleCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error);
        });
    }
}

- (void)extractAttachedMessage:(NSString *)message
             completion:(void (^)(NSError *error, NSString* attachedMessage))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_ = [message cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_extract_attached_message(handle, message_, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)extractThreadId:(NSString *)message
             completion:(void (^)(NSError *error, NSString* threadId))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * message_ = [message cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_extract_thread_id(handle, message_, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

- (void)resolveMessageByUrl:(NSString *)url
                 completion:(void (^)(NSError *error, NSString* message))completion{
    vcx_error_t ret;
    vcx_command_handle_t handle = [[VcxCallbacks sharedInstance] createCommandHandleFor:completion];
    const char * url_ = [url cStringUsingEncoding:NSUTF8StringEncoding];
    ret = vcx_resolve_message_by_url(handle, url_, VcxWrapperCommonStringCallback);

    if( ret != 0 )
    {
        [[VcxCallbacks sharedInstance] deleteCommandHandleFor: handle];

        NSError *error = [NSError errorFromVcxError:ret];
        dispatch_async(dispatch_get_main_queue(), ^{
            completion(error, nil);
        });
    }
}

@end
