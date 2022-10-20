FROM rust:1.64.0 as build-env
RUN update-ca-certificates && \
    rustup target add x86_64-unknown-none &&\
    rustup target add aarch64-unknown-none
WORKDIR /app
COPY . /app
RUN cargo build --release --all-targets

FROM gcr.io/distroless/cc:nonroot
COPY --from=build-env /app/target/release/ddnsrs /
ENTRYPOINT ["/ddnsrs"]
