# Smart Portfolio Setup Guide

Complete guide to installing, building, and deploying Smart Portfolio.

## Prerequisites

### 1. Hyperware Kit

Install the Hyperware development toolkit:

```bash
# Check if kit is installed
kit --version

# If not installed, follow instructions at:
# https://github.com/hyperware-ai/kit
```

### 2. Rust Toolchain

Install Rust with WASM target:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-wasip1

# Verify installation
rustc --version
```

### 3. Node.js

Install Node.js 18 or later:

```bash
# Check version
node --version  # Should be 18.x or higher
npm --version
```

### 4. CoinGecko API Key (Optional but Recommended)

Get a free API key from [CoinGecko](https://www.coingecko.com/api/pricing):

1. Create an account at CoinGecko
2. Go to API section
3. Generate a Demo API key
4. Note: Free tier allows ~30 calls/minute

### 5. CryptoPanic API Key (Optional)

A built-in fallback key is included, so news works out of the box. To use your own key:

1. Create an account at [CryptoPanic](https://cryptopanic.com)
2. Go to API section and get a Developer key
3. Configure after deployment (see below)

---

## Installation

### Clone/Download the Project

```bash
# If from a repository
git clone <repository-url>
cd smart-portfolio

# Or if you have the source files, navigate to the directory
cd /path/to/smart-portfolio
```

### Configure API Key

After deploying the app, set your CoinGecko API key via the config endpoint:

```bash
curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/config \
  -H "Content-Type: application/json" \
  -d '{"api_key": "your-coingecko-api-key"}'
```

Verify the key is configured:

```bash
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/config
# Returns: {"api_key_configured": true, "api_key_masked": "your...here"}
```

The API key is stored in the app's persistent state and survives restarts.

### Configure CryptoPanic API Key (Optional)

News works out of the box with a built-in fallback key. To use your own:

```bash
curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/config \
  -H "Content-Type: application/json" \
  -d '{"cryptopanic_api_key": "your-cryptopanic-api-key"}'
```

### Install Frontend Dependencies

```bash
cd ui
npm install
cd ..
```

---

## Building

### Build the Complete Package

```bash
kit build
```

This command:
1. Builds the React frontend (`npm run build`)
2. Compiles the Rust WASM process
3. Creates the package in `pkg/`

**Expected output:**
```
Building UI in "ui"...
Running npm install...
Running npm run build:copy...
Done building UI.
Compiling Rust Hyperware process...
warning: unused variable... (warnings are OK)
Finished release profile
Done compiling Rust Hyperware process.
package zip hash: <hash>
```

### Build Only Frontend (Development)

```bash
cd ui
npm run build
```

### Build Only Backend

```bash
cd smart-portfolio
cargo build --release --target wasm32-wasip1
```

---

## Deployment

### Option 1: Local Fake Node (Development/Testing)

Start a fake Hyperware node for testing:

```bash
# Terminal 1: Start the fake node
kit boot-fake-node

# Terminal 2: Deploy the package
kit start-package
```

**Access the app:**
```
http://localhost:8080/smart-portfolio:smart-portfolio:template.os/
```

### Option 2: Deploy to Real Node

```bash
# Deploy to a running Hyperware node
kit start-package --url http://your-node-address:port
```

### Option 3: Export Package

Create a distributable package:

```bash
kit build

# Package will be in pkg/ directory
ls pkg/
# smart-portfolio.zip
```

---

## Verification

### 1. Check the UI Loads

Navigate to the app URL. You should see:
- Dark background with gradient effects
- "Smart Portfolio" header
- Holdings/Exposure tabs
- "No positions yet" message (if empty)

### 2. Test API Endpoints

```bash
# Test holdings endpoint
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/holdings

# Expected: JSON with positions, summary, snapshots
```

### 3. Add a Test Position

Using the UI:
1. Click "Add Position"
2. Search for "Bitcoin"
3. Select "bitcoin" from results
4. Enter quantity and entry price
5. Click "Add Position"

Or via API:
```bash
curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/positions \
  -H "Content-Type: application/json" \
  -d '{
    "token_identifier": "bitcoin",
    "token_symbol": "BTC",
    "token_name": "Bitcoin",
    "chain": "Bitcoin",
    "quantity": "0.1",
    "entry_price_usd": "50000"
  }'
