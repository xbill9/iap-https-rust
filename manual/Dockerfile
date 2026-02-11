# Use a Rust image for building the application
FROM rust:1.93-bookworm AS builder

WORKDIR /app

# Copy Cargo.toml and Cargo.lock to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Build dependencies first. This layer is cached if Cargo.toml and Cargo.lock don't change.
# We create a dummy src/main.rs to make cargo happy.
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --locked --target x86_64-unknown-linux-gnu && \
    rm -rf src

# Copy the source code
COPY src ./src

# Build the actual application. 
# We touch src/main.rs to ensure it's rebuilt after the dummy build.
RUN touch src/main.rs && \
    cargo build --release --locked --target x86_64-unknown-linux-gnu

# Use a minimal base image for the final stage
FROM gcr.io/distroless/cc-debian12

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/manual-https-rust .

# Expose the port the application listens on
EXPOSE 8080

# Run the application
CMD ["./manual-https-rust"]
