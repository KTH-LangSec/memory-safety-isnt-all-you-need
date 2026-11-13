# SPDX-License-Identifier: CC0-1.0

FROM rust:1.97.1-alpine

RUN apk add --no-cache make musl-dev python3 \
 && rustup component add rustfmt clippy \
 && cargo install wasm-pack

WORKDIR /interpreter

CMD ["sh"]
