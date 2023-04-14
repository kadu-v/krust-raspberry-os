FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=C.UTF-8

RUN apt-get update && apt-get install -y tzdata
ENV TZ Asia/Tokyo
RUN apt-get update && apt-get install -y curl sudo git wget gcc qemu-system-aarch64
ENV LIBGL_ALWAYS_INDIRECT=1

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"

RUN rustup install nightly && \
    rustup default nightly && \
    rustup target install aarch64-unknown-none-softfloat && \
    rustup component add llvm-tools-preview

RUN cargo install cargo-binutils
CMD ["bash"]