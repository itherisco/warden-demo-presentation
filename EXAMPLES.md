# Warden Demo Server - Example Requests

## Quick Start

```bash
# Start the server
PORT=8080 ./target/release/warden-demo-server

# In another terminal, test endpoints:
```

## Health Check

```bash
curl -s http://localhost:8080/health
```

## Version Info

```bash
curl -s http://localhost:8080/version
```

## Statistics

```bash
curl -s http://localhost:8080/stats
```

## Evaluate Endpoint Examples

### Safe Command (Approved)

```bash
curl -s -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "identity": "demo-agent",
    "command": "echo hello",
    "capability": "execute",
    "priority": 1.0,
    "reward": 1.0,
    "risk": 0.1
  }'
```

**Response:**
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

### Shell Injection Blocked

```bash
curl -s -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "identity": "demo-agent",
    "command": "echo hello; cat /etc/passwd",
    "capability": "execute",
    "priority": 1.0,
    "reward": 1.0,
    "risk": 0.1
  }'
```

**Response:**
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

### Unknown Identity Blocked

```bash
curl -s -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "identity": "attacker",
    "command": "ls -la",
    "capability": "execute",
    "priority": 1.0,
    "reward": 1.0,
    "risk": 0.1
  }'
```

**Response:**
```json
{
  "decision": "BLOCKED",
  "identity": "attacker",
  "command": "ls -la",
  "reason": "UNKNOWN_IDENTITY",
  "gates": [
    {"name": "Identity", "status": "FAIL"},
    {"name": "Audit", "status": "RECORDED"}
  ]
}
```

### Unauthorized Capability Blocked

```bash
curl -s -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "identity": "readonly-agent",
    "command": "echo hello",
    "capability": "execute",
    "priority": 1.0,
    "reward": 1.0,
    "risk": 0.1
  }'
```

**Response:**
```json
{
  "decision": "BLOCKED",
  "identity": "readonly-agent",
  "command": "echo hello",
  "reason": "DENIED",
  "gates": [
    {"name": "Identity", "status": "PASS"},
    {"name": "Capability", "status": "FAIL"},
    {"name": "Audit", "status": "RECORDED"}
  ]
}
```

### Low LEP Score Blocked

```bash
curl -s -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "identity": "demo-agent",
    "command": "ls",
    "capability": "execute",
    "priority": 0.1,
    "reward": 0.1,
    "risk": 0.9
  }'
```

**Response:**
```json
{
  "decision": "BLOCKED",
  "identity": "demo-agent",
  "command": "ls",
  "reason": "VETOED",
  "gates": [
    {"name": "Identity", "status": "PASS"},
    {"name": "Capability", "status": "PASS"},
    {"name": "Shell Injection", "status": "PASS"},
    {"name": "LEP", "status": "FAIL"},
    {"name": "Audit", "status": "RECORDED"}
  ]
}
```

## Challenge Endpoint Examples

### Command Injection Challenge

```bash
curl -s -X POST http://localhost:8080/challenge \
  -H "Content-Type: application/json" \
  -d '{"challenge_id": "command-injection-01"}'
```

### Path Traversal Challenge

```bash
curl -s -X POST http://localhost:8080/challenge \
  -H "Content-Type: application/json" \
  -d '{"challenge_id": "path-traversal-01"}'
```

### Unknown Identity Challenge

```bash
curl -s -X POST http://localhost:8080/challenge \
  -H "Content-Type: application/json" \
  -d '{"challenge_id": "unknown-identity-01"}'
```

### Capability Escalation Challenge

```bash
curl -s -X POST http://localhost:8080/challenge \
  -H "Content-Type: application/json" \
  -d '{"challenge_id": "capability-escalation-01"}'
```

### Low LEP Challenge

```bash
curl -s -X POST http://localhost:8080/challenge \
  -H "Content-Type: application/json" \
  -d '{"challenge_id": "low-lep-01"}'
```

### Environment Abuse Challenge

```bash
curl -s -X POST http://localhost:8080/challenge \
  -H "Content-Type: application/json" \
  -d '{"challenge_id": "env-abuse-01"}'
```

### Network Bypass Challenge

```bash
curl -s -X POST http://localhost:8080/challenge \
  -H "Content-Type: application/json" \
  -d '{"challenge_id": "network-bypass-01"}'
```

## All Available Challenges

| ID | Category | Payload |
|---|---|---|
| `command-injection-01` | shell_injection | `echo hello; cat /etc/passwd` |
| `path-traversal-01` | path_traversal | `cat ../../../etc/passwd` |
| `unknown-identity-01` | identity | identity="attacker", command="ls" |
| `capability-escalation-01` | capability | identity="readonly-agent", command="rm -rf /" |
| `low-lep-01` | lep | priority=0.1, reward=0.1, risk=0.9 |
| `env-abuse-01` | env_injection | `echo $PATH` |
| `network-bypass-01` | network | `curl http://private.internal` |