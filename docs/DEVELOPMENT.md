# Smart Portfolio Development Guide

Guide for developers who want to extend, modify, or contribute to Smart Portfolio.

## Development Setup

### Quick Start

```bash
# 1. Clone and enter project
cd smart-portfolio

# 2. Install frontend dependencies
cd ui && npm install && cd ..

# 3. Start development mode
# Terminal 1: Run fake node
kit boot-fake-node

# Terminal 2: Build and deploy
kit build && kit start-package

# Terminal 3: Frontend dev server (optional, for hot reload)
cd ui && npm run dev
```

### Frontend Hot Reload

For faster UI development:

```bash
cd ui
npm run dev
```

This starts Vite dev server at `http://localhost:5173` with hot module replacement.

**Note:** API calls will fail unless proxied to the running Hyperware node.

---

## Code Organization

### Backend Structure

```
smart-portfolio/src/lib.rs
├── Imports and URL encoding helpers
├── Domain Types
│   ├── Positions (Position, CanonicalId, IdentifierKind)
│   ├── Pricing (PriceData, PriceSource)
│   ├── Chain/Protocol Data (ChainTvlData, ProtocolData)
│   ├── DEX Data (DexPairData)
│   ├── Snapshots (DailySnapshot)
│   ├── Analytics / Exposure (ExposureCategory, ExposureEntry, ChainExposure)
│   ├── Insights (InsightType, InsightPriority, Insight)
│   ├── Risk Metrics (CorrelationMatrix, VolatilityScore, DrawdownData)
│   ├── Recommendations (ActionableRecommendation, RecommendedAction)
│   └── Scenarios (Scenario, AssetImpact)
├── API Request / Response Types
├── Configuration and State
│   ├── Constants (cache TTLs, API URLs, validation limits)
│   ├── AppState struct (positions, prices, snapshots, config)
│   └── State persistence (load_state, save_state, migrate)
├── Demo Portfolio Generator
├── External API Integrations
│   ├── CoinGecko (search, prices, top tokens)
│   ├── DeFi Llama (chain TVL, token prices, historical prices)
│   ├── Dexscreener (pair search, token price)
│   └── Moralis Web3 API (wallet token balances + prices)
├── Risk Metrics Calculation
│   ├── Historical price fetching
│   ├── Returns and volatility calculation
│   ├── Pearson correlation
│   └── Drawdown analysis
├── Recommendations Engine
├── Stress Testing Scenarios
├── Analytics (positions, summary, insights, exposure)
├── HTTP Response Helpers
├── Route Handlers (handle_* functions)
├── Configuration Handlers (config get/set)
├── Wallet Handlers (scan, import, resync, status)
├── HTTP Request Router
└── Main Loop (init, server setup, message loop)
```

### Frontend Structure

```
ui/src/
├── App.tsx           # Root component, tab navigation
├── App.css           # Global styles (600+ lines)
│
├── components/
│   ├── HoldingsTab.tsx      # Holdings container
│   ├── ExposureTab.tsx      # Exposure container
│   ├── SummaryPanel.tsx     # Dashboard layout
│   ├── PositionsTable.tsx   # Position list
│   ├── PortfolioChart.tsx   # Recharts line graph
│   ├── InsightCards.tsx     # Insight display
│   ├── AddPositionModal.tsx    # Position form
│   ├── EditPositionModal.tsx  # Position editing form
│   ├── WalletImportModal.tsx  # Wallet scan/import modal
│   ├── TickerBanner.tsx       # Price ticker banner
│   └── InfoFAB.tsx            # Info floating action button
│
├── store/
│   └── portfolio.ts   # Zustand store (all state + actions)
│
└── types/
    └── Portfolio.ts   # TypeScript interfaces
```

---

## Common Development Tasks

### Adding a New API Endpoint

**1. Define types (if needed)**

```rust
// In lib.rs, add to API Response Types section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyNewResponse {
    pub field1: String,
    pub field2: f64,
}
```

**2. Create handler function**

```rust
fn handle_my_endpoint(state: &AppState) {
    let response = MyNewResponse {
        field1: "value".to_string(),
        field2: 42.0,
    };
    send_json_response(StatusCode::OK, response);
}
```

