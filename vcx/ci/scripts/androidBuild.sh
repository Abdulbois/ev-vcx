#!/bin/bash

set -e

retrieve_prebuilt_binaries() {
    ANDROID_BUILD_FOLDER=${PWD}
    pushd ${ANDROID_BUILD_FOLDER}
        echo -e "${GREEN}Downloading openssl for $1 ${RESET}"
        curl -sSLO https://repo.sovrin.org/android/libindy/deps-libc++/openssl/openssl_$1.zip
        unzip -o -qq openssl_$1.zip
        export OPENSSL_DIR=${ANDROID_BUILD_FOLDER}/openssl_$1
        echo -e "${GREEN}Done!${RESET}"

        echo -e "${GREEN}Downloading sodium for $1 ${RESET}"
        curl -sSLO https://repo.sovrin.org/android/libindy/deps-libc++/sodium/libsodium_$1.zip
        unzip -o -qq libsodium_$1.zip
        export SODIUM_DIR=${ANDROID_BUILD_FOLDER}/libsodium_$1
        echo -e "${GREEN}Done!${RESET}"

        echo -e "${GREEN}Downloading zmq for $1 ${RESET}"
        curl -sSLO https://repo.sovrin.org/android/libindy/deps-libc++/zmq/libzmq_$1.zip
        unzip -o -qq libzmq_$1.zip
        export LIBZMQ_DIR=${ANDROID_BUILD_FOLDER}/libzmq_$1
        echo -e "${GREEN}Done!${RESET}"

        rm openssl_$1.zip
        rm libsodium_$1.zip
        rm libzmq_$1.zip
    popd
}

generate_flags(){
    if [ -z $1 ]; then
        echo "please provide the arch e.g arm, arm64, armv7, x86, or x86_64"
        exit 1
    fi
    if [ $1 == "arm" ]; then
        export ARCH="arm"
        export TRIPLET="arm-linux-androideabi"
        export PLATFORM="21"
    elif [ $1 == "arm64" ]; then
        export ARCH="arm64"
        export TRIPLET="aarch64-linux-android"
        export PLATFORM="21"
    elif [ $1 == "armv7" ]; then
        export ARCH="armv7"
        export TRIPLET="armv7-linux-androideabi"
        export PLATFORM="21"
    elif [ $1 == "x86" ]; then
        export ARCH="x86"
        export TRIPLET="i686-linux-android"
        export PLATFORM="21"
    elif [ $1 == "x86_64" ]; then
        export ARCH="x86_64"
        export TRIPLET="x86_64-linux-android"
        export PLATFORM="21"
    else
        echo "please provide the arch e.g arm, arm64, armv7, x86, or x86_64"
        exit 1
    fi
}

get_libindy() {
    set -xv
    if [ -z ${LIBINDY_DIR} ]; then

        if [ ! -d "libindy_${ARCH}" ]; then

            if [ $1 == "arm" ]; then
                wget https://gitlab.com/evernym/verity/vdr-tools/-/package_files/23271133/download
            elif [ $1 == "arm64" ]; then
                wget https://gitlab.com/evernym/verity/vdr-tools/-/package_files/23271130/download
            elif [ $1 == "armv7" ]; then
                wget https://gitlab.com/evernym/verity/vdr-tools/-/package_files/23271149/download
            elif [ $1 == "x86" ]; then
                wget https://gitlab.com/evernym/verity/vdr-tools/-/package_files/23271127/download
            elif [ $1 == "x86_64" ]; then
                wget https://gitlab.com/evernym/verity/vdr-tools/-/package_files/23271139/download
            else
                echo "please provide the arch e.g arm, arm64, armv7, x86, or x86_64"
                exit 1
            fi

            unzip download

        fi

        export LIBINDY_DIR="${PWD}/libindy_${ARCH}"
    fi

}

build_vcx() {
    # For Jenkins
    LIBVCX_PATH=${VCX_BASE}/libvcx/build_scripts/android/vcx/
    PREBUILT_BIN=../../../../../runtime_android_build
    # For Docker when vcx is in home dir
    #PREBUILT_BIN=../../../../ci/scripts/runtime_android_build
    # PREBUILT_BIN=$(realpath ${VCX_BASE}/ci/scripts/runtime_android_build)

    if [ ! -d ${LIBINDY_DIR} ]; then
        echo "missing libindy_${ARCH} directory. Cannot proceed without it."
        exit 1
    fi

    pushd ${LIBVCX_PATH}
    mkdir -p toolchains/
    ./build.nondocker.sh ${ARCH} ${PLATFORM} ${TRIPLET} ${OPENSSL_DIR} ${SODIUM_DIR} ${LIBZMQ_DIR} ${LIBINDY_DIR}
    popd
    rm -rf libvcx_${ARCH}
    mv ${LIBVCX_PATH}libvcx_${ARCH} .

}

setup() {
    echo "Working Directory: ${PWD}"
    set -e
    export ARCH=$1

    export PATH=$PATH:/opt/gradle/gradle-3.4.1/bin
    export PATH=${PATH}:$ANDROID_HOME/platform-tools:$ANDROID_HOME/tools:$ANDROID_HOME/build-tools/25.0.2/
    export PATH=${HOME}/.cargo/bin:${PATH}
    export VCX_BASE=../vcx
    # For docker
    # export VCX_BASE=${HOME}/vcx

    source /etc/profile
	if [ ! -d runtime_android_build ]; then
        mkdir runtime_android_build
    fi
    cd runtime_android_build
	retrieve_prebuilt_binaries ${ARCH}
	generate_flags $1
    if [ ! -d "toolchains" ]; then
        mkdir toolchains
    fi

    ANDROID_JNI_LIB=${VCX_BASE}/wrappers/java/vcx/src/main/jniLibs
}

setup $@
get_libindy $1
build_vcx
