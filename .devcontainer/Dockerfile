FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=C.UTF-8

RUN apt-get update && apt-get install -y tzdata
ENV TZ Asia/Tokyo
RUN apt-get update && apt-get install -y curl sudo git wget gcc xz-utils

# Build the latest QEMU from source (the raspi4b machine requires QEMU >= 8.2,
# but Ubuntu 22.04 ships QEMU 6.2)
ARG QEMU_VERSION=11.0.2
RUN apt-get update && apt-get install -y \
    build-essential ninja-build pkg-config python3 python3-venv python3-pip \
    flex bison \
    libglib2.0-dev libpixman-1-dev libfdt-dev libslirp-dev zlib1g-dev \
    libgtk-3-dev libepoxy-dev && \
    python3 -m pip install tomli
RUN wget -q https://download.qemu.org/qemu-${QEMU_VERSION}.tar.xz && \
    tar xf qemu-${QEMU_VERSION}.tar.xz && \
    cd qemu-${QEMU_VERSION} && \
    ./configure --target-list=aarch64-softmmu --enable-gtk --enable-slirp && \
    make -j"$(nproc)" && \
    make install && \
    cd .. && rm -rf qemu-${QEMU_VERSION} qemu-${QEMU_VERSION}.tar.xz

ENV LIBGL_ALWAYS_INDIRECT=1

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"

# Must install cargo-binutils using the newest rustc
RUN cargo install cargo-binutils 

RUN rustup install nightly-2022-10-13 && \
    rustup default nightly-2022-10-13 && \
    rustup target install aarch64-unknown-none-softfloat && \
    rustup component add llvm-tools-preview

CMD ["bash"]