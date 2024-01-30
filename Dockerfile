# Use rust base image
FROM rust:latest as builder

# Set working directory inside the container
WORKDIR /usr/src/ezi

# Copy the Rust project to the container
COPY . .

# Build the Rust application
RUN cargo build --release

# Start a new stage from the base image
FROM ubuntu:latest

# Install necessary dependencies for Chromium
RUN apt-get update && apt-get install -y \
    chromium-browser \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the built executable from the previous stage
COPY --from=builder /usr/src/ezi/target/release/ezi /usr/local/bin/ezi

# Run the Rust application
CMD ["ezi"]
