#!/bin/bash

cd "$(dirname "${BASH_SOURCE[0]}")"

set -ea
shopt -s expand_aliases

if [ -z "$ARCH" ]; then
    ARCH=$(uname -m)
fi

if [ "$ARCH" = "arm64" ]; then
    ARCH="aarch64"
fi

USE_TTY=
if tty -s; then
    USE_TTY="-it"
fi

alias 'rust-musl-builder'='docker run $USE_TTY --rm -e "RUSTFLAGS=$RUSTFLAGS" -v "$HOME/.cargo/registry":/root/.cargo/registry -v "$HOME/.cargo/git":/root/.cargo/git -v "$(pwd)":/home/rust/src -w /home/rust/src -P messense/rust-musl-cross:$ARCH-musl'

rust-musl-builder cargo build --release --target=$ARCH-unknown-linux-musl
if [ "$(ls -nd target/$ARCH-unknown-linux-musl/release/guess-who | awk '{ print $3 }')" != "$UID" ]; then
    rust-musl-builder sh -c "chown -R $UID:$UID target && chown -R $UID:$UID /root/.cargo"
fi