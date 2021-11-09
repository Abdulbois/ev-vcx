# VCX NodeJS Wrapper

This is a NodeJS wrapper for VCX library.
VCX is the open-source library on top of VDR-Tools which fully implements the credentials exchange.

**Note**: This library is currently in experimental state.

## Contribution Guide

Make sure you have these packages installed:

* StandardJS
* Typescript
* TSLint


Also this has a dependency on:
* libvcx debian
Because it creates a symlink (/usr/lib/libvcx.so)

Run this commands before submitting your PR:

```
npm run lint
```

## Documentation:
 Run these commands:
```
npm install
npm ci
npm run doc-gen
```
* A directory will be created locally `./docs` which contains an `index.html` file which can be used to navigate the
generated documents.

### Pre-requirements
##### Libraries
Before you'll be able to run demo, you need to make sure you've compiled
- [`vdr-tools`](https://gitlab.com/evernym/verity/vdr-tools)
- [`libvcx`](https://gitlab.com/evernym/mobile/ev-vcx)

Library binaries must be located `/usr/local/lib` on OSX, `/usr/lib` on Linux.

#### Indy pool
You'll also have to run pool of Indy nodes on your machine. You can achieve by simply running a docker container
which encapsulates multiple interconnected Indy nodes.
[Instructions here](https://github.com/hyperledger/indy-sdk#how-to-start-local-nodes-pool-with-docker).

### Steps to run demo
- Install NodeJS dependencies
```
npm install
```

- Compile LibVCX Wrapper
```
npm run compile
```
- Run Faber agent, representing an institution
```
npm run demo:faber
```
- Give it a few seconds, then run Alice's agent which will connect with Faber's agent
```
npm run demo:alice
```
