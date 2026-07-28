# Warden Demo Server - Render Deployment Guide

## Deployment Steps

1. **Create New Web Service**
   - Navigate to Render Dashboard → New → Web Service

2. **Repository Setup**
   - Connect your GitHub repository containing this demo-server folder
   - Root Directory: Leave empty (or specify if in subfolder)

3. **Build Configuration**
   - Build Command: `cargo build --release -p demo-server`
   - Start Command: `./target/release/warden-demo-server`

4. **Environment Variables**
   - `RUST_LOG`: `info`
   - `CORS_ORIGINS`: `https://your-frontend.vercel.app,http://localhost:3000,http://localhost:5173`

5. **Service Settings**
   - Plan: Free
   - Region: Oregon (or preferred)
   - Runtime: Rust
   - Health Check Path: `/health`
   - Auto Deploy: Enabled

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 8080 | Server port (auto-assigned by Render) |
| `RUST_LOG` | info | Logging level |
| `CORS_ORIGINS` | (see above) | Allowed CORS origins |

## Post-Deployment

1. Verify health check passes in Render dashboard
2. Test `/health` endpoint returns 200 OK
3. Test `/version` endpoint is accessible
4. Test `/evaluate` endpoint with sample requests
5. Verify CORS headers for your frontend domain

## Troubleshooting

**Build fails**: Ensure all workspace members are available
**Port binding error**: Verify no hardcoded ports in code
**CORS errors**: Update `CORS_ORIGINS` with your frontend domain
**Health check fails**: Check logs for startup errors