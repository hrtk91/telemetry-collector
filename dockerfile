# RaspberryPi向けビルド glibc2.31
FROM ubuntu:20.04

# パッケージの更新
RUN apt update -y && apt install -y gcc-arm-linux-gnueabi gcc-arm-linux-gnueabihf build-essential curl

# Rustのインストール
ENV RUST_HOME /usr/local/lib/rust
ENV RUSTUP_HOME ${RUST_HOME}/rustup
ENV CARGO_HOME ${RUST_HOME}/cargo
RUN mkdir /usr/local/lib/rust && \
    chmod 0755 $RUST_HOME
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > ${RUST_HOME}/rustup.sh \
    && chmod +x ${RUST_HOME}/rustup.sh \
    && ${RUST_HOME}/rustup.sh -y --default-toolchain nightly --no-modify-path
ENV PATH $PATH:$CARGO_HOME/bin

# クロスコンパイル用のツールチェイン
RUN rustup target add arm-unknown-linux-gnueabi
RUN rustup target add armv7-unknown-linux-gnueabihf

RUN mkdir /app

# ビルド
CMD ["./entrypoint.sh"]
