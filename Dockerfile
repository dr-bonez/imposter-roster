FROM scratch

ARG ARCH

ADD ./target/${ARCH}-unknown-linux-musl/release/guess-who /usr/bin/guess-who