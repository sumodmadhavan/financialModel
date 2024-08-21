# Use a newer version of Rust
FROM rust:1.75 as builder

# Set the working directory in the container
WORKDIR /usr/src/financial_calculator

# Copy the current directory contents into the container
COPY . .

# Build the application
RUN cargo build --release

# Use a newer base image for the final stage
FROM debian:bookworm-slim

# Install necessary libraries
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/financial_calculator/target/release/financial_calculator /usr/local/bin/financial_calculator

# Expose port 8080
EXPOSE 8080

# Run the binary
CMD ["financial_calculator"]
