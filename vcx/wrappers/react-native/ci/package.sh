#!/bin/bash
set -e
export PATH=${PATH}:$(pwd)/vcx/ci/scripts

VCX_VERSION=$(toml_utils.py vcx/libvcx/Cargo.toml)
VCX_REVISION=$(git rev-parse HEAD | cut -c 1-7)

OUTPUTDIR=output
DIR=vcx/wrappers/react-native
CURDIR=$(pwd)

cd $DIR

sed -riE "s|com.evernym:vcx:.*@aar|com.evernym:vcx:${VCX_VERSION}-${VCX_REVISION}@aar|" android/build.gradle

npm i
npm run build
npm pack

rename \s/evernym-react-native-sdk-/evernym-react-native-sdk_/ *.tgz

cd $CURDIR
cp $DIR/evernym-react-native-sdk*.tgz $OUTPUTDIR