```

---

## Configuration Options

### Package Metadata

Edit `metadata.json` to customize:

```json
{
  "name": "Smart Portfolio",
  "description": "Local-first crypto portfolio tracker",
  "image": "",
  "properties": {
    "package_name": "smart-portfolio",
    "current_version": "0.1.0",
    "publisher": "template.os",
    "mirrors": [],
    "code_hashes": {}
  }
}
```

### UI Configuration

Edit `ui/vite.config.ts` for base URL:

```typescript
export default defineConfig({
  base: '/smart-portfolio:smart-portfolio:template.os/',
  // ...
});
```

### Backend Constants

In `smart-portfolio/src/lib.rs`:

```rust
// Cache duration for prices (seconds)
const COINGECKO_CACHE_TTL_SECS: u64 = 60;

// API endpoints
const DEFILLAMA_API_BASE: &str = "https://api.llama.fi";
const DEXSCREENER_API_BASE: &str = "https://api.dexscreener.com/latest/dex";
```

---

## Troubleshooting

### Build Errors

**Problem:** WASM target not found
```
error: could not compile `smart-portfolio`
```
**Solution:**
```bash
rustup target add wasm32-wasip1
```

**Problem:** Node modules issues
```
npm ERR! ...
```
**Solution:**
```bash
cd ui
rm -rf node_modules package-lock.json
npm install
```

### Runtime Errors

**Problem:** "Failed to serve UI"
- Check that `ui/dist/` exists after build
- Verify `pkg/` contains the UI files

**Problem:** API returns 404
- Check the URL path includes the full package path
- Verify the process is running: `kit ps`

**Problem:** Prices not updating / Token search returns empty
- Check CoinGecko API key is valid
- Check API rate limits (30/min for free tier)
- Look for errors in console output
- Check terminal logs for: `doesn't have capability to message process http-client:distro:sys`

**Problem:** "doesn't have capability to message process http-client:distro:sys"

This error occurs when the package cannot make outbound HTTP requests. Fix by ensuring `pkg/manifest.json` includes the http-client capability:

```json
{
  "request_capabilities": [
    "http-server:distro:sys",
    "http-client:distro:sys",
    "vfs:distro:sys"
  ]
}
```

After updating, rebuild and redeploy:
```bash
kit build
kit start-package
```

**Problem:** HTTP requests timeout or fail silently

The HTTP client uses milliseconds for timeouts, not seconds. If you're making custom HTTP requests, use:
```rust
// Correct: 30 seconds = 30000 milliseconds
send_request_await_response(Method::GET, url, None, 30000, vec![])

// Wrong: This is only 30 milliseconds!
send_request_await_response(Method::GET, url, None, 30, vec![])
```

### Connection Issues

**Problem:** Can't connect to fake node
- Ensure `kit boot-fake-node` is running
- Check port 8080 is not in use
- Try: `lsof -i :8080`

**Problem:** Package not found
```
http://localhost:8080/smart-portfolio:smart-portfolio:template.os/
# Returns 404
```
**Solution:**
```bash
# Restart package
kit start-package

# Or check running processes
kit ps
```

---

## Updating

### Update Dependencies

```bash
# Frontend
cd ui
npm update

# Backend
cd smart-portfolio
cargo update
```

### Rebuild After Changes

Always rebuild after code changes:

```bash
kit build
kit start-package
```

### Clear State (Development)

To reset all data:

```bash
# Stop the node
# Delete state directory (location depends on setup)
# Restart node
kit boot-fake-node
kit start-package
```

---

## Production Considerations

### Security

1. **API Keys**: Configured at runtime via `POST /api/config` and stored in persistent state. Never hardcoded in source.
2. **HTTPS**: Use a reverse proxy for production
3. **Updates**: Keep dependencies updated

### Performance

1. **Price Caching**: Adjust `COINGECKO_CACHE_TTL_SECS` as needed
2. **Snapshot Limits**: Currently keeps 365 days of snapshots
3. **Position Limits**: No hard limit, but UI may slow with 100+ positions

### Backup

Export your data regularly:
- `GET /api/export/positions` - Download positions CSV
- `GET /api/export/snapshots` - Download history CSV
