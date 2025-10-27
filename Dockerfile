# syntax=docker/dockerfile:1

################################################################################
# Stage 1: Build
################################################################################
FROM rust:1.90-alpine AS builder
WORKDIR /app

# Install required build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig clang lld git

# Copy manifest files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release || true

# Now copy the full project source
COPY . .

# Build the release binary
RUN cargo build --release

################################################################################
# Stage 2: Runtime
################################################################################
FROM alpine:3.18 AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/space-together-api /app/space-together-api

# Expose the port your Actix app listens on
EXPOSE 4646

# Create a non-root user for security
RUN adduser -D appuser
USER appuser

# Start the application
CMD ["./space-together-api"]
