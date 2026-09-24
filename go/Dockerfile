# ---------------------------------------
# Stage 1: Build the Go binary
# ---------------------------------------
FROM docker.io/golang:1.27-alpine AS builder

WORKDIR /app

# Cache dependencies first (speeds up rebuilds)
COPY go.mod go.sum ./
RUN go mod download

# Copy source and compile statically
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-w -s" -o /go-worker .

# ---------------------------------------
# Stage 2: Minimal Runtime Environment
# ---------------------------------------
FROM docker.io/alpine:latest

# Install Chromium for the headless browser, and CA certs for HTTPS downloads
RUN apk add --no-cache chromium ca-certificates tzdata

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /go-worker /app/go-worker

# Create a non-root user
RUN addgroup -S workergroup && adduser -S worker -G workergroup
RUN chown -R worker:workergroup /app

# Switch to the non-root user
USER worker

# Tell chromedp where Alpine installed the browser
ENV CHROME_BIN=/usr/bin/chromium-browser

CMD ["/app/go-worker"]
