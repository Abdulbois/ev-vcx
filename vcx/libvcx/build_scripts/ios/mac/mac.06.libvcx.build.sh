#!/bin/sh

set -e
source ./shared.functions.sh

START_DIR=$PWD
WORK_DIR=$START_DIR/../../../../../.macosbuild
mkdir -p $WORK_DIR
WORK_DIR=$(abspath "$WORK_DIR")

INDY_SDK=$WORK_DIR/vcx-indy-sdk
VCX_SDK=$START_DIR/../../../../..
VCX_SDK=$(abspath "$VCX_SDK")

export IOS_TARGETS=$3
source ./mac.05.libvcx.env.sh
cd ../../..
DEBUG_SYMBOLS="debuginfo"

if [ ! -z "$1" ]; then
    DEBUG_SYMBOLS=$1
fi

if [ "$DEBUG_SYMBOLS" = "nodebug" ]; then
    sed -i .bak 's/debug = true/debug = false/' Cargo.toml
fi

if [ -z "${IOS_TARGETS}" ]; then
    echo "please provide the targets e.g aarch64-apple-ios,armv7-apple-ios,i386-apple-ios,x86_64-apple-ios"
    exit 1
fi

CLEAN_BUILD="cleanbuild"
if [ ! -z "$2" ]; then
    CLEAN_BUILD=$2
fi

if [ "${CLEAN_BUILD}" = "cleanbuild" ]; then
    echo "cleanbuild"
    cargo clean
    rm -rf ${BUILD_CACHE}/target
    rm -rf ${BUILD_CACHE}/arch_libs
fi

git log -1 > $WORK_DIR/evernym.vcx-sdk.git.commit.log

export OPENSSL_LIB_DIR_DARWIN=${OPENSSL_LIB_DIR}

bkpIFS="$IFS"
IFS=',()][' read -r -a targets <<<"${IOS_TARGETS}"
echo "Building targets: ${targets[@]}"    ##Or printf "%s\n" ${array[@]}
IFS="$bkpIFS"

to_combine=""
for target in ${targets[*]}
do
    if [ "${target}" = "aarch64-apple-ios" ]; then
        target_arch="arm64"
    elif [ "${target}" = "x86_64-apple-ios" ]; then
        target_arch="x86_64"
    fi

    libtool="/usr/bin/libtool"
    libvdrtools_dir="${BUILD_CACHE}/libvdrtools/${LIBINDY_VERSION}"

    if [ -e ${libvdrtools_dir}/${target_arch}/libvdrtools.a ]; then
        echo "${target_arch} libvdrtools architecture already extracted"
    else
        mkdir -p ${libvdrtools_dir}/${target_arch}
        lipo -extract $target_arch ${libvdrtools_dir}/libvdrtools.a -o ${libvdrtools_dir}/${target_arch}/libvdrtools.a
        ${libtool} -static ${libvdrtools_dir}/${target_arch}/libvdrtools.a -o ${libvdrtools_dir}/${target_arch}/libvdrtools_libtool.a
        mv ${libvdrtools_dir}/${target_arch}/libvdrtools_libtool.a ${libvdrtools_dir}/${target_arch}/libvdrtools.a
    fi


    export IOS_SODIUM_LIB=$WORK_DIR/libzmq-ios/libsodium-ios/dist/ios/lib/${target_arch}
    export IOS_ZMQ_LIB=$WORK_DIR/libzmq-ios/dist/ios/lib/${target_arch}
    export LIBINDY_DIR=${libvdrtools_dir}/${target_arch}

    export OPENSSL_LIB_DIR=${WORK_DIR}/OpenSSL-for-iPhone/lib/
    export OPENSSL_INCLUDE_DIR=${WORK_DIR}/OpenSSL-for-iPhone/include/
    export OPENSSL_DIR=${WORK_DIR}/OpenSSL-for-iPhone/bin

    cargo build --target "${target}" --release --no-default-features --features "ci"
    to_combine="${to_combine} ./target/${target}/release/libvcx.a"
done
mkdir -p ./target/universal/release
lipo -create $to_combine -o ./target/universal/release/libvcx.a

export OPENSSL_LIB_DIR=$OPENSSL_LIB_DIR_DARWIN
