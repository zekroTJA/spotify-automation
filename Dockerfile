FROM rust:latest AS build
WORKDIR /build
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev musl-tools
RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build -r -p native --target x86_64-unknown-linux-musl

FROM alpine:latest
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/native /app/native
ENV ROCKET_ADDRESS="127.0.0.1"
ENV ROCKET_PORT="80"
EXPOSE 80
ENTRYPOINT [ "/app/native" ]