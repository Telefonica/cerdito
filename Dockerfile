# Versions must be major.minor
ARG RUST_VERSION=1.77
ARG ALPINE_VERSION=3.19

FROM docker.io/rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS builder
COPY . /data
RUN apk -U add libc-dev && cd /data && cargo build --release --locked

FROM docker.io/alpine:${ALPINE_VERSION}
COPY --from=builder /data/target/release/cerdito /bin
CMD [ "/bin/sh" ]
