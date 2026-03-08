# Stage 1: Build rift_tui and rift_server
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    clang \
    libclang-dev \
    cmake \
    make \
    git \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain 1.93.0 --profile minimal

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY assets/ assets/
COPY static/ static/

RUN cargo build --release -p rift_tui -p rift_server

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    libasound2 \
    ca-certificates \
    fzf \
    ripgrep \
    fd-find \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/* \
    && ln -s /usr/bin/fdfind /usr/bin/fd

COPY --from=builder /app/target/release/rt /usr/local/bin/rt
COPY --from=builder /app/target/release/rift_server /usr/local/bin/rift_server
COPY --from=builder /app/static /opt/rift/static

WORKDIR /opt/rift

EXPOSE 3000

CMD ["rift_server"]
