FROM ubuntu:20.04

RUN apt-get update && \
    apt-get install -y curl build-essential && \
    curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /usr/src/app
COPY . .

RUN --mount=type=cache,target=target cargo build -p lsp-server --release && mkdir output && cp target/release/lsp-server output/