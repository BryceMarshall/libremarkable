FROM rust:latest
LABEL authors="bryce"


RUN apt-get update
RUN #apt-get install -y gcc-arm-linux-gnueabihf cmake libclang-dev musl-tools musl-dev libssl-dev
RUN apt-get install -y \
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
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

RUN wget -q https://musl.cc/armv7l-linux-musleabihf-cross.tgz && \
    tar -xzf armv7l-linux-musleabihf-cross.tgz && \
    mv armv7l-linux-musleabihf-cross /opt/ && \
    rm armv7l-linux-musleabihf-cross.tgz

RUN rustup target add armv7-unknown-linux-musleabihf

ENV PATH="/opt/armv7l-linux-musleabihf-cross/bin:${PATH}"


ENV CC_armv7_unknown_linux_musleabihf=armv7l-linux-musleabihf-gcc
ENV CXX_armv7_unknown_linux_musleabihf=armv7l-linux-musleabihf-g++
ENV AR_armv7_unknown_linux_musleabihf=armv7l-linux-musleabihf-ar
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=armv7l-linux-musleabihf-gcc

ENV CFLAGS_armv7_unknown_linux_musleabihf="-I/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf/include"
ENV CXXFLAGS_armv7_unknown_linux_musleabihf="-I/opt/armv7l-linux-musleabihf-cross/armv7l-linux-musleabihf/include"

ENV PYO3_CROSS_PYTHON_VERSION=3.8
ENV PYO3_CROSS=1
ENV PYO3_NO_PYTHON=1

RUN cargo install --force --locked bindgen-cli

# Install useful development tools
#RUN cargo install cargo-watch cargo-edit cargo-tree || true

WORKDIR /app