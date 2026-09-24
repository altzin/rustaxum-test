# Bookmark Worker & BFF Architecture

## Overview

This monorepo contains a Rust/Axum Backend-for-Frontend (BFF) and a Go-based worker service. They communicate asynchronously via NATS JetStream.

The primary workflow involves a browser extension sending URLs to the Axum API, which queues them in JetStream. The scalable Go worker pulls these jobs, generates Markdown representations of the web pages, and publishes completion events back to NATS.

## System Components

### 1. Axum BFF (Rust)

The Axum service acts as the core domain owner for Bookmarks. It utilizes a "Ports and Adapters" architecture to maintain the Single Responsibility Principle without needing unnecessary microservices.

- **Core Domain Logic:** A single shared Rust function handles PostgreSQL database mutations (status updates).
- **HTTP Adapter:** Serves the REST API for the browser extension and Admin UI (e.g., `PATCH /api/bookmarks/:id`).
- **Event Adapter:** A Tokio background task that subscribes to NATS JetStream `bookmark.completed` events and invokes the core domain logic.

### 2. Generator Worker (Go)

The Go worker is strictly a consumer and producer of NATS messages. It does not communicate with Axum over HTTP.

- **Web Scraping:** Uses `chromedp` to fully render Single Page Applications (SPAs) and wait for the `<body>` to load, ignoring CSS and ads.
- **Processing:** Scrubs metadata, generates Markdown, and handles images separately (saving to an S3-compatible backend like GarageHQ on a shared cluster disk).
- **Event Publishing:** Upon completion, publishes a `bookmark.completed` event to JetStream, then `Ack`s the original job.

## The Fast Path

1.  Extension sends a URL to Axum.
2.  Axum saves the record to PostgreSQL with a `pending` status.
3.  Axum publishes the job to JetStream.
4.  Go worker receives the event and begins generation.
5.  Go worker uploads the generated files to storage.
6.  Go worker publishes a `bookmark.completed` event to JetStream with the storage URLs.
7.  Axum's background NATS subscriber receives the completion event and updates PostgreSQL.
8.  Go worker `Ack`s the original message, clearing it from the queue.

## Resiliency & Idempotency

We rely on NATS JetStream's native features and idempotent worker logic rather than manual database sweeps.

- **AckWait Timers:** If a Go worker crashes before sending an `Ack`, JetStream's timer expires and automatically redelivers the message to another worker.
- **Storage Idempotency:** Storage object names are created by SHA-hashing the target URLs. If a worker receives a ghost/redelivered job, it checks storage first. If the file exists, it skips generation, fires the completion event, and `Ack`s the queue.
- **Poison Pills:** If a URL fails to generate after 3 retries, the worker catches the poison pill and routes it to a Dead Letter Queue (DLQ).
- **Unthrottled Ingest:** The API does not limit uploads. Spikes in traffic are buffered safely by JetStream, allowing workers to drain the queue at their own pace.

## Observability & Tracing

- Distributed trace context is passed from the HTTP request through JetStream to the Go worker.
- We use span links to track the latency of database writes versus the long run-time of generators.
- Metrics are collected from PostgreSQL and JetStream via Vector or Prometheus.
- PostgreSQL statement statistics are toggled to monitor query performance.

## Over-Engineering for Learning

- **Postgres Advisory Locks:** Even though the asynchronous NATS architecture handles state cleanly, transaction-level advisory locks are implemented in Axum during specific database mutations to explore and learn about advanced PostgreSQL locking mechanisms.

## Client Integration

- The system will be consumed by a custom Firefox (or Brave) extension.
- The BFF URL is not hardcoded; it is configurable within the addon's settings.
