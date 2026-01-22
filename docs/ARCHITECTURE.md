# Smart Portfolio Architecture

This document explains the system architecture, data flow, and key design decisions in Smart Portfolio.

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        HYPERWARE NODE                           │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Smart Portfolio                        │  │
│  │  ┌─────────────────┐    ┌────────────────────────────┐   │  │
│  │  │   React UI      │◄──►│     Rust WASM Backend      │   │  │
│  │  │   (Frontend)    │    │       (Process)            │   │  │
│  │  └─────────────────┘    └────────────────────────────┘   │  │
│  │           │                         │                     │  │
│  │           │                         │                     │  │
│  │           ▼                         ▼                     │  │
│  │  ┌─────────────────┐    ┌────────────────────────────┐   │  │
│  │  │  Zustand Store  │    │      HTTP Server           │   │  │
│  │  │  (State Mgmt)   │    │   (API Endpoints)          │   │  │
│  │  └─────────────────┘    └────────────────────────────┘   │  │
│  │                                     │                     │  │
│  │                                     ▼                     │  │
│  │                         ┌────────────────────────────┐   │  │
│  │                         │   Persistent State         │   │  │
│  │                         │   (bincode serialized)     │   │  │
│  │                         └────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    ▼              ▼              ▼
            ┌───────────┐  ┌───────────┐  ┌───────────┐
            │ CoinGecko │  │DeFi Llama │  │Dexscreener│
            │    API    │  │    API    │  │    API    │
            └───────────┘  └───────────┘  └───────────┘
```

## Component Architecture

### Backend (Rust WASM Process)

Located in `smart-portfolio/src/lib.rs`, the backend handles:

```
lib.rs
├── Domain Types
│   ├── Position          - User's token holdings
│   ├── CanonicalId       - Consistent token identifier
│   ├── IdentifierKind    - Coingecko | Address | Symbol
│   ├── PriceData         - Cached price information
│   ├── DailySnapshot     - Historical portfolio values
│   ├── ExposureEntry     - Asset category exposure
│   └── Insight           - Generated portfolio insights
│
├── State Management
│   └── AppState          - Serialized persistent state
│
├── API Integrations
│   ├── CoinGecko         - Price feeds, token search
│   ├── DeFi Llama        - Chain TVL data
│   └── Dexscreener       - DEX prices, liquidity
│
├── Analytics Engine
│   ├── calculate_positions_with_derived()
│   ├── calculate_portfolio_summary()
│   ├── calculate_exposure()
│   └── generate_insights()
│
├── HTTP Handlers
│   ├── handle_get_holdings()
│   ├── handle_add_position()
│   ├── handle_get_insights()
│   └── ... (13 handlers total)
│
└── Main Loop
    └── Process incoming HTTP requests
```

### Frontend (React + TypeScript)

Located in `ui/src/`, the frontend is structured as:

```
ui/src/
├── App.tsx               - Main app with tab navigation
├── App.css               - Global styles (glass-morphism)
│
├── components/
│   ├── HoldingsTab.tsx   - Holdings view container
│   ├── ExposureTab.tsx   - Exposure view container
│   ├── SummaryPanel.tsx  - Dashboard with chart & stats
│   ├── PositionsTable.tsx- Position list with actions
│   ├── PortfolioChart.tsx- Line chart for history
│   ├── InsightCards.tsx  - Dynamic insight display
│   └── AddPositionModal.tsx - Position entry form
│
├── store/
│   └── portfolio.ts      - Zustand state management
│
└── types/
    └── Portfolio.ts      - TypeScript interfaces
```

## Data Flow

### 1. Position Lifecycle

```
User Action          Frontend                 Backend                External API
    │                   │                       │                        │
    │  Add Position     │                       │                        │
    ├──────────────────►│  POST /api/positions  │                        │
    │                   ├──────────────────────►│                        │
    │                   │                       │  Fetch price           │
    │                   │                       ├───────────────────────►│
    │                   │                       │◄───────────────────────┤
    │                   │                       │  Store position        │
    │                   │                       │  + price in state      │
    │                   │◄──────────────────────┤                        │
    │                   │  Update Zustand store │                        │
    │◄──────────────────┤  Re-render UI         │                        │
```

### 2. Price Update Flow

```
User clicks "Refresh"
        │
        ▼
POST /api/refresh
        │
        ▼
┌───────────────────────┐
│ Collect all CoinGecko │
│ IDs from positions    │
└───────────────────────┘
        │
        ▼
