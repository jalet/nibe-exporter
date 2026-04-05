# Stage 1: Chef base (install cargo-chef once)
FROM --platform=${BUILDPLATFORM} rust:1.94 AS chef
WORKDIR /app
RUN cargo install cargo-chef

# Stage 2: Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Cacher
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Builder
FROM --platform=${BUILDPLATFORM} rust:1.94 AS builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin nibe-exporter

# Stage 5: Runtime
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/nibe-exporter /

EXPOSE 9090
ENTRYPOINT ["/nibe-exporter"]
