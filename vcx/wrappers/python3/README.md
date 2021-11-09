# VCX for Python

This is a Python wrapper for VCX library. 
VCX is the open-source library on top of Libindy which fully implements the credentials exchange.

**Note**: This library is currently in experimental state.

This Python wrapper currently requires python 3.6.

### How to install
    pip install python3-wrapper-vcx
    
**Note** that before you can use python wrapper you must install  c-callable SDK and Vcx.  
* See the section "Installing the VDR-Tools" in the [VDR Tools documentation](https://gitlab.com/evernym/mobile/ev-vcx) 
* See the section "Installing VCX" in the [VCX documentation](../../../README.md) 

## Documentation:
 Run this command:
```
python3 generateDocs.py
```
* A directory will be created locally `./docs` which contains subdirectories 'vcx' and within that 'api'.  Html files are generated and put here that give details on each api function.

### Example use
For the main workflow example check [Python demo](./demo/README.md).

#### Logging
The Python wrapper uses default Python logging module. So, to enable logs you need just to configure its usual way. 
Note: there is an additional log level=0 that is equal to `trace` level.