┌───────────────────────┐
│ Batch fetch prices    │
│ from CoinGecko API    │
└───────────────────────┘
        │
        ▼
┌───────────────────────┐
│ Update price cache    │
│ in AppState           │
└───────────────────────┘
        │
        ▼
┌───────────────────────┐
│ Create daily snapshot │
│ if none exists today  │
└───────────────────────┘
        │
        ▼
┌───────────────────────┐
│ Save state to disk    │
└───────────────────────┘
```

### 3. Insight Generation Flow

```
GET /api/insights
        │
        ▼
┌─────────────────────────┐
│ calculate_positions_    │
│ with_derived()          │
│ - Current prices        │
│ - P&L calculations      │
│ - Allocation %          │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│ calculate_portfolio_    │
│ summary()               │
│ - Total value           │
│ - Top positions         │
│ - Chain summary         │
│ - Biggest mover         │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│ calculate_exposure()    │
│ - ETH/BTC/Stable/Other  │
│ - Chain breakdown       │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│ generate_insights()     │
│ - Concentration         │
│ - Performance           │
│ - Price movements       │
│ - Diversification       │
│ - Liquidity warnings    │
│ - Chain TVL analysis    │
│ - Volume insights       │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│ Sort by priority,       │
│ limit to 6, calculate   │
│ health score            │
└─────────────────────────┘
```

## State Management

### Backend State (Rust)

```rust
struct AppState {
    // Core data
    positions: HashMap<String, Position>,  // User's holdings
    prices: HashMap<String, PriceData>,    // Cached prices (keyed by canonical ID)
    snapshots: Vec<DailySnapshot>,         // Historical values

    // Market data
    chain_tvl: HashMap<String, ChainTvlData>,  // DeFi Llama data
    dex_pairs: HashMap<String, Vec<DexPairData>>,  // Dexscreener data

    // Timestamps
    last_price_fetch: u64,
    last_tvl_fetch: u64,
}

struct Position {
    id: String,
    token_identifier: Option<String>,  // Legacy CoinGecko ID
    canonical_id: Option<CanonicalId>, // Preferred canonical identifier
    token_symbol: String,
    token_name: String,
    chain: String,
    quantity: Decimal,
    entry_price_usd: Decimal,
    // ...
}

struct CanonicalId {
    canonical: String,            // e.g., "coingecko:bitcoin"
    kind: IdentifierKind,         // Coingecko | Address | Symbol
    raw_input: String,            // Original user input
    coingecko_id: Option<String>, // CoinGecko API ID if resolved
    address: Option<String>,      // Normalized address if applicable
    chain: Option<String>,        // Chain for address-based IDs
}
```

State is serialized using `bincode` and persisted via Hyperware's `set_state()`.

**Migration:** On startup, existing positions without `canonical_id` are automatically migrated.

### Frontend State (Zustand)

```typescript
interface PortfolioStore {
  // Data mirrors
  positions: PositionWithDerived[];
  summary: PortfolioSummary | null;
  snapshots: DailySnapshot[];
  exposure: ExposureResponse | null;
  insights: Insight[];
  marketData: MarketDataResponse | null;

  // UI state
  activeTab: 'holdings' | 'exposure';
  isLoading: boolean;
  error: string | null;
  sortField: SortField;
  sortDirection: SortDirection;
  isAddModalOpen: boolean;

  // Actions
  fetchHoldings(): Promise<void>;
  fetchInsights(): Promise<void>;
  addPosition(pos: AddPositionRequest): Promise<boolean>;
  // ... more actions
}
```

## Key Design Decisions

### 1. Local-First Architecture

All data is stored locally on the user's Hyperware node:
- No external database dependencies
- Complete data ownership
- Works offline (except for price updates)

### 2. Multi-Source Pricing

```
Token Price Resolution:
1. Check canonical ID in price cache (preferred)
2. Check CoinGecko ID in price cache (legacy fallback)
3. If stablecoin → return $1.00
4. Fallback to entry price
5. DEX data available via separate endpoint
```

This ensures users always see *some* price, even if APIs fail.

### 3. Canonical Identifier System

All tokens are identified using a canonical format for consistent lookups:

```
Canonical ID Formats:
┌─────────────────────────────────────────────────────────┐
│ coingecko:bitcoin     → CoinGecko API ID               │
│ coingecko:ethereum    → CoinGecko API ID               │
│ address:ethereum:0x...→ On-chain contract address      │
│ symbol:XYZ            → Unresolved symbol (placeholder) │
└─────────────────────────────────────────────────────────┘

