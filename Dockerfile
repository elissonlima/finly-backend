FROM rust:1.83 AS build

# 1. Create a new empty shell project
RUN USER=root cargo new --bin finly-backend
WORKDIR /finly-backend

# 2. Copy our manifest
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This build step will cache the dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN mkdir /database
COPY ./.db/main.db /database
ENV DATABASE_URL sqlite:///database/main.db
RUN rm ./target/release/deps/finly_backend*
RUN cargo build --release

FROM rust:1.83

# copy the build artifact from the build stage
COPY --from=build /finly-backend/target/release/finly-backend .

RUN mkdir -p /app/database
RUN mkdir /app/sql
RUN mkdir /app/ssl
RUN mkdir /app/html

# set the startup command to run your binary
CMD ["./finly-backend"]
