# Stage 1: Planner
FROM --platform=${BUILDPLATFORM} rust:1.88 AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cacher
FROM --platform=${BUILDPLATFORM} rust:1.88 AS cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: Builder
FROM --platform=${BUILDPLATFORM} rust:1.88 AS builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
RUN cargo build --release --bin nibe-exporter

# Stage 4: Runtime
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/nibe-exporter /

EXPOSE 9090
ENTRYPOINT ["/nibe-exporter"]
