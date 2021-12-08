# Evernym VCX

Ev-VCX provides a high level API to use [VDR Tools](https://gitlab.com/evernym/verity/vdr-tools) for credential exchange, packaged as a Rust library that can be accessed through wrappers in various languages. Ev-VCX simplifies using VDR Tools for Aries compatible credential exchange.

## Evolution

Evernym LibVCX was originally built as a high level API for LibIndy and deployed as the core of Verity 1, Evernym’s original enterprise product for issuing and verifying credentials. It was contributed to Hyperledger as part of [the Indy SDK](https://github.com/hyperledger/indy-sdk). Over time, the Hyperledger community decided to split LibVCX from the Indy SDK [Aries VCX](https://github.com/hyperledger/aries-vcx) where it continues to be maintained for their use cases. Evernym has resumed maintaining Ev-VCX as our own library in order to focus on mobile deployments which leverage Evernym's agency infrastructure. Ev-VCX is now principally used by the [Evernym Mobile SDK](https://gitlab.com/evernym/mobile/mobile-sdk).

## Roadmap

* Remove Indy SDK components from the repository and reorganize the code.
* Pipelines that use VDR Tools as a dependency.
* Removal of legacy components: unused wrappers and the agency.
* Support for multiple ledgers, including Indy ledgers like Sovrin and IDUnion.
* Support for additional signature types such as BBS+.

## Prerequisites
* VCX requires access to some Cloud Agent for full work. [Here](https://gitlab.com/evernym/mobile/mobile-sdk/-/blob/main/docs/2.Initialization.md#2-initializing-the-wallet-and-cloud-agent) see instructions how to get Cloud Agent.

## How to build VCX from source

### Linux 
1) Install rust and rustup (https://www.rust-lang.org/install.html). 
2) Install VDR-Tools:
   ```
   add-apt-repository "deb https://repo.corp.evernym.com/deb evernym-agency-dev-ubuntu main"
   apt-get update && apt-get install -y libvdrtools=${VDRTOOLS_VER}-{bionic|xenial}
   ```
3) Clone this repo to your local machine. 
4) From the vcx/libvcx folder inside this local repository run the following commands to verify everything works: 
    ``` 
    $ cargo build 
    $ cargo test 
    ``` 
5) Currently developers are using intellij for IDE development (https://www.jetbrains.com/idea/download/) with the rust plugin (https://plugins.jetbrains.com/plugin/8182-rust). 

### Android
1) Install rust and rustup (https://www.rust-lang.org/install.html).
2) Clone this repo to your local machine.
4) Run `install_toolchains.sh`. You need to run this once to setup toolchains for android
5) Run `android.build.sh aarm64` to build libvcx for aarm64 architecture.(Other architerctures will follow soon)
6) Tests are not working on Android as of now.
 
## Wrappers documentation

The following wrappers are tested and complete.

* [Java](vcx/wrappers/java/README.md)
* [Python](vcx/wrappers/python3/README.md)
* [iOS](vcx/wrappers/ios/README.md)
* [NodeJS](vcx/wrappers/node/README.md)

## Library initialization
Libvcx library must be initialized with one of the functions:
* `vcx_init_with_config` -  initializes with <configuration> passed as JSON string. 
* `vcx_init` -  initializes with a path to the file containing <configuration>. 

Each library function will use this <configuration> data after the initialization. 
The list of options can be find [here](https://gitlab.com/evernym/mobile/mobile-sdk/-/blob/main/docs/Configuration.md)
An example of <configuration> file can be found [here](https://gitlab.com/evernym/mobile/mobile-sdk/-/blob/main/docs/3.Initialization.md#sdk-provisioning-config-sample-single-pool-ledger)

If the library works with an agency `vcx_provision_agent_with_token` function must be called before initialization to populate configuration and wallet for this agent.
Provisioning token must be received from your sponsor server. More information about Cloud Agent provisioning and Sponsor registration you can find [here](https://gitlab.com/evernym/mobile/mobile-sdk/-/blob/main/docs/3.Initialization.md#2-initializing-the-wallet-and-cloud-agent).

The result of this function is <configuration> JSON which can be extended and used for initialization.

To change <configuration> a user must call `vcx_shutdown` and then call initialization function again.

## Getting started guide
[The tutorial](docs/getting-started/getting-started.md) which introduces Libvcx and explains how the whole ecosystem works, and how the functions in the SDK can be used to construct rich clients.

### Example use
For the main workflow example check [demo](./vcx/wrappers/python3/demo).

## Actors
Libvcx provides APIs for acting as different actors.
The actor states, transitions and messages depend on [protocol_type](https://gitlab.com/evernym/mobile/mobile-sdk/-/blob/main/docs/Configuration.md#communication-protocol) is used in the configuration JSON.

* Connection:
    * Inviter
        * [API](./vcx/libvcx/src/api/connection.rs) 
        * [State diagram](docs/states/aries/connection-inviter.puml)
    * Invitee
        * [API](./vcx/libvcx/src/api/connection.rs) 
        * [State diagram](docs/states/aries/connection-invitee.puml)

* Credential Issuance:
    * Issuer
        * [API](./vcx/libvcx/src/api/issuer_credential.rs) 
        * [State diagram](docs/states/aries/issuer-credential.puml)
    * Holder
        * [API](./vcx/libvcx/src/api/credential.rs) 
        * [State diagram](docs/states/aries/credential.puml)

* Credential Presentation:
    * Verifier
        * [API](./vcx/libvcx/src/api/proof.rs) 
        * [State diagram](docs/states/aries/proof.puml)
      * Prover
        * [API](./vcx/libvcx/src/api/disclosed_proof.rs) 
        * [State diagram](docs/states/aries/disclosed-proof.puml) 

## How to migrate
The documents that provide necessary information for Libvcx migrations.
