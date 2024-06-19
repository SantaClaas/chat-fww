# Build the client SPA
FROM node:21 AS build-client
WORKDIR /client

# Installs pnpm as it is set as package manager in package.json
RUN corepack enable

# Copy over manifests
COPY ./client/package.json ./package.json
COPY ./client/pnpm-lock.yaml ./pnpm-lock.yaml

# Install dependencies
RUN pnpm install

# Copy over the source to build the application
COPY ./client ./

# Build and cache
RUN pnpm run build

# Build the server containing the API and hosting the client
FROM rust:1.79 AS build-server

# Create a new empty shell project
RUN USER=root cargo new --bin server
WORKDIR /server

# Copy over manifests
# The lock file is in the workspace root. Should probably use the workspace Cargo.toml too but this works so far
COPY ./Cargo.lock ./Cargo.lock
COPY ./server/Cargo.toml ./Cargo.toml

# Install serialport crate dependencies
RUN apt-get update && apt-get install -y libudev-dev

# Build and cache the dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy over the source to build the application
COPY ./server/src ./src

# Build the application
RUN rm ./target/release/deps/server*
RUN cargo build --release

# Final base image
FROM debian:bookworm-slim AS final

# Copy the build artifacts from the build stage
COPY --from=build-server /server/target/release/server .
# The ./client directory is where the server looks for when client static files are requested
COPY --from=build-client /client/dist ./client

EXPOSE 3000
# Set the startup command to run the application
CMD ["./server"]