# Warden Demo Server Presentation

A unified, presentation-ready folder containing all assets for the Warden Demo Server.

## Folder Structure

```
warden-demo-presentation/
├── Cargo.toml           # Standalone package definition
├── README.md            # Full documentation
├── QUICKSTART.md        # 5-minute setup guide
├── RENDER_DEPLOYMENT.md # Render deployment instructions
├── EXAMPLES.md          # API request/response examples
├── render.yaml          # Render deployment configuration
├── test.sh              # Automated test script
└── src/
    ├── main.rs          # HTTP API server (444 lines)
    └── types.rs         # Request/response types
```

## Quick Start

```bash
# Build
cargo build --release

# Run
PORT=8080 ./target/release/warden-demo-server

# Test
./test.sh
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /health | Health check |
| GET | /version | Service version |
| GET | /stats | Security statistics |
| POST | /evaluate | Evaluate command |
| POST | /challenge | Run security challenge |

## Key Features

- Real Warden SDK integration
- Shell injection detection
- Gate trace simulation
- 7 predefined security challenges
- CORS enabled, 64KB request limit
- Render-ready configuration