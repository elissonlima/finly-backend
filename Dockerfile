FROM ubuntu:24.04 AS build

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install --no-install-recommends -y curl build-essential ca-certificates libssl-dev pkg-config golang-go
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# 1. Create a new empty shell project
WORKDIR /root
RUN cargo new --bin finly-backend
WORKDIR /root/finly-backend

# Build google_oauth_api_lib
COPY ./google_oauth_api_client ./google_oauth_api_client
WORKDIR /root/finly-backend/google_oauth_api_client
RUN go build

WORKDIR /root/finly-backend

# 2. Copy our manifest
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This build step will cache the dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
#RUN mkdir /database
ENV DATABASE_URL postgres://finly:somefancypassword@finly-backend-db-1:5432/finly
RUN rm ./target/release/deps/finly_backend*
RUN cargo build --release


FROM ubuntu:24.04

RUN apt-get update && apt-get install --no-install-recommends -y ca-certificates
RUN update-ca-certificates

# copy the build artifact from the build stage
COPY --from=build /root/finly-backend/target/release/finly-backend .
COPY --from=build /root/finly-backend/google_oauth_api_client/google_oauth_api_client /usr/bin/google_oauth_api_client

RUN mkdir -p /app/database
RUN mkdir /app/sql
RUN mkdir /app/ssl
RUN mkdir /app/html

# set the startup command to run your binary
CMD ["./finly-backend"]
