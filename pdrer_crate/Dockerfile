# Stage 1: Builder stage to compile the Rust application
FROM rust:1.86 as builder
# Set the working directory inside the container
WORKDIR /usr/src/rust_code
# Copy the Cargo manifest first for efficient caching
COPY Cargo.toml ./
# Copy build script
COPY build.rs ./
# Copy the source code directory
COPY src ./src
# Copy the examples directory
COPY examples ./examples
# Build the Rust project in release mode and build all examples
RUN RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --example pdr_engine_for_hwmcc --target x86_64-unknown-linux-gnu 

# Final Stage: Runtime stage to create the final, smaller image
FROM debian:bullseye-slim
# Update package lists and install necessary runtime dependencies
# --no-install-recommends avoids installing unnecessary suggested packages
RUN apt-get update
# RUN apt-get install -y --no-install-recommends libssl-dev ca-certificates 
# RUN apt-get install -y software-properties-common
# RUN add-apt-repository -y ppa:ubuntu-toolchain-r/test
# RUN apt update
# RUN apt-get install gcc-11 libstdc++6

RUN rm -rf /var/lib/apt/lists/*
# Copy the specific compiled example binary from the builder stage
# The binary will be located in the target/release/examples directory
COPY --from=builder /usr/src/rust_code/target/x86_64-unknown-linux-gnu/release/examples/pdr_engine_for_hwmcc /usr/local/bin/pdr_engine_for_hwmcc

# Set the default command to run the specified example executable
ENTRYPOINT  ["pdr_engine_for_hwmcc"]