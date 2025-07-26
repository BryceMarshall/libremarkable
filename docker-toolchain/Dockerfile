FROM ubuntu:16.04
LABEL maintainer='Charlton Rodda'

# Need to specify UID and GID so they match the external user.
# UNAME has no significance.
ARG UNAME=builder
ARG UID=1000
ARG GID=1000
ARG ostype=Linux

RUN apt-get -qq update
RUN #apt-get -qq install curl build-essential gcc-arm-linux-gnueabihf vim
RUN apt-get -qq install -y \
    curl \
    musl-tools \
    musl-dev \
    libclang-dev \
    wget \
    clang \
    llvm \
    gcc-multilib \
    libc6-dev \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN wget -q https://musl.cc/armv7l-linux-musleabihf-cross.tgz && \
    tar -xzf armv7l-linux-musleabihf-cross.tgz && \
    mv armv7l-linux-musleabihf-cross /opt/ && \
    rm armv7l-linux-musleabihf-cross.tgz

RUN bash -c 'if [ ${ostype} == Linux ]; then groupadd -r --gid ${GID} ${UNAME} || true; fi'
RUN useradd -u $UID -g $GID -m $UNAME
USER $UNAME

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
ENV PATH="${PATH}:/home/$UNAME/.cargo/bin"
RUN rustup target add armv7-unknown-linux-musleabihf
ENV PATH="/opt/armv7l-linux-musleabihf-cross/bin:${PATH}"

ENV CC_armv7_unknown_linux_musleabihf=armv7l-linux-musleabihf-gcc
ENV CXX_armv7_unknown_linux_musleabihf=armv7l-linux-musleabihf-g++
ENV AR_armv7_unknown_linux_musleabihf=armv7l-linux-musleabihf-ar
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=armv7l-linux-musleabihf-gcc

ENV CFLAGS_armv7_unknown_linux_musleabihf="-I/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf/include"
ENV CXXFLAGS_armv7_unknown_linux_musleabihf="-I/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf/include"

RUN #rustup target add armv7-unknown-linux-gnueabihf

# make the registry folder to ensure correct permissions
RUN mkdir -p "/home/$UNAME/.cargo/registry"

ADD ./.cargo/config /home/$UNAME/.cargo/config
RUN cargo install --force --locked bindgen-cli