**3. Add route to router**

```rust
// In handle_http_request()
match (method, path_parts.as_slice()) {
    // ... existing routes ...
    (Method::GET, ["api", "my-endpoint"]) => handle_my_endpoint(state),
    // ...
}
```

**4. Register path in init()**

```rust
let api_paths = [
    // ... existing paths ...
    "/api/my-endpoint",
];
```

**5. Add frontend types**

```typescript
// In ui/src/types/Portfolio.ts
export interface MyNewResponse {
  field1: string;
  field2: number;
}
```

**6. Add store action**

```typescript
// In ui/src/store/portfolio.ts
interface PortfolioStore {
  // ... existing ...
  fetchMyEndpoint: () => Promise<void>;
}

// In create() implementation:
fetchMyEndpoint: async () => {
  const response = await fetch(`${BASE_URL}/api/my-endpoint`);
  const data: MyNewResponse = await response.json();
  // Update state as needed
},
```

### Adding a New Insight Type

**1. Add to InsightType enum**

```rust
pub enum InsightType {
    // ... existing ...
    MyNewInsight,  // Add here
}
```

**2. Generate the insight in generate_insights()**

```rust
// In generate_insights(), add before SORT AND LIMIT section:

// -------------------------------------------------------------------------
// X. MY NEW INSIGHT
// -------------------------------------------------------------------------

if /* condition for insight */ {
    insights.push(Insight {
        id: "my_new_insight".to_string(),
        insight_type: InsightType::MyNewInsight,
        priority: InsightPriority::Medium,
        title: "My Insight Title".to_string(),
        description: "Detailed description...".to_string(),
        icon: "🔍".to_string(),
        color: "blue".to_string(),
        action_label: None,
        action_url: None,
        metadata: HashMap::new(),
    });
    health_score -= 5;  // Adjust health score if appropriate
}
```

**3. Update frontend type**

```typescript
// In ui/src/types/Portfolio.ts
export type InsightType =
  | 'Concentration'
  // ... existing ...
  | 'MyNewInsight';  // Add here
```

### Adding a New External API

**1. Add constants**

```rust
const MY_API_BASE: &str = "https://api.example.com";
```

**2. Define response types**

```rust
#[derive(Debug, Deserialize)]
struct MyApiResponse {
    data: Vec<MyApiData>,
}

#[derive(Debug, Deserialize)]
struct MyApiData {
    field: String,
}
```

**3. Create fetch function**

```rust
fn fetch_from_my_api(params: &str) -> Vec<MyApiData> {
    let url = format!("{}/endpoint?param={}", MY_API_BASE, params);

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30,  // timeout seconds
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let data: MyApiResponse =
                            serde_json::from_slice(response.body())
                            .unwrap_or(MyApiResponse { data: vec![] });
                        data.data
                    } else {
                        println!("API error: {}", response.status());
                        vec![]
                    }
                }
                Err(e) => {
                    println!("HTTP error: {:?}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            println!("URL parse error: {:?}", e);
            vec![]
        }
    }
}
```

### Adding a New UI Component

**1. Create component file**

```typescript
// ui/src/components/MyComponent.tsx
import usePortfolioStore from '../store/portfolio';

function MyComponent() {
  const { someData } = usePortfolioStore();

  return (
    <div className="my-component glass-card">
      <h3>My Component</h3>
      <p>{someData}</p>
    </div>
  );
}

export default MyComponent;
```

**2. Add styles**

```css
/* In ui/src/App.css */

/* My Component */
.my-component {
  padding: var(--spacing-lg);
  margin-bottom: var(--spacing-lg);
}

.my-component h3 {
  color: var(--text-primary);
  margin-bottom: var(--spacing-md);
}
```

**3. Use in parent component**

```typescript
import MyComponent from './MyComponent';

// In render:
<MyComponent />
```

### Adding a New Risk Metric

