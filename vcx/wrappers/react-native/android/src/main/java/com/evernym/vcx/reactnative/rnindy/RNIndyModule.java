//  Created by react-native-create-bridge

package com.evernym.vcx.reactnative.rnindy;

import android.Manifest;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.DialogInterface;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.Color;
import androidx.core.app.ActivityCompat;
import android.util.Base64;
import android.util.Log;
import android.app.ActivityManager;
import android.app.ActivityManager.MemoryInfo;
import android.content.Context;
import android.content.ContextWrapper;
import android.net.Uri;
import android.os.Environment;

import com.evernym.vcx.reactnative.BridgeUtils;
import com.evernym.sdk.vcx.VcxException;
import com.evernym.sdk.vcx.wallet.WalletApi;
import com.evernym.sdk.vcx.connection.ConnectionApi;
import com.evernym.sdk.vcx.credential.CredentialApi;
import com.evernym.sdk.vcx.credential.GetCredentialCreateMsgidResult;
import com.evernym.sdk.vcx.proof.CreateProofMsgIdResult;
import com.evernym.sdk.vcx.proof.DisclosedProofApi;
import com.evernym.sdk.vcx.proof.ProofApi;
import com.evernym.sdk.vcx.proof.GetProofResult;
import com.evernym.sdk.vcx.proof.CreateProofMsgIdResult;
import com.evernym.sdk.vcx.token.TokenApi;
import com.evernym.sdk.vcx.utils.UtilsApi;
import com.evernym.sdk.vcx.vcx.AlreadyInitializedException;
import com.evernym.sdk.vcx.vcx.VcxApi;
import com.evernym.sdk.vcx.issuer.IssuerApi;
import com.evernym.sdk.vcx.indy.IndyApi;
import com.facebook.react.bridge.Arguments;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.bridge.WritableArray;
import com.facebook.react.bridge.WritableMap;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.FileWriter;
import java.io.IOException;
import java.io.RandomAccessFile;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.SecureRandom;
import java.util.Arrays;
import java.util.List;
import java.util.Scanner;
import java.util.Timer;
import java.util.TimerTask;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;
import java.io.InputStream;
import java.util.HashMap;
import java.util.Map;
import java.net.HttpURLConnection;
import java.net.MalformedURLException;
import java.net.URL;

import javax.annotation.Nullable;

public class RNIndyModule extends ReactContextBaseJavaModule {
    public static final String REACT_CLASS = "RNIndy";
    public static final String TAG = "RNIndy::";
    private static final int BUFFER = 2048;
    private static ReactApplicationContext reactContext = null;
    private static RNIndyStaticData staticData = new RNIndyStaticData();

    public RNIndyModule(ReactApplicationContext context) {
        // Pass in the context to the constructor and save it so you can emit events
        // https://facebook.github.io/react-native/docs/native-modules-android.html#the-toast-module
        super(context);

        reactContext = context;
    }

    @Override
    public String getName() {
        // Tell React the name of the module
        // https://facebook.github.io/react-native/docs/native-modules-android.html#the-toast-module
        return REACT_CLASS;
    }

