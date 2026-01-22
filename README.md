# Smart Portfolio

A local-first cryptocurrency portfolio tracker built as a Hyperware app. Track your holdings, analyze exposure, and get AI-powered insights - all while keeping your data private and under your control.

## Features

### Holdings Tracking
- Add and manage crypto positions across multiple chains
- Track entry prices, quantities, and dates
- View real-time P&L (Profit/Loss) calculations
- Automatic price updates from multiple data sources

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

### Multi-Source Data
Integrated with three major data providers:
- **CoinGecko**: Token prices, market caps, 24h changes
- **DeFi Llama**: Chain TVL data, protocol information
- **Dexscreener**: DEX prices, liquidity, volume for newer tokens

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
└── metadata.json           # Hyperware package metadata
```

## Documentation

- [API Reference](docs/API.md) - Complete REST API documentation
- [Architecture](docs/ARCHITECTURE.md) - System design and data flow
- [Setup Guide](docs/SETUP.md) - Installation and deployment
- [Development Guide](docs/DEVELOPMENT.md) - Contributing and extending

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

## Configuration

### API Keys
The CoinGecko API key is configured in `smart-portfolio/src/lib.rs`:
```rust
const COINGECKO_API_KEY: &str = "your-api-key-here";
```

DeFi Llama and Dexscreener do not require API keys.

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

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read the [Development Guide](docs/DEVELOPMENT.md) before submitting PRs.
