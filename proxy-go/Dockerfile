FROM golang:1.26-bookworm

WORKDIR /app

# Copy everything and build
COPY . .
RUN go mod download
RUN CGO_ENABLED=0 GOOS=linux go build -v -o proxy-go .

# Expose the port
EXPOSE 8080

# Run the application
CMD ["./proxy-go"]
