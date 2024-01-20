FROM rust:1.75-slim-buster as builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY ./jirascope-dyn ./jirascope-dyn
COPY ./jirascope-core ./jirascope-core
COPY ./jirascope-cli ./jirascope-cli
COPY ./jirascope-test-server ./jirascope-test-server

# Build jirascope-dyn in release mode

RUN cargo build --release -p jirascope-dyn

# Output: /usr/src/app/target/release/libjirascope_dyn.so
