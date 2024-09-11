FROM rust:1.81.0 AS build-env
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc:nonroot
COPY --from=build-env /app/target/release/ddnsrs /
ENTRYPOINT ["/ddnsrs"]
