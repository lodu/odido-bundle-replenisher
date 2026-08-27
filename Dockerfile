FROM rust:1.88-alpine3.21 AS builder

WORKDIR /app

RUN apk add --no-cache build-base cmake perl

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM alpine:3.21 AS production

RUN apk add --no-cache ca-certificates libgcc

COPY --from=builder /app/target/release/odido-bundle-replenisher /usr/local/bin/odido-bundle-replenisher

ENTRYPOINT ["odido-bundle-replenisher"]
