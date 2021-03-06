## React Native SDK

This is a React Native wrapper for VCX library.

VCX is the open-source library on top of Libindy which fully implements the credentials exchange.

**Note**: The SDK is currently in **EXPERIMENTAL** state, so the API functions can be changed.

### Installation

Added `@evernym/react-native-sdk` package as dependency into your `package.json`:
```javascript
  "dependencies": {
    ...
    "@evernym/react-native-sdk": "version"
  },
```

### Linking

##### Android

1. Update the minimum supported SDK version in your `android/build.gradle` file to be `23`:
    ```groovy
    buildscript {
        ext {
            ...
            minSdkVersion = 23
            ...
        }
        ...
    ```

1. Add a new source repository in your `android/build.gradle` file:
    ```groovy
    allprojects {
        repositories {
            ...
            maven {
                url 'https://evernym.mycloudrepo.io/public/repositories/libvcx-android'
            }
            maven {
                url 'https://evernym.mycloudrepo.io/public/repositories/evernym'
            }
        }
    }

1. Setup packaging options in your `android/app/build.gradle` file:
   ```groovy
   android {
       ...
       packagingOptions{
           pickFirst 'lib/armeabi-v7a/libc++_shared.so'
           pickFirst 'lib/arm64-v8a/libc++_shared.so'
           pickFirst 'lib/x86_64/libc++_shared.so'
           pickFirst 'lib/x86/libc++_shared.so'
       }
       ...
   }
   ```

1. Update your `MainActivity` by adding the following code (it's needed to configure your app storage):
    ```
    import android.content.ContextWrapper;
    import android.system.Os;
    ```
    ```
    @Override
    protected void onStart() {
        super.onStart();
        try {
            ContextWrapper c = new ContextWrapper(this);
            Os.setenv("EXTERNAL_STORAGE", c.getFilesDir().toString(), true);
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
   ```

1. Enable backups in your `AndroidManifest.xml` file:
    ```
    android:allowBackup="true"
    ```

##### iOS

1. Add a next source to the top of your `Podfile`:

    ```ruby
    ....
    source 'git@gitlab.com:evernym/mobile/mobile-sdk.git'
    ```

1. Add VCX dependency into your Podfile inside target <ProjectName>:
    * release build for devices only (`arm64`):
        ```ruby
        pod 'vcx', 0.0.233
        ```
    * debug build for simulators only (`x86_64`)
        ```ruby
        pod 'vcx', 0.0.234
        ```

1. Run `pod install`

### Modules:
* Library - functions related to library initialization.
* Agent - functions related to operations with your Cloud Agent.
* Connection - functions related to establishing a connection with a remote side.
* Credential - functions related to obtaining credentials. 
* DisclosedProof - functions related to proving credential data.
* Verifier - functions related to credential data verification.
* Utils - different helper functions.
* Logger - functions related to library logging.
* DocumentVerification - functions related to document verification

### Usage

* Create an agent with received token: 
    ```javascript
    import { Agent } from '@evernym/react-native-sdk'
    
    const config: string = await Agent.provisionWithToken({
       agencyConfig,
       token,
    })
    ```

* Initialize library with a config:
    ```javascript
    import { Library } from '@evernym/react-native-sdk'
    
    await Library.init({
        config,
      })
    ```

