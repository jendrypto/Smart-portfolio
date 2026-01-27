# Smart Portfolio

A local-first cryptocurrency portfolio tracker built as a Hyperware app. Track your holdings, analyze exposure, and get AI-powered insights - all while keeping your data private and under your control.

## Features

### Holdings Tracking
- Add and manage crypto positions across multiple chains
- Track entry prices, quantities, and dates
- View real-time P&L (Profit/Loss) calculations
- Automatic price updates from multiple data sources
- Import positions directly from an EVM wallet address

### Wallet Import
- Import token positions by entering an EVM wallet address
- Scans 7 EVM chains: Ethereum, Arbitrum, Optimism, Base, Polygon, Avalanche, BNB Chain
- Automatic balance and price detection via Moralis API
- Selective import: review discovered tokens and choose which to add
- Dust filtering: small-value tokens (< $1) filtered by default
- Re-sync button to update wallet positions after initial import
- Wallet-imported positions marked with "W" badge in the positions table

### Portfolio Analytics
- **Summary Dashboard**: Total value, unrealized P&L, top positions
- **Historical Snapshots**: Daily portfolio value tracking with charts
- **Chain Exposure**: See how your portfolio is distributed across blockchains
- **Asset Exposure**: Understand your underlying exposure (ETH, BTC, Stablecoins, Other)

### Smart Insights
AI-powered portfolio analysis that provides actionable insights:
- Concentration warnings (single position risk)
- Performance feedback (gains/losses)
- 24h price movement alerts
- Chain diversification analysis
- Liquidity warnings (from DEX data)
- Portfolio health scoring (0-100)

### Risk Analysis
- **Portfolio Risk Score**: Aggregate risk assessment (0-100)
- **Correlation Matrix**: Asset correlation analysis using Pearson coefficients
- **Volatility Tracking**: 7-day and 30-day annualized volatility scores per asset
- **Drawdown Analysis**: Current and max drawdown monitoring over 30-day windows
- **Stress Testing**: Scenario-based portfolio impact simulations (crypto winter, bull run, ETH rally, stablecoin depeg)

### Actionable Recommendations
- Prioritized actions (critical/high/medium/low)
- Categories: risk mitigation, opportunities, rebalancing, alerts
- Specific rebalance/sell/buy suggestions with estimated impact

### Live News Feed
- **CryptoPanic Integration**: Real-time crypto news per token/category via the CryptoPanic Developer API v2
- Shows the 5 most recent articles in the Exposure side panel
- Each headline is clickable, opening the source article in a new tab
- Displays source, date, and community sentiment votes
- Works out of the box with a built-in fallback API key

### Multi-Source Data
Integrated with four data providers:
- **CoinGecko**: Token prices, market caps, 24h changes
- **DeFi Llama**: Chain TVL data, protocol information, historical prices
- **Dexscreener**: DEX prices, liquidity, volume for newer tokens
- **CryptoPanic**: Real-time crypto news and sentiment (Developer API v2)
- **Moralis**: EVM wallet token balances and USD prices across 7 chains

### Security & Data Integrity
- **Input Validation**: Strict bounds checking on all user inputs
- **CSV Injection Prevention**: Safe exports following RFC 4180
- **URL Encoding**: All external API queries properly encoded
- **Canonical Identifiers**: Consistent token identification across data sources

## Quick Start

