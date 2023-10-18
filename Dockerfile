FROM rust:1.73 as builder

WORKDIR /usr/src/massrelay
COPY . .

RUN cargo install --path .

FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/massrelay /usr/local/bin/massrelay

CMD ["massrelay"]