FROM rust:1.64.0 as build-env
WORKDIR /app
COPY . /app
RUN cargo build --release --offline

FROM gcr.io/distroless/cc:nonroot
COPY --from=build-env /app/target/release/ddnsrs /
ENTRYPOINT ["/ddnsrs"]
