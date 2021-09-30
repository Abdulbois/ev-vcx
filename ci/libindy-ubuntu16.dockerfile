# Development
FROM ubuntu:16.04

# JRE installation and gcc
RUN apt-get update -y && apt-get install -y \
    gcc \
    pkg-config \
    build-essential \
    libsodium-dev \
    libssl-dev \
    libgmp3-dev \
    libsqlite3-dev \
    libsqlite0 \
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
    libzmq5 \
    python3 \
    libtool \
    openjdk-8-jdk \
    maven \
    apt-transport-https \
    libzmq3-dev \
    zip \
    unzip \
    sudo

# Install Nodejs
RUN curl -sL https://deb.nodesource.com/setup_8.x | bash - \
    && apt-get install -y nodejs

# Install Rust
ARG RUST_VER
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER
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

# Add evernym.corp to sources.list
RUN add-apt-repository 'deb https://repo.corp.evernym.com/deb evernym-agency-dev-ubuntu main' && \
    curl https://repo.corp.evernym.com/repo.corp.evenym.com-sig.key | apt-key add -

ARG LIBINDY_VER

RUN apt update && apt install -y libmysqlstorage=0.1.13 \
    libindy-async=1.95.0~208-xenial

