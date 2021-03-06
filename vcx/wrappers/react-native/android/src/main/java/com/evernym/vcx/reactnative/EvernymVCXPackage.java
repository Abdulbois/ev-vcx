package com.evernym.vcx.reactnative;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import com.facebook.react.ReactPackage;
import com.facebook.react.bridge.NativeModule;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.uimanager.ViewManager;

import com.evernym.vcx.reactnative.rnindy.RNIndyModule;
import com.evernym.vcx.reactnative.mids.MIDSDocumentVerification;

public class EvernymVCXPackage implements ReactPackage {
    @Override
    public List<NativeModule> createNativeModules(ReactApplicationContext reactContext) {
        return Arrays.<NativeModule>asList(new NativeModule[]{
          new RNIndyModule(reactContext),
          new MIDSDocumentVerification(reactContext),
        });
    }

    @Override
    public List<ViewManager> createViewManagers(ReactApplicationContext reactContext) {
        return Collections.emptyList();
    }
}
