FROM ubuntu:bionic

ENV LIBVCX_VERSION 0.11.2

RUN apt-get update -y && \
    apt-get install -y \
    ca-certificates \
    curl \
    software-properties-common

# install certificates for sovrin
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys CE7709D068DB5E88
RUN add-apt-repository "deb https://repo.sovrin.org/sdk/deb bionic stable"

# install certificates for evernym
RUN mkdir -p /usr/local/share/ca-certificates
RUN curl -k https://repo.corp.evernym.com/ca.crt | tee /usr/local/share/ca-certificates/Evernym_Root_CA.crt
RUN update-ca-certificates

# install libraries and libindy
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    libsodium-dev \
    libtool \
    pkg-config \
    build-essential \
    autoconf \
    automake \
    uuid-dev \
    wget

RUN curl https://gitlab.com/evernym/verity/vdr-tools/-/package_files/41403643/download --output libvdrtools.deb
RUN dpkg --force-all -i libvdrtools.deb

# install libvcx
RUN curl https://gitlab.com/evernym/mobile/ev-vcx/-/package_files/40564831/download --output libvcx.deb
RUN dpkg --force-all -i libvcx.deb

# install vcx python wrapper
RUN curl https://gitlab.com/evernym/mobile/ev-vcx/-/package_files/40564782/download --output vcx-wrapper.tar.gz
RUN pip3 install vcx-wrapper.tar.gz

WORKDIR /demo