1. Define the metric type in the "Risk Metrics" domain types section
2. Add calculation logic in the risk metrics section (near `calculate_risk_metrics`)
3. Include in `PortfolioRiskMetrics` response struct
4. Add TypeScript interface in `ui/src/types/Portfolio.ts`
5. Update the `PortfolioRiskMetrics` interface to include new field

### Adding a New Stress Test Scenario

1. Add scenario definition in `calculate_scenarios()` function's `scenario_definitions` vec
2. Define per-category impact percentages (ETH, BTC, Alt L1s, Protocol Tokens, Stablecoins, Other)
3. No frontend changes needed - scenarios are rendered dynamically

---

## Security Patterns

### API Key Configuration

API keys are configured at runtime via the `/api/config` endpoint and stored in
Hyperware's persistent process state.

- **CoinGecko**: Set via `POST /api/config` with `{"api_key": "your-key"}`; no hardcoded fallback
- **CryptoPanic**: Set via `POST /api/config` with `{"cryptopanic_api_key": "your-key"}`; a hardcoded fallback key (`CRYPTOPANIC_FALLBACK_KEY`) is included so news works out of the box
- **Moralis**: Hardcoded `MORALIS_API_KEY` constant in the backend; no runtime configuration needed
- Check: `GET /api/config` returns masked key status
- Keys persist across restarts via bincode-serialized state

### Input Validation

All user inputs should be validated against defined limits:

```rust
// Validation constants
const MAX_SYMBOL_LENGTH: usize = 20;
const MAX_NAME_LENGTH: usize = 100;
const MAX_CHAIN_LENGTH: usize = 50;
const MAX_NOTE_LENGTH: usize = 1000;
const MAX_TAG_LENGTH: usize = 50;
const MAX_TAGS_COUNT: usize = 20;

// Validation function pattern
fn validate_input(request: &AddPositionRequest) -> Result<(), &'static str> {
    if request.token_symbol.len() > MAX_SYMBOL_LENGTH {
        return Err("Token symbol too long (max 20 characters)");
    }
    // ... more checks
    Ok(())
}

// Usage in handler
fn handle_add_position(state: &mut AppState, body: &[u8]) {
    let request: AddPositionRequest = serde_json::from_slice(body)?;

    if let Err(msg) = validate_input(&request) {
        return send_error_response(StatusCode::BAD_REQUEST, msg);
    }

    // Proceed with valid input
}
```

### URL Encoding for External APIs

Always URL-encode user input before using in external API URLs:

```rust
fn url_encode(input: &str) -> String {
    url::form_urlencoded::byte_serialize(input.as_bytes()).collect()
}

// Usage
let url = format!(
    "https://api.coingecko.com/api/v3/search?query={}&x_cg_demo_api_key={}",
    url_encode(query),  // Encode user input
    COINGECKO_API_KEY   // API key doesn't need encoding
);
```

This prevents:
- Query injection attacks
- Malformed requests from special characters (e.g., `+`, `/`, `&`)
- API errors from unencoded symbols

### HTTP Client Configuration

To make outbound HTTP requests (e.g., to CoinGecko, DeFi Llama), the package requires the `http-client:distro:sys` capability in `pkg/manifest.json`:

```json
{
  "request_capabilities": [
    "http-server:distro:sys",
    "http-client:distro:sys",
    "vfs:distro:sys"
  ]
}
```

**Important notes about the HTTP client:**

1. **Timeout is in milliseconds**, not seconds:
```rust
// Correct: 30 seconds
send_request_await_response(Method::GET, url, None, 30000, vec![])

// Wrong: Only 30 milliseconds!
send_request_await_response(Method::GET, url, None, 30, vec![])
```

2. **Use Hyperware's query_params()** for accessing query parameters:
```rust
fn handle_http_request(state: &mut AppState, req: &IncomingHttpRequest, body: &[u8]) {
    // Use Hyperware's built-in query parameter parsing
    let query_params = req.query_params();

    // Access parameters directly
    let search_query = query_params.get("q").map(|s| s.as_str()).unwrap_or("");
}
```

3. **Debug endpoint** available at `/api/debug/http` to test outbound connectivity.

### CSV Export Safety

When exporting user data to CSV, follow RFC 4180 with CSV injection prevention:

