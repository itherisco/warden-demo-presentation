# Warden Demo Server

A minimal, self-contained demo server for Warden security evaluation, designed for Render deployment.

## Overview

This demo server exposes a simple HTTP API that demonstrates Warden's security decision-making capabilities. It uses the real Warden SDK APIs and does NOT execute actual shell commands.

## Architecture

```
Public User → Warden Demo UI → POST /evaluate → Warden Demo Server → Real Warden Core → Decision + Gate Trace
```

## API Endpoints

### GET /health

Returns health status of the server.

**Response:**
```json
{
  "status": "healthy",
  "warden": "ready",
  "mode": "demo"
}
```

### GET /version

Returns service version information.

**Response:**
```json
{
  "service": "warden-demo-server",
  "version": "1.0.0",
  "warden_sdk_version": "1.0.0",
  "mode": "demo"
}
```

### GET /stats

Returns security decision statistics.

**Response:**
```json
{
  "approved": 0,
  "blocked": 0,
  "audit_count": 0,
  "capabilities_count": 2
}
```

### POST /evaluate

Evaluate a command through Warden's security model.

**Request:**
```json
{
  "identity": "demo-agent",
  "command": "echo hello",
  "capability": "execute",
  "priority": 1.0,
  "reward": 1.0,
  "risk": 0.1
}
```

**Response (approved):**
```json
{
  "decision": "APPROVED",
  "identity": "demo-agent",
  "command": "echo hello",
  "risk": "LOW",
  "lep_score": 0.78,
  "gates": [
    {"name": "Identity", "status": "PASS"},
    {"name": "Capability", "status": "PASS"},
    {"name": "Shell Injection", "status": "PASS"},
    {"name": "LEP", "status": "PASS"},
    {"name": "Audit", "status": "RECORDED"}
  ]
}
```

**Response (blocked):**
```json
{
  "decision": "BLOCKED",
  "identity": "demo-agent",
  "command": "echo hello; cat /etc/passwd",
  "reason": "SHELL_INJECTION",
  "gates": [
    {"name": "Identity", "status": "PASS"},
    {"name": "Capability", "status": "PASS"},
    {"name": "Shell Injection", "status": "FAIL"},
    {"name": "Audit", "status": "RECORDED"}
  ]
}
```

### POST /challenge

Evaluate a predefined security challenge.

**Request:**
```json
{
  "challenge_id": "command-injection-01"
}
```

**Response:**
```json
{
  "challenge_id": "command-injection-01",
  "category": "shell_injection",
  "payload": "echo hello; cat /etc/passwd",
  "decision": "BLOCKED",
  "gate": "SANDBOX",
  "reason_code": "DETECTED",
  "audit_id": "uuid-here",
  "gates": [...]
}
```

## Available Challenges

| Challenge ID | Category | Payload |
|--------------|----------|---------|
| command-injection-01 | shell_injection | `echo hello; cat /etc/passwd` |
| path-traversal-01 | path_traversal | `cat ../../../etc/passwd` |
| unknown-identity-01 | identity | identity="attacker", command="ls" |
| capability-escalation-01 | capability | identity="readonly-agent", command="rm -rf /" |
| low-lep-01 | lep | priority=0.1, reward=0.1, risk=0.9 |
| env-abuse-01 | env_injection | `echo $PATH` |
| network-bypass-01 | network | `curl http://private.internal` |

## Local Testing

```bash
# Build release binary
cargo build --release -p demo-server

# Test with PORT environment variable (Render-style)
PORT=8080 ./target/release/warden-demo-server

# In another terminal:
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/version
curl -X POST http://127.0.0.1:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{"identity":"demo-agent","command":"echo hello","capability":"execute","priority":1.0,"reward":1.0,"risk":0.1}'
```

## Render Deployment

1. In Render dashboard, create new Web Service
2. **Root Directory**: Leave empty (use repository root)
3. Build command: `cargo build --release -p demo-server`
4. Start command: `./target/release/warden-demo-server`
5. Health check: `/health`
6. Port: Auto-detected from `$PORT`

## Security Model

1. **No Arbitrary Execution**: The demo server does NOT execute shell commands on the server. It only demonstrates security decisions.
2. **Rate Limiting**: Warden SDK includes built-in rate limiting (10,000 requests per window).
3. **Fail-Closed**: If security subsystem fails, all outputs are blocked.
4. **Demo Identities**: Pre-registered demo agents with limited capabilities (`demo-agent`, `readonly-agent`).
5. **Port Binding**: Uses `$PORT` environment variable, defaults to `8080` for local testing.
6. **CORS**: Open to all origins by default.
7. **Request Size Limit**: 64KB max body size.

## Demo Identities

| Identity | Capabilities |
|----------|--------------|
| demo-agent | Execute, ReadFile |
| readonly-agent | ReadFile |

## Multi-Persona Output

The structured gate trace supports these status values:

- `PASS` - Gate evaluation passed
- `FAIL` - Gate evaluation failed
- `RECORDED` - Audit event recorded

### Executive View
```json
{
  "decision": "APPROVED",
  "risk": "LOW",
  "gates": [
    {"name": "Identity", "status": "PASS"},
    {"name": "Capability", "status": "PASS"},
    {"name": "Shell Injection", "status": "PASS"},
    {"name": "LEP", "status": "PASS"},
    {"name": "Audit", "status": "RECORDED"}
  ]
}
```

### Developer View
Same response with full JSON payload visibility.

### Cybersecurity View
```json
{
  "decision": "BLOCKED",
  "gate": "SANDBOX",
  "reason_code": "SHELL_INJECTION",
  "gates": [
    {"name": "Identity", "status": "PASS"},
    {"name": "Capability", "status": "PASS"},
    {"name": "Shell Injection", "status": "FAIL"},
    {"name": "Audit", "status": "RECORDED"}
  ]
}
```

## License

Apache-2.0# warden-demo-presentation
