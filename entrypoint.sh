#!/bin/sh

cargo build --release --target armv7-unknown-linux-gnueabihf
cargo build --release --target arm-unknown-linux-gnueabi

mkdir armv7
mkdir armv6

cp ./target/armv7-unknown-linux-gnueabihf/release/collector-core ./armv7
cp ./target/armv7-unknown-linux-gnueabihf/release/collector-webapi ./armv7
cp ./target/arm-unknown-linux-gnueabi/release/collector-core ./armv6
cp ./target/arm-unknown-linux-gnueabi/release/collector-webapi ./armv6
