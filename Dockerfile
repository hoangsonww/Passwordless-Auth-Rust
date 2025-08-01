FROM rust:1.72-bookworm AS builder
WORKDIR /usr/src/app
COPY . .
RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/passwordless-auth /usr/local/bin/passwordless-auth
COPY --from=builder /usr/src/app/target/release/email-worker /usr/local/bin/email-worker
COPY config.toml .
COPY migrations ./migrations
VOLUME ["/app/auth.db"]
ENV RUST_LOG=info
EXPOSE 3000
ENTRYPOINT ["passwordless-auth"]
