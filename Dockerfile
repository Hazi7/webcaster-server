FROM rust:latest
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-gnu