```rust
fn escape_csv_field(field: &str) -> String {
    // Prefix formula-triggering characters to prevent injection
    let sanitized = if field.starts_with('=')
        || field.starts_with('+')
        || field.starts_with('-')
        || field.starts_with('@')
        || field.starts_with('\t')
        || field.starts_with('\r')
    {
        format!("'{}", field)  // Prefix with single quote
    } else {
        field.to_string()
    };

    // RFC 4180 escaping
    let needs_escape = sanitized.contains(',')
        || sanitized.contains('"')
        || sanitized.contains('\n')
        || sanitized.contains('\r');

    if needs_escape {
        format!("\"{}\"", sanitized.replace('"', "\"\""))
    } else {
        sanitized
    }
}

// Usage in CSV export
csv.push_str(&format!(
    "{},{},{}\n",
    escape_csv_field(&position.token_symbol),
    escape_csv_field(&position.token_name),
    escape_csv_field(&position.user_note.clone().unwrap_or_default())
));
```

### Canonical Identifier System

Use canonical IDs for consistent token identification:

```rust
// Types
pub enum IdentifierKind {
    Coingecko,  // CoinGecko API identifier
    Address,    // On-chain contract address
    Symbol,     // Unresolved symbol (placeholder)
}

pub struct CanonicalId {
    pub canonical: String,            // e.g., "coingecko:bitcoin"
    pub kind: IdentifierKind,
    pub raw_input: String,            // Original user input
    pub coingecko_id: Option<String>,
    pub address: Option<String>,
    pub chain: Option<String>,
}

// Constructors
CanonicalId::from_coingecko("bitcoin", "bitcoin")
// → canonical: "coingecko:bitcoin"

CanonicalId::from_address("0xabc...", "ethereum", "0xabc...")
// → canonical: "address:ethereum:0xabc..."

CanonicalId::from_symbol("XYZ")
// → canonical: "symbol:XYZ"
```

**Resolution Logic:**

```rust
fn resolve_canonical_id(
    token_identifier: Option<&str>,
    token_symbol: &str,
    chain: &str,
) -> CanonicalId {
    if let Some(identifier) = token_identifier {
        // If it looks like an address (0x + 40 hex chars)
        if is_eth_address(identifier) {
            return CanonicalId::from_address(identifier, chain, identifier);
        }
        // Otherwise, treat as CoinGecko ID
        return CanonicalId::from_coingecko(identifier, identifier);
    }
    // No identifier provided - use symbol as unresolved
    CanonicalId::from_symbol(token_symbol)
}
```

**Price Lookup with Canonical IDs:**

```rust
fn get_position_price(position: &Position, state: &AppState) -> f64 {
    // 1. Try canonical ID first (preferred)
    if let Some(ref canonical) = position.canonical_id {
        if let Some(price_data) = state.prices.get(&canonical.canonical) {
            return price_data.price_usd;
        }
    }

    // 2. Legacy fallback: check old token_identifier field
    if let Some(ref cg_id) = position.token_identifier {
        if let Some(price_data) = state.prices.get(cg_id) {
            return price_data.price_usd;
        }
    }

    // 3. Stablecoin fallback
    if is_stablecoin(&position.token_symbol) {
        return 1.0;
    }

    // 4. Default to entry price
    position.entry_price_usd.to_f64().unwrap_or(0.0)
}
```

### Cache TTL Implementation

Enforce cache TTL on refresh endpoints:

```rust
fn handle_refresh_prices(state: &mut AppState) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Skip refresh if cache is still fresh
    if state.last_price_fetch > 0 && now - state.last_price_fetch < COINGECKO_CACHE_TTL_SECS {
        send_json_response(StatusCode::OK, serde_json::json!({
            "message": "Prices still fresh",
            "cached": true,
            "next_refresh_in": COINGECKO_CACHE_TTL_SECS - (now - state.last_price_fetch)
        }));
        return;
    }

    // Proceed with actual refresh
    // ...

    // Update timestamp after successful fetch
    state.last_price_fetch = now;
}
```

### State Migration

When adding new fields to existing structs, implement migrations:

