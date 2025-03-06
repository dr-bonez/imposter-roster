FROM scratch

ARG ARCH

ADD ./target/${ARCH}-unknown-linux-musl/release/imposter-roster /usr/bin/imposter-roster