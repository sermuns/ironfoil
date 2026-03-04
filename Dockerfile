FROM rust:1.91.0-alpine3.22 AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock .
COPY cli/Cargo.toml cli/Cargo.toml
COPY core/Cargo.toml core/Cargo.toml

RUN mkdir -p cli/src core/src \
    && echo "fn main() { println!(\"dummy build\") }" > cli/src/main.rs \
    && touch core/src/lib.rs

RUN cargo build --release

COPY . .

RUN cargo build --release --locked


FROM scratch

COPY --from=builder /app/target/release/ironfoil /bin/ironfoil

WORKDIR /app
USER 1000:1000

ENTRYPOINT ["ironfoil"]