```rust
fn migrate_positions_to_canonical(state: &mut AppState) {
    let mut migrated = false;

    for position in state.positions.values_mut() {
        // Only migrate if field is missing
        if position.canonical_id.is_none() {
            position.canonical_id = Some(resolve_canonical_id(
                position.token_identifier.as_deref(),
                &position.token_symbol,
                &position.chain,
            ));
            migrated = true;
        }
    }

    if migrated {
        save_state(state);
        println!("smart-portfolio: migrated positions to canonical IDs");
    }
}

// Call in init() after loading state
fn init(our: Address) {
    let mut state = load_state().unwrap_or_default();
    migrate_positions_to_canonical(&mut state);
    // ...
}
```

**Migration Guidelines:**
- The `AppState` struct uses `#[serde(default)]` at the struct level, so new fields automatically default when deserializing old state
- Make new fields `Option<T>` or use types that implement `Default` for backwards compatibility
- Check for `None` before migrating
- Only save state if actually migrated
- Log migration for debugging
- Call migration in `init()` after loading state

**Important:** Without `#[serde(default)]`, adding new fields to `AppState` will cause old serialized state (bincode) to fail deserialization. The `load_state()` fallback resets ALL state, which loses existing data. Always keep the `#[serde(default)]` attribute.

---

## Styling Guidelines

### CSS Variables

Use the defined CSS variables for consistency:

```css
/* Colors */
--primary: #D9FD65;         /* Lime green */
--secondary: #0F52FF;       /* Blue */
--bg-dark: #0A0A0F;         /* Background */
--glass-bg: rgba(255, 255, 255, 0.03);

/* Spacing */
--spacing-xs: 4px;
--spacing-sm: 8px;
--spacing-md: 16px;
--spacing-lg: 24px;
--spacing-xl: 32px;

/* Typography */
--text-primary: #FFFFFF;
--text-secondary: rgba(255, 255, 255, 0.7);
--text-muted: rgba(255, 255, 255, 0.5);
```

### Glass Card Pattern

```css
.my-element {
  background: var(--glass-bg);
  backdrop-filter: blur(20px);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
}
```

### Insight Colors

```css
.insight-icon.lime { color: var(--primary); }
.insight-icon.blue { color: var(--secondary); }
.insight-icon.red { color: var(--negative); }
.insight-icon.yellow { color: var(--warning); }
```

---

## Testing

### Manual Testing Checklist

**Holdings Tab:**
- [ ] Add position with CoinGecko search
- [ ] Add position manually
- [ ] Add position with contract address (0x format)
- [ ] Edit position
- [ ] Delete position
- [ ] Refresh prices
- [ ] Refresh prices within 60 seconds (should return cached)
- [ ] Sort positions by value/P&L/allocation
- [ ] Verify pencil icons are hidden by default (edit mode off)
- [ ] Click "Edit" button in header → pencil icons appear
- [ ] Click pencil → edit/delete actions appear
- [ ] Click "Done" → pencils disappear, active selection cleared

**Input Validation:**
- [ ] Add position with symbol > 20 chars (should fail)
- [ ] Add position with name > 100 chars (should fail)
- [ ] Add position with note > 1000 chars (should fail)
- [ ] Add position with > 20 tags (should fail)
- [ ] Search for token with special chars (e.g., "USD+")

**Exposure Tab:**
- [ ] View asset exposure chart
- [ ] View chain exposure
- [ ] Verify percentages add to 100%
- [ ] Click on a category (e.g., BTC) → side panel opens with news
- [ ] Verify up to 5 news items appear
- [ ] Click a news headline → opens article in new tab
- [ ] Verify news shows source, date, and vote counts

**Insights:**
- [ ] Verify insights appear based on portfolio state
- [ ] Test concentration warning (>50% in one position)
- [ ] Test diversification insights

**Risk Analysis:**
- [ ] Navigate to Risk & Analysis tab → metrics load
- [ ] Refresh the tab within 60 seconds → should load much faster (cached)
- [ ] Verify correlation matrix, volatility scores, and drawdowns display
- [ ] Check recommendations tab also loads quickly (shares risk cache)

