#!/bin/sh

source ./shared.functions.sh

START_DIR=$PWD
WORK_DIR=$START_DIR/../../../../../.macosbuild
mkdir -p $WORK_DIR
WORK_DIR=$(abspath "$WORK_DIR")

#LIBINDY_PATH=$PWD/vcx-indy-sdk/libindy/target/debug/libindy.dylib
#LIBINDY_PATH=$WORK_DIR/vcx-indy-sdk/libindy/target/universal/debug/libindy.a
LIBINDY_PATH=$WORK_DIR/libvdrtools/libvdrtools.a
LIBINDY_HEADER_PATH=$WORK_DIR/libvdrtools
VCXHEADER_PATH=$(abspath "$START_DIR/../../../include")/vcx.h

ls -al $LIBINDY_PATH
#ln -sf $LIBINDY_PATH /usr/local/lib/libindy.dylib
ln -sf $LIBINDY_PATH /usr/local/lib/libvdrtools.a
otool -L /usr/local/lib/libvdrtools.a
lipo -info /usr/local/lib/libvdrtools.a

ln -sf $VCXHEADER_PATH /usr/local/include/vcx.h

for h in `ls ${LIBINDY_HEADER_PATH}/*.h`
do
    ln -sf $LIBINDY_HEADER_PATH/$h /usr/local/include/$h
done