    /*
     * Agent API
     */
    @ReactMethod
    public void vcxUpdatePushToken(String config, Promise promise) {
        try {
            UtilsApi.vcxUpdateAgentInfo(config).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "vcxUpdateAgentInfo - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxUpdateAgentInfo - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void createOneTimeInfo(String agencyConfig, Promise promise) {
        Log.d(TAG, "createOneTimeInfo()");
        // We have top create thew ca cert for the openssl to work properly on android
        BridgeUtils.writeCACert(this.getReactApplicationContext());

        try {
            UtilsApi.vcxAgentProvisionAsync(agencyConfig).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxAgentProvisionAsync - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                Log.d(TAG, "vcxGetProvisionToken: Success");
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxAgentProvisionAsync - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void createOneTimeInfoWithToken(String agencyConfig, String token, Promise promise) {
        Log.d(TAG, "createOneTimeInfoWithToken()");
        BridgeUtils.writeCACert(this.getReactApplicationContext());

        try {
            String result = UtilsApi.vcxAgentProvisionWithToken(agencyConfig, token);
            BridgeUtils.resolveIfValid(promise, result);
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxAgentProvisionWithToken - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void agentProvisionWithTokenAsync(String agencyConfig, String token, Promise promise) {
        Log.d(TAG, "agentProvisionWithTokenAsync()");
        // We have top create thew ca cert for the openssl to work properly on android
        BridgeUtils.writeCACert(this.getReactApplicationContext());

        try {
            UtilsApi.vcxAgentProvisionWithTokenAsync(agencyConfig, token).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "agentProvisionWithTokenAsync - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                Log.d(TAG, "agentProvisionWithTokenAsync: Success");
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "agentProvisionWithTokenAsync - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }


    @ReactMethod
    public void getProvisionToken(String agencyConfig, Promise promise) {
        Log.d(TAG, "getProvisionToken()");
        try {
            UtilsApi.vcxGetProvisionToken(agencyConfig)
              .exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxGetProvisionToken - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
              }).thenAccept(result -> {
                Log.d(TAG, "vcxGetProvisionToken: Success");
                BridgeUtils.resolveIfValid(promise, result);
              });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxGetProvisionToken - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void downloadMessages(String messageStatus, String uid_s, String pwdids, Promise promise) {
        Log.d(TAG, "downloadMessages()");
        try {
            UtilsApi.vcxGetMessages(messageStatus, uid_s, pwdids).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxGetMessages - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
        }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));

        } catch (VcxException e) {
              e.printStackTrace();
              Log.e(TAG, "vcxGetMessages - Error: ", e);
              promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void vcxGetAgentMessages(String messageStatus, String uid_s, Promise promise) {
        Log.d(TAG, "vcxGetAgentMessages()");
        try {
            UtilsApi.vcxGetAgentMessages(messageStatus, uid_s).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxGetAgentMessages - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));

        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxGetAgentMessages - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void updateMessages(String messageStatus, String pwdidsJson, Promise promise) {
        Log.d(TAG, "updateMessages()");

        try {
            UtilsApi.vcxUpdateMessages(messageStatus, pwdidsJson).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "vcxUpdateMessages - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxUpdateMessages - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void createPairwiseAgent(Promise promise) {
        try {
            UtilsApi.vcxCreatePairwiseAgent().exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "createPairwiseAgent - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "createPairwiseAgent - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    /*
     * Cloud Wallet Backup API
     */
    @ReactMethod
    public void restoreWallet(String config, Promise promise) {
        try {
            WalletApi.restoreWalletBackup(config).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "restoreWalletBackup - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "restoreWalletBackup - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void createWalletBackup(String sourceID, String backupKey, Promise promise) {
        try {
            WalletApi.createWalletBackup(sourceID, backupKey).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "createWalletBackup - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "createWalletBackup - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void backupWalletBackup(int walletBackupHandle, String path, Promise promise) {
        try {
            WalletApi.backupWalletBackup(walletBackupHandle, path).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "backupWalletBackup - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "createWalletBackup - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void updateWalletBackupStateWithMessage(int walletBackupHandle, String message, Promise promise ) {
        try {
            WalletApi.updateWalletBackupStateWithMessage(walletBackupHandle, message ).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "updateWalletBackupStateWithMessage - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "updateWalletBackupState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    /*
     * Connection API
     */
    @ReactMethod
    public void createConnection(String sourceId, Promise promise) {
        Log.d(TAG, "vcxConnectionCreate()");
        try {
            ConnectionApi.vcxConnectionCreate(sourceId).whenComplete((result, e) -> {
                if (e != null) {
                    VcxException ex = (VcxException) e;
                    ex.printStackTrace();
                    Log.e(TAG, "vcxConnectionCreate - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxConnectionCreate - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void deleteConnection(int connectionHandle, Promise promise) {
        try {
            ConnectionApi.deleteConnection(connectionHandle).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "deleteConnection - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "deleteConnection - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void createConnectionWithInvite(String invitationId, String inviteDetails, Promise promise) {
        Log.d(TAG, "createConnectionWithInvite()");
        try {
            ConnectionApi.vcxCreateConnectionWithInvite(invitationId, inviteDetails).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxCreateConnectionWithInvite - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });

        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxCreateConnectionWithInvite - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void createConnectionWithOutofbandInvite(String invitationId, String inviteDetails, Promise promise) {
        Log.d(TAG, "createConnectionWithOutofbandInvite()");
        try {
            ConnectionApi.vcxCreateConnectionWithOutofbandInvite(invitationId, inviteDetails).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "createConnectionWithOutofbandInvite - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });

        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxCreateConnectionWithInvite - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionConnect(int connectionHandle, String options, Promise promise) {
        Log.d(TAG, "connectionConnect()");
        try {
            ConnectionApi.vcxConnectionConnect(connectionHandle, options).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionConnect - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionConnect - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }


    @ReactMethod
    public void getSerializedConnection(int connectionHandle, Promise promise) {
        // TODO:KS call vcx_connection_serialize and pass connectionHandle
        try {
            ConnectionApi.connectionSerialize(connectionHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionSerialize - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionSerialize - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void deserializeConnection(String serializedConnection, Promise promise) {
        // TODO call vcx_connection_deserialize and pass serializedConnection
        // it would return an error code and an integer connection handle in callback
        try {
            ConnectionApi.connectionDeserialize(serializedConnection).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionDeserialize - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionDeserialize - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionSendMessage(int connectionHandle, String message, String sendMessageOptions, Promise promise) {
        try {
            ConnectionApi.connectionSendMessage(connectionHandle, message, sendMessageOptions).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionSendMessage - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionSendMessage - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionSignData(int connectionHandle, String data, String base64EncodingOption, boolean encode, Promise promise) {
        try {
            int base64EncodeOption = base64EncodingOption.equalsIgnoreCase("NO_WRAP") ? Base64.NO_WRAP : Base64.URL_SAFE;
            byte[] dataToSign = encode ? Base64.encode(data.getBytes(), base64EncodeOption) : data.getBytes();
            ConnectionApi.connectionSignData(connectionHandle, dataToSign, dataToSign.length).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionSignData - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                try {
                    // We would get Byte array from libvcx
                    // we cannot perform operation on Buffer inside react-native due to react-native limitations for Buffer
                    // so, we are converting byte[] to Base64 encoded string and then returning that data to react-native
                    if (result != null) {
                        // since we took the data from JS layer as simple string and
                        // then converted that string to Base64 encoded byte[]
                        // we need to pass same Base64 encoded byte[] back to JS layer, so that it can included in full message response
                        // otherwise we would be doing this calculation again in JS layer which does not handle Buffer
                        WritableMap signResponse = Arguments.createMap();
                        signResponse.putString("data", new String(dataToSign));
                        signResponse.putString("signature", Base64.encodeToString(result, base64EncodeOption));
                        promise.resolve(signResponse);
                    } else {
                        promise.reject("NULL-VALUE", "Null value was received as result from wrapper");
                    }
                } catch(Exception e) {
                    // it might happen that we get value of result to not be a byte array
                    // or we might get empty byte array
                    // in all those case outer try...catch will not work because this inside callback of a Future
                    // so we need to handle the case for Future callback inside that callback
                    promise.reject(e);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionSignData - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionVerifySignature(int connectionHandle, String data, String signature, Promise promise) {
        // Base64 decode signature because we encoded signature returned by libvcx to base64 encoded string
        // Convert data to just byte[], because base64 encoded byte[] was used to generate signature
        byte[] dataToVerify = data.getBytes();
        byte[] signatureToVerify = Base64.decode(signature, Base64.NO_WRAP);
        try {
            ConnectionApi.connectionVerifySignature(
                    connectionHandle, dataToVerify, dataToVerify.length, signatureToVerify, signatureToVerify.length
            ).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionVerifySignature - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionVerifySignature - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionRedirect(int redirectConnectionHandle, int connectionHandle, Promise promise) {
        Log.d(TAG, "connectionRedirect()");

        try {
            ConnectionApi.vcxConnectionRedirect(connectionHandle, redirectConnectionHandle).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "vcxConnectionRedirect - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxConnectionRedirect - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionReuse(int connectionHandle, String invite, Promise promise) {
        Log.d(TAG, "connectionReuse() called with connectionHandle = " + connectionHandle + ", promise = " + promise);

        try {
            ConnectionApi.connectionSendReuse(connectionHandle, invite).whenComplete((result, t) -> {
                if (t != null) {
                    Log.e(TAG, "connectionReuse - Error: ", t);
                    promise.reject("VcxException", t.getMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch(VcxException e) {
            promise.reject("VcxException", e.getMessage());
        }
    }

    @ReactMethod
    public void connectionGetState(int connectionHandle, Promise promise) {
        try {
            ConnectionApi.connectionGetState(connectionHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionGetState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionGetState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionUpdateState(int connectionHandle, Promise promise) {
        try {
            ConnectionApi.vcxConnectionUpdateState(connectionHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxConnectionUpdateState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionGetState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionUpdateStateWithMessage(int connectionHandle, String message, Promise promise) {
        try {
            ConnectionApi.vcxConnectionUpdateStateWithMessage(connectionHandle, message).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxConnectionUpdateStateWithMessage - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxConnectionUpdateStateWithMessage - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionSendAnswer(int connectionHandle, String question, String answer, Promise promise) {
        Log.d(TAG, "connectionSendAnswer() called");
        try {
            ConnectionApi.connectionSendAnswer(connectionHandle, question, answer).whenComplete((result, e) -> {
                if (e != null) {
                    Log.e(TAG, "connectionSendAnswer", e);
                    promise.reject("VcxException", e.getMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            promise.reject("VcxException", e.getMessage());
        }
    }

    @ReactMethod
    public void connectionSendInviteAction(int connectionHandle, String message, Promise promise) {
        try {
            ConnectionApi.connectionSendInviteAction(connectionHandle, message).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionSendInviteAction - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionSendInviteAction - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void createOutOfBandConnection(String sourceId, String goalCode, String goal, boolean handshake, String requestAttach, Promise promise) {
        Log.d(TAG, "connectionCreateOutofband()");
        try {
            ConnectionApi.vcxConnectionCreateOutofband(sourceId, goalCode, goal, handshake, requestAttach).whenComplete((result, e) -> {
                if (e != null) {
                    VcxException ex = (VcxException) e;
                    ex.printStackTrace();
                    Log.e(TAG, "connectionCreateOutofband - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionCreateOutofband - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void getConnectionInvite(int connectionHandle, Promise promise) {
        Log.d(TAG, "connectionInviteDetails()");
        try {
            ConnectionApi.connectionInviteDetails(connectionHandle, 0).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionInviteDetails - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionInviteDetails - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void connectionSendPing(int connectionHandle, String comment, Promise promise) {
        try {
            ConnectionApi.connectionSendPing(connectionHandle, comment).whenComplete((result, e) -> {
                if (e != null) {
                    Log.e(TAG, "connectionSendPing", e);
                    promise.reject("VcxException", e.getMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            promise.reject("VcxException", e.getMessage());
        }
    }

    @ReactMethod
    public void connectionInfo(int connectionHandle, Promise promise) {
        try {
            ConnectionApi.connectionInfo(connectionHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxConnectionInfo - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                Log.e(TAG, ">>>><<<< got result back");
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxConnectionInfo - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());

        }
    }

    @ReactMethod
    public void connectionGetProblemReport(int connectionHandle, Promise promise) {
        try {
            ConnectionApi.connectionGetProblemReport(connectionHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "vcxConnectionGetProblemReport - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                Log.e(TAG, ">>>><<<< got result back");
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxConnectionGetProblemReport - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());

        }
    }

    @ReactMethod
    public void getRedirectDetails(int connectionHandle, Promise promise) {
        Log.d(TAG, "getRedirectDetails()");

        try {
            ConnectionApi.vcxConnectionGetRedirectDetails(connectionHandle).exceptionally((e) -> {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "vcxConnectionGetRedirectDetails - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "getRedirectDetails - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    /*
     * Credential API
     */
    @ReactMethod
    public void credentialCreateWithOffer(String sourceId, String credOffer, Promise promise) {
        try {
            CredentialApi.credentialCreateWithOffer(sourceId, credOffer).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "credentialCreateWithOffer - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                Log.e(TAG, ">>>><<<< got result back");
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialCreateWithOffer - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }
    @ReactMethod
    public void serializeClaimOffer(int credentialHandle, Promise promise) {
        // it would return error code, json string of credential inside callback

        try {
            CredentialApi.credentialSerialize(credentialHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "credentialSerialize - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialSerialize - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }

    }

    @ReactMethod
    public void deserializeClaimOffer(String serializedCredential, Promise promise) {
        // it would return an error code and an integer credential handle in callback

        try {
            CredentialApi.credentialDeserialize(serializedCredential).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "credentialDeserialize - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialDeserialize - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void sendClaimRequest(int credentialHandle, int connectionHandle, int paymentHandle, Promise promise) {
        // it would return an error code in callback
        // we resolve promise with an empty string after success
        // or reject promise with error code

        try {
            CredentialApi.credentialSendRequest(credentialHandle, connectionHandle, paymentHandle).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "credentialSendRequest - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialSendRequest - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }
    @ReactMethod
    public void updateClaimOfferState(int credentialHandle, Promise promise) {
        try {
            CredentialApi.credentialUpdateState(credentialHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "credentialUpdateState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialUpdateState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void updateClaimOfferStateWithMessage(int credentialHandle, String message, Promise promise) {
        try {
            CredentialApi.credentialUpdateStateWithMessage(credentialHandle, message).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "updateClaimOfferStateWithMessage - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "updateClaimOfferStateWithMessage - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void getClaimOfferState(int credentialHandle, Promise promise) {
        try {
            CredentialApi.credentialGetState(credentialHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "credentialGetState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialGetState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void getClaimVcx(int credentialHandle, Promise promise) {
        try {
            CredentialApi.getCredential(credentialHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "getClaimVcx - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "getClaimVcx - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void deleteCredential(int credentialHandle, Promise promise) {
        try {
            CredentialApi.deleteCredential(credentialHandle).whenComplete((result, t) -> {
                if (t != null) {
                    Log.e(TAG, "deleteCredential: ", t);
                    promise.reject("FutureException", t.getMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            promise.reject("VCXException", e.getMessage());
        }
    }

    @ReactMethod
    public void credentialReject(int credentialHandle, int connectionHandle, String comment, Promise promise) {
        Log.d(TAG, "credentialReject()");
        try {
            CredentialApi.credentialReject(credentialHandle, connectionHandle, comment).whenComplete((result, e) -> {
                if (e != null) {
                    VcxException ex = (VcxException) e;
                    ex.printStackTrace();
                    Log.e(TAG, "credentialReject - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialReject - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void credentialGetPresentationProposal(int credentialHandle, Promise promise) {
        Log.d(TAG, "credentialGetPresentationProposal()");
        try {
            CredentialApi.credentialGetPresentationProposal(credentialHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "credentialGetPresentationProposal - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));

        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialGetPresentationProposal - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void credentialGetProblemReport(int credentialHandle, Promise promise) {
        Log.d(TAG, "credentialGetProblemReport()");
        try {
            CredentialApi.credentialGetProblemReport(credentialHandle).exceptionally((t) ->
            {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "credentialGetProblemReport - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));

        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "credentialGetProblemReport - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void credentialGetOffers(int connectionHandle, Promise promise) {
        try {
            CredentialApi.credentialGetOffers(connectionHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "connectionHandle - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                Log.e(TAG, ">>>><<<< got result back");
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "connectionHandle - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    /*
     * Library API
     */
    @ReactMethod
    public void shutdownVcx(Boolean deleteWallet, Promise promise) {
        Log.d(TAG, "shutdownVcx()");
        try {
            VcxApi.vcxShutdown(deleteWallet);
            promise.resolve("");
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxShutdown - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void init(String config, Promise promise) {
        Log.d(TAG, "init()");
        // When we restore data, then we are not calling createOneTimeInfo
        // and hence ca-crt is not written within app directory
        // since the logic to write ca cert checks for file existence
        // we won't have to pay too much cost for calling this function inside init
        BridgeUtils.writeCACert(this.getReactApplicationContext());

        try {
            int retCode = VcxApi.initSovToken();
            if(retCode != 0) {
                promise.reject("Could not init sovtoken", String.valueOf(retCode));
            } else {
                VcxApi.vcxInitWithConfig(config).exceptionally((t) -> {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "vcxInitWithConfig - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                    return -1;
                }).thenAccept(result -> {
                    // Need to put this logic in every accept because that is how ugly Java's
                    // promise API is
                    // even if exceptionally is called, then also thenAccept block will be called
                    // we either need to switch to complete method and pass two callbacks as
                    // parameter
                    // till we change to that API, we have to live with this IF condition
                    // also reason to add this if condition is because we already rejected promise
                    // in
                    // exceptionally block, if we call promise.resolve now, then it `thenAccept`
                    // block
                    // would throw an exception that would not be caught here, because this is an
                    // async
                    // block and above try catch would not catch this exception
                    if (result != -1) {
                        promise.resolve(true);
                    }
                });
            }

        } catch (AlreadyInitializedException e) {
            // even if we get already initialized exception
            // then also we will resolve promise, because we don't care if vcx is already
            // initialized
            promise.resolve(true);
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxInitWithConfig - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void vcxInitPool(String config, Promise promise) {
        Log.d(TAG, "vcxInitPool() called");
        try {
            VcxApi.vcxInitPool(config).whenComplete((result, e) -> {
                if (e != null) {
                    VcxException ex = (VcxException) e;
                    ex.printStackTrace();
                    Log.e(TAG, "vcxInitPool - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxInitPool - Error: ", e);
        }
    }

    /*
     * DisclosedProof
     */
    @ReactMethod
    public void proofRetrieveCredentials(int proofHandle, Promise promise) {
        try {
            DisclosedProofApi.proofRetrieveCredentials(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofRetrieveCredentials - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofRetrieveCredentials - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofGenerate(int proofHandle, String selectedCredentials, String selfAttestedAttributes,
                              Promise promise) {
        try {
            DisclosedProofApi.proofGenerate(proofHandle, selectedCredentials, selfAttestedAttributes).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "proofGenerate - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofGenerate - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofSend(int proofHandle, int connectionHandle, Promise promise) {
        try {
            DisclosedProofApi.proofSend(proofHandle, connectionHandle).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "proofSend - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofSend - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofCreateWithRequest(String sourceId, String proofRequest, Promise promise) {
        Log.d(TAG, "proofCreateWithRequest()");

        try {
            DisclosedProofApi.proofCreateWithRequest(sourceId, proofRequest).exceptionally((t)-> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofCreateWithRequest - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofCreateWithRequest - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofSerialize(int proofHandle, Promise promise) {
        Log.d(TAG, "proofSerialize()");
        try {
            DisclosedProofApi.proofSerialize(proofHandle).exceptionally((e) -> {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "proofSerialize - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofSerialize - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofDeserialize(String serializedProof, Promise promise) {
        Log.d(TAG, "proofDeserialize()");

        try {
            DisclosedProofApi.proofDeserialize(serializedProof).exceptionally((e)-> {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "proofDeserialize - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofDeserialize - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofReject(int proofHandle, int connectionHandle, Promise promise) {
        Log.d(TAG, "proofReject()");
        try {
            DisclosedProofApi.proofReject(proofHandle, connectionHandle).whenComplete((result, e) -> {
                if (e != null) {
                    VcxException ex = (VcxException) e;
                    ex.printStackTrace();
                    Log.e(TAG, "proofReject - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofReject - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofGetState(int proofHandle, Promise promise) {
       Log.d(TAG, "proofGetState()");
         try {
            DisclosedProofApi.proofGetState(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofGetState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofGetState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofUpdateState(int proofHandle, Promise promise) {
       Log.d(TAG, "proofUpdateState()");
         try {
            DisclosedProofApi.proofUpdateState(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofUpdateState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofUpdateState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofUpdateStateWithMessage(int proofHandle, String message, Promise promise) {
       Log.d(TAG, "proofUpdateStateWithMessage()");
         try {
            DisclosedProofApi.proofUpdateStateWithMessage(proofHandle, message).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofUpdateStateWithMessage - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofUpdateStateWithMessage - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofGetProblemReport(int proofHandle, Promise promise) {
        Log.d(TAG, "proofGetProblemReport()");
        try {
            DisclosedProofApi.proofGetProblemReport(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofGetProblemReport - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
          e.printStackTrace();
          Log.e(TAG, "proofGetProblemReport - Error: ", e);
          promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
       }
    }

    @ReactMethod
    public void proofDeclineRequest(int proofHandle, int connectionHandle, String reason, String proposal, Promise promise) {
        Log.d(TAG, "proofDeclineRequest()");
        try {
            DisclosedProofApi.proofDeclineRequest(proofHandle, connectionHandle, reason, proposal).whenComplete((result, t) -> {
                if (t != null) {
                    Log.e(TAG, "proofDeclineRequest - Error: ", t);
                    promise.reject("VcxException", t.getMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofDeclineRequest - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofGetRequests(int connectionHandle, Promise promise) {
        Log.d(TAG, "proofGetRequests()");

        try {
            DisclosedProofApi.proofGetRequests(connectionHandle).exceptionally((t)-> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofGetRequests - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofGetRequests - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

   /*
    * Proof Verifier API
    */
   @ReactMethod
   public void createProofVerifier(String sourceId, String requestedAttrs, String requestedPredicates, String revocationInterval, String name, Promise promise) {
       Log.d(TAG, "createProofVerifier()");
       try {
           ProofApi.proofCreate(sourceId, requestedAttrs, requestedPredicates, revocationInterval, name).exceptionally((t) -> {
               VcxException ex = (VcxException) t;
               ex.printStackTrace();
               Log.e(TAG, "verifierCreate - Error: ", ex);
               promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
               return -1;
           }).thenAccept(result -> {
               if (result != -1) {
                   BridgeUtils.resolveIfValid(promise, result);
               }
           });

       } catch (VcxException e) {
           e.printStackTrace();
           Log.e(TAG, "verifierCreate - Error: ", e);
           promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
       }
   }

   @ReactMethod
   public void createProofVerifierWithProposal(String sourceId, String presentationProposal, String name, Promise promise) {
       Log.d(TAG, "createProofVerifierWithProposal()");
       try {
           ProofApi.proofCreateWithProposal(sourceId, presentationProposal, name).exceptionally((t) -> {
               VcxException ex = (VcxException) t;
               ex.printStackTrace();
               Log.e(TAG, "createProofVerifierWithProposal - Error: ", ex);
               promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
               return -1;
           }).thenAccept(result -> {
               if (result != -1) {
                   BridgeUtils.resolveIfValid(promise, result);
               }
           });

       } catch (VcxException e) {
           e.printStackTrace();
           Log.e(TAG, "createProofVerifierWithProposal - Error: ", e);
           promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
       }
   }

    @ReactMethod
    public void proofVerifierUpdateState(int proofHandle, Promise promise) {
       Log.d(TAG, "proofVerifierUpdateState()");
         try {
            ProofApi.proofUpdateState(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierUpdateState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierUpdateState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierUpdateStateWithMessage(int proofHandle, String message, Promise promise) {
       Log.d(TAG, "proofVerifierUpdateStateWithMessage()");
         try {
            ProofApi.proofUpdateStateWithMessage(proofHandle, message).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierUpdateStateWithMessage - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierUpdateStateWithMessage - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierGetState(int proofHandle, Promise promise) {
       Log.d(TAG, "proofVerifierGetState()");
         try {
            ProofApi.proofGetState(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierGetState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierGetState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierSerialize(int proofHandle, Promise promise) {
       Log.d(TAG, "proofVerifierSerialize()");
         try {
            ProofApi.proofSerialize(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierSerialize - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                if (result != null) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierSerialize - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierDeserialize(String serialized, Promise promise) {
       Log.d(TAG, "proofVerifierDeserialize()");
         try {
            ProofApi.proofDeserialize(serialized).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierDeserialize - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierDeserialize - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierGetProofMessage(int proofHandle, Promise promise) {
        try {
            ProofApi.getProofMsg(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierGetProofMessage - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                if (result != null) {
                    GetProofResult typedResult = (GetProofResult) result;
                    WritableMap obj = Arguments.createMap();
                    obj.putInt("proofState", typedResult.getProof_state());
                    obj.putString("message", typedResult.getResponse_data());
                    BridgeUtils.resolveIfValid(promise, obj);
                }
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierGetProofMessage - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierGetPresentationRequest(int proofHandle, Promise promise) {
        try {
            ProofApi.proofGetRequestMsg(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierGetPresentationRequest - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierGetPresentationRequest - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierGetProblemReport(int proofHandle, Promise promise) {
        try {
            ProofApi.proofGetProblemReport(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofGetProblemReport - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofGetProblemReport - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierRequestPresentation(int proofHandle,
                                         int connectionHandle,
                                         String requestedAttrs,
                                         String requestedPredicates,
                                         String revocationInterval,
                                         String name,
                                         Promise promise
    ) {
        try {
            ProofApi.proofRequestPresentation(
                    proofHandle,
                    connectionHandle,
                    requestedAttrs,
                    requestedPredicates,
                    revocationInterval,
                    name
                    ).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierRequestPresentation - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
                }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierRequestPresentation - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void proofVerifierGetProofProposal(int proofHandle, Promise promise) {
        try {
            ProofApi.getProofProposal(proofHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "proofVerifierGetProofProposal - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierGetProofProposal - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    /*
     * Wallet API
     */
    @ReactMethod
    public void sendTokens(int paymentHandle, String tokens, String recipient, Promise promise) {
        try {
            TokenApi.sendTokens(paymentHandle, tokens, recipient).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "sendTokens - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "sendTokens - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void exportWallet(String exportPath, String encryptionKey, Promise promise) {
        Log.d(TAG, "exportWallet()");
        try {
            WalletApi.exportWallet(exportPath, encryptionKey).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "exportWallet - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if(result != -1){
                   BridgeUtils.resolveIfValid(promise, result);
                }
            });


        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxAgentProvisionAsync - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void decryptWalletFile(String config, Promise promise) {
        try {
            WalletApi.importWallet(config).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "importWallet - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return -1;
            }).thenAccept(result -> {
                if (result != -1) {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxAgentProvisionAsync - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void getTokenInfo(int paymentHandle, Promise promise) {
        try {
            TokenApi.getTokenInfo(paymentHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "getTokenInfo - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;

            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "getTokenInfo - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void createPaymentAddress(String seed, Promise promise) {
        try {
            TokenApi.createPaymentAddress(seed).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "createPaymentAddress - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "createPaymentAddress - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void addWalletRecord(String type, String key, String value, Promise promise) {
        try {
            WalletApi.addRecordWallet(type, key, value).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "addRecordWallet - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "addRecordWallet - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void getWalletRecord(String type, String key, Promise promise) {
        try {
            WalletApi.getRecordWallet(type, key, "").exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "getRecordWallet - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "getRecordWallet - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void deleteWalletRecord(String type, String key, Promise promise) {
        try {
            WalletApi.deleteRecordWallet(type, key).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "deleteRecordWallet - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "deleteRecordWallet - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void updateWalletRecord(String type, String key, String value, Promise promise) {
        try {
            WalletApi.updateRecordWallet(type, key, value).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "updateRecordWallet - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "updateRecordWallet - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void addWalletRecordTags(String type, String key, String tags, Promise promise) {
        try {
            WalletApi.addRecordTags(type, key, tags).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "addWalletRecordTags - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "addWalletRecordTags - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void updateWalletRecordTags(String type, String key, String tags, Promise promise) {
        try {
            WalletApi.updateRecordTags(type, key, tags).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "updateWalletRecordTags - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "updateWalletRecordTags - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void deleteWalletRecordTags(String type, String key, String tags, Promise promise) {
        try {
            WalletApi.deleteRecordTags(type, key, tags).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "deleteWalletRecordTags - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "deleteWalletRecordTags - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void openWalletSearch(String type, String query, String options, Promise promise) {
        Log.d(TAG, "openWalletSearch()");
        try {
            WalletApi.openSearch(type, query, options).whenComplete((result, e) -> {
                if (e != null) {
                    VcxException ex = (VcxException) e;
                    ex.printStackTrace();
                    Log.e(TAG, "openWalletSearch - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    BridgeUtils.resolveIfValid(promise, result);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "openWalletSearch - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void searchNextWalletRecords(int searchHandle, int count, Promise promise) {
        try {
            ProofApi.searchNextRecords(searchHandle, count).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "searchNextWalletRecords - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> BridgeUtils.resolveIfValid(promise, result));
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "proofVerifierGetProofMessage - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void closeWalletSearch(int searchHandle, Promise promise) {
        try {
            WalletApi.closeSearch(searchHandle).whenComplete((result, t) -> {
                if (t != null) {
                    VcxException ex = (VcxException) t;
                    ex.printStackTrace();
                    Log.e(TAG, "closeWalletSearch - Error: ", ex);
                    promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "closeWalletSearch - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void updateWalletBackupState(int walletBackupHandle, Promise promise) {
        try {
            WalletApi.updateWalletBackupState(walletBackupHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "updateWalletBackupState - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "updateWalletBackupState - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void serializeBackupWallet(int walletBackupHandle, Promise promise) {
        try {
            WalletApi.serializeBackupWallet(walletBackupHandle).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "serializeBackupWallet - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "serializeBackupWallet - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void deserializeBackupWallet(String message, Promise promise) {
        try {
            WalletApi.deserializeBackupWallet(message).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "deserializeBackupWallet - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "serializeBackupWallet - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    /*
     * Utils and secondary methods API
     */
    private static int getLogLevel(String levelName) {
        if("Error".equalsIgnoreCase(levelName)) {
            return 1;
        } else if("Warning".equalsIgnoreCase(levelName) || levelName.toLowerCase().contains("warn")) {
            return 2;
        } else if("Info".equalsIgnoreCase(levelName)) {
            return 3;
        } else if("Debug".equalsIgnoreCase(levelName)) {
            return 4;
        } else if("Trace".equalsIgnoreCase(levelName)) {
            return 5;
        } else {
            return 3;
        }
    }

    @ReactMethod
    public void encryptVcxLog(String logFilePath, String key, Promise promise) {
        try {
            RandomAccessFile logFile = new RandomAccessFile(logFilePath, "r");
            byte[] fileBytes = new byte[(int)logFile.length()];
            logFile.readFully(fileBytes);
            logFile.close();

            IndyApi.anonCrypt(key, fileBytes).exceptionally((t) -> {
                Log.e(TAG, "anonCrypt - Error: ", t);
                promise.reject("FutureException", "Error occurred while encrypting file: " + logFilePath + " :: " + t.getMessage());
                return null;
            }).thenAccept(result -> {
                try {
                    RandomAccessFile encLogFile = new RandomAccessFile(RNIndyStaticData.ENCRYPTED_LOG_FILE_PATH, "rw");
                    encLogFile.write(result, 0, result.length);
                    encLogFile.close();
                    BridgeUtils.resolveIfValid(promise, RNIndyStaticData.ENCRYPTED_LOG_FILE_PATH);
                } catch(IOException ex) {
                    promise.reject("encryptVcxLog Exception", ex.getMessage());
                    ex.printStackTrace();
                }
            });
        } catch (VcxException | IOException e) {
            promise.reject("encryptVcxLog - Error", e.getMessage());
            e.printStackTrace();
        }
    }

    @ReactMethod
    public  void writeToVcxLog(String loggerName, String logLevel, String message, String logFilePath, Promise promise) {
        VcxApi.logMessage(loggerName, getLogLevel(logLevel), message);
        promise.resolve(0);
    }

    @ReactMethod
    public void setVcxLogger(String logLevel, String uniqueIdentifier, int MAX_ALLOWED_FILE_BYTES, Promise promise) {

        ContextWrapper cw = new ContextWrapper(reactContext);
        RNIndyStaticData.MAX_ALLOWED_FILE_BYTES = MAX_ALLOWED_FILE_BYTES;
        RNIndyStaticData.LOG_FILE_PATH = cw.getFilesDir().getAbsolutePath() +
                "/connectme.rotating." + uniqueIdentifier + ".log";
        RNIndyStaticData.ENCRYPTED_LOG_FILE_PATH = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS).getAbsolutePath() +
                "/connectme.rotating." + uniqueIdentifier + ".log.enc";
        //get the documents directory:
        Log.d(TAG, "Setting vcx logger to: " + RNIndyStaticData.LOG_FILE_PATH);

        if (Environment.MEDIA_MOUNTED.equals(Environment.getExternalStorageState())) {
            RNIndyStaticData.initLoggerFile(cw);
        }
        promise.resolve(RNIndyStaticData.LOG_FILE_PATH);

    }

    @ReactMethod
    public void createWalletKey(int lengthOfKey, Promise promise) {
        try {
            SecureRandom random = new SecureRandom();
            byte bytes[] = new byte[lengthOfKey];
            random.nextBytes(bytes);
            promise.resolve(Base64.encodeToString(bytes, Base64.NO_WRAP));
        } catch(Exception e) {
            e.printStackTrace();
            Log.e(TAG, "createWalletKey - Error: ", e);
            promise.reject("Exception", e.getMessage());
        }
    }

    @ReactMethod
    public void getLedgerFees(Promise promise) {
        Log.d(TAG, "getLedgerFees()");

        try {
            UtilsApi.getLedgerFees().exceptionally((e)-> {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "getLedgerFees - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch(VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "getLedgerFees - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void getTxnAuthorAgreement(Promise promise) {
        try {
            // IndyApi.getTxnAuthorAgreement(submitterDid, data).exceptionally((e) -> {
            UtilsApi.getLedgerAuthorAgreement().exceptionally((e) -> {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "getLedgerAuthorAgreement - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "getLedgerAuthorAgreement - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void getAcceptanceMechanisms(String submitterDid, int timestamp, String version, Promise promise) {
        Long longtimestamp= new Long(timestamp);
        try {
            IndyApi.getAcceptanceMechanisms(submitterDid, longtimestamp, version).exceptionally((e) -> {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "getAcceptanceMechanisms - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "getAcceptanceMechanisms - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void setActiveTxnAuthorAgreementMeta(String text, String version, String taaDigest, String mechanism, int timestamp, Promise promise) {
         Long longtimestamp= new Long(timestamp);
        try {
            UtilsApi.setActiveTxnAuthorAgreementMeta(text, version, taaDigest, mechanism, longtimestamp);
            promise.resolve("");
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "setActiveTxnAuthorAgreementMeta - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void appendTxnAuthorAgreement(String requestJson, String text, String version, String taaDigest, String mechanism, int timestamp, Promise promise) {
        Long longtimestamp= new Long(timestamp);
        try {
            IndyApi.appendTxnAuthorAgreement(requestJson, text, version, taaDigest, mechanism, longtimestamp).exceptionally((e) -> {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "appendTxnAuthorAgreement - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "appendTxnAuthorAgreement - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void fetchPublicEntities(Promise promise) {
        Log.d(TAG, "fetchPublicEntities() called");
        try {
            UtilsApi.vcxFetchPublicEntities().whenComplete((result, e) -> {
                if (e != null) {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "vcxFetchPublicEntities - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                } else {
                    promise.resolve(0);
                }
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxFetchPublicEntities - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void vcxHealthCheck(Promise promise) {
        try {
            UtilsApi.vcxHealthCheck().exceptionally((e) -> {
                VcxException ex = (VcxException) e;
                ex.printStackTrace();
                Log.e(TAG, "vcxHealthCheck - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "vcxHealthCheck - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());

        }
    }

    @ReactMethod
    public void extractAttachedMessage(String message, Promise promise) {
        try {
            UtilsApi.vcxExtractAttachedMessage(message).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "extractAttachedMessage - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "extractAttachedMessage - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }

    @ReactMethod
    public void resolveMessageByUrl(String url, Promise promise) {
        try {
            UtilsApi.vcxResolveMessageByUrl(url).exceptionally((t) -> {
                VcxException ex = (VcxException) t;
                ex.printStackTrace();
                Log.e(TAG, "resolveMessageByUrl - Error: ", ex);
                promise.reject(String.valueOf(ex.getSdkErrorCode()), ex.getSdkMessage());
                return null;
            }).thenAccept(result -> {
                BridgeUtils.resolveIfValid(promise, result);
            });
        } catch (VcxException e) {
            e.printStackTrace();
            Log.e(TAG, "resolveMessageByUrl - Error: ", e);
            promise.reject(String.valueOf(e.getSdkErrorCode()), e.getSdkMessage());
        }
    }
}
