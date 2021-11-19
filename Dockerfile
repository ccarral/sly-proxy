FROM rust:1.56

# WORKDIR /usr/src/sly-proxy
COPY ./ ./

RUN cargo build --release

EXPOSE 8080
EXPOSE 8081
