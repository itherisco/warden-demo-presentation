# Warden Demo Server - Quick Start Guide

## Prerequisites

- Rust 1.70+ installed
- Cargo package manager
- Access to Warden SDK (in parent directory)

## Build & Run

```bash
# Build the release binary
cargo build --release

# Start the server (Render-style)
PORT=8080 ./target/release/warden-demo-server
```

## Test Endpoints

```bash
# Health check
curl http://localhost:8080/health

# Test evaluate
curl -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{"identity":"demo-agent","command":"echo hello","capability":"execute","priority":1.0,"reward":1.0,"risk":0.1}'

# Run automated tests
./test.sh
```

## Available Identities

| Identity | Capabilities |
|----------|--------------|
| `demo-agent` | Execute, ReadFile |
| `readonly-agent` | ReadFile |

## API Reference

### GET /health
Returns server health status.

### GET /version
Returns service version information.

### GET /stats
Returns security decision statistics.

### POST /evaluate
Evaluate a command through Warden's security model.

### POST /challenge
Evaluate a predefined security challenge.

See EXAMPLES.md for detailed request/response examples.