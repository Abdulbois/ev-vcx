# Development
FROM ubuntu:18.04

ENV DEBIAN_FRONTEND noninteractive

# JRE installation and gcc
RUN apt-get update -y && apt-get install -y \
    gcc \
    pkg-config \
    build-essential \
    libsodium-dev \
    libssl-dev \
    libgmp3-dev \
    libsqlite3-dev \
    cmake \
    apt-transport-https \
    ca-certificates \
    software-properties-common \
    debhelper \
    wget \
    git \
    curl \
    libffi-dev \
    ruby \
    ruby-dev \
    rubygems \
    python3 \
    libtool \
    openjdk-8-jdk \
    maven \
    apt-transport-https \
    libzmq3-dev \
    zip \
    unzip \
    rename \
    mysql-client-core-5.7 \
    sudo

# Adding Evernym ca cert
RUN mkdir -p /usr/local/share/ca-certificates
RUN curl -k https://repo.corp.evernym.com/ca.crt | tee /usr/local/share/ca-certificates/Evernym_Root_CA.crt
RUN update-ca-certificates

# Setup apt for evernym repositories
RUN curl https://repo.corp.evernym.com/repo.corp.evenym.com-sig.key | apt-key add -
RUN add-apt-repository "deb https://repo.corp.evernym.com/deb evernym-agency-dev-ubuntu main"
RUN add-apt-repository "deb https://repo.corp.evernym.com/deb evernym-agency-rc-ubuntu main"

# Install Nodejs
RUN curl -sL https://deb.nodesource.com/setup_10.x | bash - \
    && apt-get install -y nodejs

# Install Rust
ARG RUST_VER
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VER}
ENV PATH /root/.cargo/bin:$PATH
RUN cargo install cargo-deb --color=never --version=1.21.1 #TEMPORARY - REMOVE WHEN 1.22 COMPILES

# Install Gradle
RUN wget -q https://services.gradle.org/distributions/gradle-3.4.1-bin.zip &&\
    mkdir /opt/gradle &&\
    unzip -q -d /opt/gradle gradle-3.4.1-bin.zip &&\
    rm gradle-3.4.1-bin.zip

# fpm for deb packaging of npm
RUN gem install fpm
RUN apt-get install rpm -y

COPY ./vcx/ci/scripts/installCert.sh /tmp
RUN /tmp/installCert.sh

# Add evernym repo to sources.lis
RUN add-apt-repository "deb https://repo.corp.evernym.com/deb evernym-agency-dev-ubuntu main"

ARG LIBVDRTOOLS_VER
RUN echo "Libvdrtools Version: ${LIBVDRTOOLS_VER}"

RUN apt-get update && apt install -y libvdrtools=${LIBVDRTOOLS_VER}-bionic

