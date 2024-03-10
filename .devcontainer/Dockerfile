FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=C.UTF-8

RUN apt-get update && apt-get install -y tzdata
ENV TZ Asia/Tokyo
RUN apt-get update && apt-get install -y curl sudo git wget gcc qemu-system-aarch64
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