### Prerequisites
- [Hyperware Kit](https://github.com/hyperware-ai/kit) installed
- Rust toolchain with `wasm32-wasip1` target
- Node.js 18+

### Build & Deploy

```bash
# Build the package
kit build

# Start a fake node for testing
kit boot-fake-node

# Deploy to the fake node
kit start-package
```

### Access the App
Once deployed, access your portfolio at:
```
http://localhost:8080/smart-portfolio:smart-portfolio:template.os/
```

## Project Structure

```
smart-portfolio/
├── smart-portfolio/        # Rust backend (WASM process)
│   └── src/
│       └── lib.rs          # Main application logic
├── ui/                     # React frontend
│   └── src/
│       ├── components/     # UI components
│       ├── store/          # Zustand state management
│       └── types/          # TypeScript type definitions
├── docs/                   # Documentation
│   ├── API.md              # API reference
│   ├── ARCHITECTURE.md     # System architecture
│   ├── SETUP.md            # Setup guide
│   └── DEVELOPMENT.md      # Development guide
├── pkg/                    # Built package output
├── CHANGELOG.md            # Version history and recent changes
└── metadata.json           # Hyperware package metadata
```

## Documentation

- [API Reference](docs/API.md) - Complete REST API documentation
- [Architecture](docs/ARCHITECTURE.md) - System design and data flow
- [Setup Guide](docs/SETUP.md) - Installation and deployment
- [Development Guide](docs/DEVELOPMENT.md) - Contributing and extending
- [Changelog](CHANGELOG.md) - Version history and recent changes

## Technology Stack

### Backend
- **Rust** - Core application logic
- **WebAssembly (WASM)** - Runs in Hyperware runtime
- **hyperware_process_lib** - Hyperware process SDK
- **serde** - Serialization/deserialization
- **rust_decimal** - Precise decimal arithmetic
- **url** - URL encoding for API queries

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type safety
- **Zustand** - State management
- **Recharts** - Portfolio charts
- **Vite** - Build tooling

### APIs
- CoinGecko API (requires API key)
- DeFi Llama API (free, no key)
- Dexscreener API (free, no key)
- CryptoPanic Developer API v2 (API key included as fallback; can be overridden via config)
- Moralis Web3 Data API (API key built-in)

## Configuration

### Package Capabilities

The package requires the following Hyperware capabilities in `pkg/manifest.json`:

```json
{
  "request_capabilities": [
    "http-server:distro:sys",
    "http-client:distro:sys",
    "vfs:distro:sys"
  ]
}
```

**Important:** The `http-client:distro:sys` capability is required for fetching live prices from CoinGecko and other external APIs. Without it, all API calls will fail silently.

### API Keys

The CoinGecko API key is configured at runtime via the config endpoint:

```bash
# Set your API key
curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/config \
  -H "Content-Type: application/json" \
  -d '{"api_key": "your-coingecko-api-key"}'

# Verify configuration
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/config
```

The key is stored in persistent state and survives restarts.
DeFi Llama and Dexscreener do not require API keys.

**CryptoPanic API Key** (optional): A built-in fallback key is included so news works immediately. To use your own key:

```bash
curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/config \
  -H "Content-Type: application/json" \
  -d '{"cryptopanic_api_key": "your-cryptopanic-api-key"}'
```

### HTTP Client Notes

When making HTTP requests in Hyperware:
- Use `hyperware_process_lib::http::client::send_request_await_response`
- Timeout is specified in **milliseconds** (not seconds): use `30000` for 30 seconds
- Use `req.query_params()` to access parsed query parameters from the request

## Privacy

Smart Portfolio is designed with privacy in mind:
- **Local-first**: All data stored locally on your Hyperware node
- **No tracking**: No analytics or telemetry
- **Your keys, your data**: You control your portfolio information

## Security

The application includes several security hardening measures:
- **Input Validation**: All user inputs are validated against maximum length limits
- **CSV Export Safety**: Exports follow RFC 4180 with CSV injection prevention
- **Canonical Identifiers**: Tokens are identified using a consistent format (`coingecko:<id>` or `address:<chain>:<addr>`) for reliable price lookups
- **Price Cache TTL**: 60-second cache prevents excessive API calls while ensuring fresh data
- **Risk Metrics Cache**: 60-second backend cache for risk analysis, eliminating 30 redundant HTTP requests per load

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read the [Development Guide](docs/DEVELOPMENT.md) before submitting PRs.
