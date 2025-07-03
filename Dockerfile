# Use Rust official image with musl support
FROM rust:1.88.0-alpine3.22 as builder

# Install required dependencies for building
RUN apk add --no-cache musl-dev pkgconf perl make

# Set working directory inside the container
WORKDIR /app

# First: copy Cargo.toml files to cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY entity/Cargo.toml entity/
COPY entity/src entity/src/

# Create a dummy main.rs that references entity
RUN mkdir src && \
    echo 'fn main() { println!("dummy"); }' > src/main.rs

# Build for dependency caching
RUN cargo build --release --target x86_64-unknown-linux-musl

# Copy the actual source code
COPY src src/

# Touch main.rs to ensure it rebuilds with actual content
RUN touch src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

# ---- Final Stage ----
FROM alpine:3.22.0

# Install necessary runtime dependencies and debugging tools
RUN apk add --no-cache \
    libgcc \
    ca-certificates \
    postgresql-client \
    strace \
    bash \
    iputils \
    bind-tools \
    curl

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ensenas-service /app/ensenas-service

# Make sure the binary is executable
RUN chmod +x /app/ensenas-service

# Expose the application port
EXPOSE 8080

# Set default environment variables
ENV RUST_BACKTRACE=1 \
    RUST_LOG=debug \
    JWT_SECRET=somesecret

# Copy and setup entrypoint
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

CMD ["/app/entrypoint.sh"]
