# Build stage
FROM golang:1.26-bookworm AS builder

WORKDIR /app

# Download dependencies first for better caching
COPY go.mod go.sum ./
RUN go mod download

# Copy source and build
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -v -o bearer-go .

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

# Install certificates for HTTPS requests if needed
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/bearer-go .

# Expose the port
EXPOSE 8080

# Run the application
CMD ["./bearer-go"]
