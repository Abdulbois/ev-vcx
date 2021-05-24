## React Native VCX wrapper

This is a React Native wrapper for VCX library.

VCX is the open-source library on top of Libindy which fully implements the credentials exchange.

**Note**: The wrapper is currently in **EXPERIMENTAL** state, so the API functions can be changed.

### Installation

Added `react-native-vcx-wrapper` package as dependency into your `package.json`:
```javascript
  "dependencies": {
    ...
    "react-native-vcx-wrapper": "link to the tgz archive"
  },
```

### Linking

##### Android

There are no steps need to do.

##### iOS

1. Add a next source to the top of your `Podfile`:

    ```ruby
    ....
    source 'git@github.com:evernym/mobile-sdk.git'
    ```

1. Add VCX dependency into your `Podfile` inside `target <ProjectName>`: 

    ```ruby
    pod 'vcx', 0.0.205
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

### Usage

* Create an agent with received token: 
    ```javascript
    import { Agent } from 'react-native-vcx-wrapper'
    
    const config: string = await Agent.provisionWithToken({
       agencyConfig,
       token,
    })
    ```

* Initialize library with a config:
    ```javascript
    import { Library } from 'react-native-vcx-wrapper'
    
    await Library.init({
        config,
      })
    ```

