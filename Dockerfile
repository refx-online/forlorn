FROM rustlang/rust:nightly AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ENV RUSTFLAGS="-C target-cpu=native -C target-feature=+crt-static -C link-arg=-static"

# depend on musl since im still at 22.04
# and i dont have glibc_2.39
RUN apt-get update && apt-get install -y musl-tools pkg-config build-essential
RUN rustup target add x86_64-unknown-linux-musl

COPY --from=planner /app/recipe.json recipe.json

# build da dependencies with registry cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl

COPY . .

# "build" da actual application with cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --target x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/forlorn /forlorn

FROM gcr.io/distroless/static

COPY --from=builder /forlorn /usr/local/bin/forlorn

ENTRYPOINT ["/usr/local/bin/forlorn"]