Resolution Logic:
1. If input looks like 0x address → address:<chain>:<addr>
2. If CoinGecko ID provided → coingecko:<id>
3. Otherwise → symbol:<SYMBOL>
```

Benefits:
- Consistent price cache keys across all data sources
- Addresses normalized to lowercase
- Backwards compatible with legacy token_identifier field
- Migration runs automatically on startup

### 3. Exposure Classification

```
Token → Exposure Category:
┌────────────────────────────────────────┐
│ ETH, WETH, stETH, rETH → ETH           │
│ BTC, WBTC             → BTC            │
│ USDC, USDT, DAI, etc. → Stablecoins    │
│ Everything else       → Other          │
└────────────────────────────────────────┘
```

Confidence levels indicate how certain we are about the classification.

### 4. Insight Prioritization

Insights are ranked by:
1. **Priority**: High > Medium > Low
2. **Relevance**: Only generated if conditions met
3. **Limit**: Max 6 insights to avoid overwhelm

Health score (0-100) provides quick portfolio assessment.

### 5. Glass-Morphism UI

The UI uses a modern glass-morphism design:
- Semi-transparent backgrounds
- Backdrop blur effects
- Gradient accents
- Dark theme optimized

## Security Considerations

### API Key Storage
- CoinGecko API key stored in source code (acceptable for demo keys)
- For production: consider environment variables or secure storage

### Input Validation
All user inputs are validated against strict bounds:

```
┌─────────────────┬─────────────┐
│ Field           │ Max Length  │
├─────────────────┼─────────────┤
│ Token Symbol    │ 20 chars    │
│ Token Name      │ 100 chars   │
│ Chain           │ 50 chars    │
│ User Note       │ 1000 chars  │
│ Single Tag      │ 50 chars    │
│ Tags Count      │ 20 tags     │
└─────────────────┴─────────────┘
```

Numeric validation:
- All numeric inputs parsed as `Decimal` for precision
- Quantity must be > 0
- Entry price must be >= 0

### URL Encoding
All external API queries are URL-encoded to prevent:
- Query injection attacks
- Malformed requests from special characters
- API errors from unencoded symbols (e.g., "USD+", "ETH/BTC")

### CSV Export Safety
CSV exports follow RFC 4180 with additional protections:
- Fields containing commas, quotes, or newlines are properly escaped
- Formula-triggering characters (`=`, `+`, `-`, `@`, `\t`, `\r`) are prefixed with `'`
- Prevents CSV injection attacks in spreadsheet applications

### HTTP Security
- No authentication (local app assumption)
- All requests validated before processing
- No SQL (no injection risk)

## Performance Optimizations

### Caching Strategy
```
┌──────────────┬─────────────┬───────────────────────┐
│ Data Type    │ Cache TTL   │ Invalidation          │
├──────────────┼─────────────┼───────────────────────┤
│ Prices       │ 60 seconds  │ POST /api/refresh     │
│ Chain TVL    │ 5 minutes   │ POST /market/tvl      │
│ Snapshots    │ 1 day       │ Auto-created daily    │
└──────────────┴─────────────┴───────────────────────┘
```

Price cache TTL is enforced on the `/api/refresh` endpoint:
- If prices were fetched less than 60 seconds ago, returns cached data
- Response includes `cached: true` and `next_refresh_in` seconds
- Prevents excessive API calls while ensuring data freshness

### Batch Operations
- Price fetches batched by CoinGecko ID
- Positions calculated in single pass
- Snapshots limited to 365 entries

## Extensibility Points

The code includes marked extension points:

```rust
// EXTENSION POINT: Future wallet import integration
// fn import_from_wallet(wallet_address: &str, chain: &str) -> Vec<Position>

// EXTENSION POINT: Future advanced exposure mapping
// fn get_protocol_exposure(position: &Position) -> Vec<ProtocolExposure>

// EXTENSION POINT: Future AI narrative summaries
// fn generate_portfolio_narrative(summary: &PortfolioSummary) -> String

// EXTENSION POINT: Future historical exposure snapshots
// fn store_exposure_snapshot(exposure: &ExposureResponse)
```

These can be implemented to add:
- Automatic wallet scanning
- DeFi protocol decomposition
- LLM-generated summaries
- Historical exposure tracking