**Data Export:**
- [ ] Export positions CSV
- [ ] Export snapshots CSV
- [ ] Verify CSV with formula chars (=SUM) is escaped
- [ ] Open exported CSV in spreadsheet app safely

**Wallet Import:**
- [ ] Click "Import Wallet" → modal opens
- [ ] Enter invalid address → error shown
- [ ] Enter valid 0x address with all chains selected → click "Scan Wallet"
- [ ] Scanning spinner appears → results table loads
- [ ] Verify token list shows symbol, chain, balance, value
- [ ] Toggle individual tokens and "Select All" / "Deselect All"
- [ ] Click "Import" → positions appear in Holdings with "W" badge
- [ ] Click "Refresh Wallet" button → resync runs, toast shows counts
- [ ] Verify wallet-imported position shows "W" badge next to token name
- [ ] Edit a wallet-imported position's entry price → verify it persists
- [ ] Add a manual position → verify it's unaffected by wallet re-sync

**Wallet Import (Mobile):**
- [ ] Modal fits on mobile screen
- [ ] Chain/Balance columns hidden, chain shown inline in token name
- [ ] Import Wallet button visible in header on mobile

### API Testing

```bash
# Test each endpoint
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/holdings
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/exposure
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/insights
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/market
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/tokens/search?q=eth
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/dex/search?q=pepe

# Risk Analysis (first call may be slow; second call within 60s returns cached)
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/risk/metrics
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/recommendations
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/scenarios

# News (CryptoPanic)
curl "http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/news?currencies=BTC"
curl "http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/news?currencies=BTC,ETH"

# Configuration
curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/config \
  -H "Content-Type: application/json" \
  -d '{"api_key": "your-key"}'
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/config

# Demo Portfolio
curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/demo/load
curl -X DELETE http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/demo/clear

# Top Tokens
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/tokens/top

# Debug
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/debug/http

# Wallet Import
curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/wallet/scan \
  -H "Content-Type: application/json" \
  -d '{"address": "0xYOUR_WALLET_ADDRESS", "chains": ["Ethereum", "Arbitrum"]}'

curl -X POST http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/wallet/resync
curl http://localhost:8080/smart-portfolio:smart-portfolio:template.os/api/wallet/status
```

---

## Debugging

### Backend Logs

Rust println! statements appear in the Hyperware node console:

```rust
println!("smart-portfolio: debug message");
```

### Frontend Debugging

```typescript
// Add to store actions for debugging
console.log('Fetched data:', data);

// Check state in browser console
// (Zustand has devtools integration)
```

### Common Issues

**State not persisting:**
- Check `save_state(state)` is called after modifications
- Verify bincode serialization succeeds

**API calls failing:**
- Check CORS settings (shouldn't be an issue for same-origin)
- Verify endpoint path matches router
- Check for JSON parse errors in request body

**UI not updating:**
- Verify Zustand state is being set correctly
- Check component is subscribed to correct store slice
- Force re-render with key change if needed

---

## Performance Tips

### Backend

```rust
// Batch API calls when possible
let prices = fetch_prices_from_coingecko(&[id1, id2, id3]);

// Avoid recalculating on every request if data unchanged
if now - state.last_price_fetch < CACHE_TTL {
    return cached_response;
}
```

### Frontend

```typescript
// Memoize expensive calculations
const sortedPositions = useMemo(() =>
  positions.sort(...),
  [positions, sortField]
);

// Avoid unnecessary re-renders
const specificSlice = usePortfolioStore(state => state.positions);
```

---

## Contributing

### Code Style

**Rust:**
- Use `rustfmt` for formatting
- Follow Rust naming conventions
- Add doc comments for public functions

**TypeScript:**
- Use Prettier for formatting
- Prefer functional components
- Type all function parameters and returns

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add/update tests if applicable
5. Update documentation
6. Submit PR with clear description

### Commit Messages

Follow conventional commits:
```
feat: add new insight type for volume analysis
fix: correct P&L calculation for negative values
docs: update API documentation
refactor: simplify price fetching logic
```
