ARG BASE_IMAGE=rust:1.62.0-slim-buster

FROM $BASE_IMAGE as builder
WORKDIR app
COPY . .
RUN cargo build --release
CMD ["./target/release/esp32-receiver"]

FROM $BASE_IMAGE
COPY --from=builder /app/target/release/esp32-receiver /
COPY --from=builder /app/ca-certificate.crt /
COPY --from=builder /app/Rocket.toml /
CMD ["./esp32-receiver"]