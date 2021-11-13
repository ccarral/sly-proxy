FROM rust:1.56

# WORKDIR /usr/src/sly-proxy
COPY ./ ./

RUN cargo build --release

CMD ["./target/release/sly-proxy"]
