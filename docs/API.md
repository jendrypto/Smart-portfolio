# Smart Portfolio API Reference

Complete REST API documentation for Smart Portfolio.

## Base URL

All endpoints are relative to the application base URL:
```
/api/
```

## Authentication

No authentication required - the app runs locally on your Hyperware node.

---

## Holdings API

### Get Holdings

Retrieve all positions with calculated values and portfolio summary.

```http
GET /api/holdings
```

**Response:**
```json
{
  "positions": [
    {
      "position": {
        "id": "btc_1705936800",
        "token_identifier": "bitcoin",
        "canonical_id": {
          "canonical": "coingecko:bitcoin",
          "kind": "Coingecko",
          "raw_input": "bitcoin",
          "coingecko_id": "bitcoin",
          "address": null,
          "chain": null
        },
        "token_symbol": "BTC",
        "token_name": "Bitcoin",
        "chain": "Bitcoin",
        "quantity": "0.5",
        "entry_price_usd": "42000.00",
        "entry_date": "2024-01-15",
        "user_note": "DCA purchase",
        "user_tags": ["long-term", "btc"],
        "created_at": 1705936800,
        "updated_at": 1705936800
      },
      "current_price_usd": 67000.00,
      "price_change_24h": 2.5,
      "position_value_usd": 33500.00,
      "unrealized_pnl_usd": 12500.00,
      "unrealized_pnl_percent": 59.52,
      "allocation_percent": 45.2
    }
  ],
  "summary": {
    "total_value_usd": 74150.00,
    "total_unrealized_pnl_usd": 15230.00,
    "total_unrealized_pnl_percent": 25.85,
    "top_positions": [
      {"symbol": "BTC", "name": "Bitcoin", "allocation_percent": 45.2},
      {"symbol": "ETH", "name": "Ethereum", "allocation_percent": 32.1}
    ],
    "largest_position_percent": 45.2,
    "chain_summary": [
      {"chain": "Bitcoin", "value_usd": 33500.00, "percentage": 45.2},
      {"chain": "Ethereum", "value_usd": 40650.00, "percentage": 54.8}
    ],
    "biggest_24h_mover": {
      "symbol": "ETH",
      "name": "Ethereum",
      "change_24h_percent": 5.2,
      "weighted_impact": 1.67
    },
    "last_price_update": 1705950000
  },
  "snapshots": [
    {"date": "2024-01-20", "total_value_usd": 72000.00, "position_count": 5, "timestamp": 1705708800},
    {"date": "2024-01-21", "total_value_usd": 74150.00, "position_count": 5, "timestamp": 1705795200}
  ]
}
```

---

## Position Management

### Add Position

Create a new position in your portfolio.

```http
POST /api/positions
Content-Type: application/json
```

**Request Body:**
```json
{
  "token_identifier": "bitcoin",
  "token_symbol": "BTC",
  "token_name": "Bitcoin",
  "chain": "Bitcoin",
  "quantity": "0.5",
  "entry_price_usd": "42000.00",
  "entry_date": "2024-01-15",
  "user_note": "DCA purchase",
  "user_tags": ["long-term", "btc"]
}
```

| Field | Type | Required | Description | Max Length |
|-------|------|----------|-------------|------------|
| `token_identifier` | string | No | CoinGecko token ID or contract address | - |
| `token_symbol` | string | Yes | Token ticker symbol | 20 chars |
| `token_name` | string | Yes | Full token name | 100 chars |
| `chain` | string | Yes | Blockchain name | 50 chars |
| `quantity` | string | Yes | Amount held (decimal string, > 0) | - |
| `entry_price_usd` | string | Yes | Purchase price per unit (>= 0) | - |
| `entry_date` | string | No | Purchase date (YYYY-MM-DD) | - |
| `user_note` | string | No | Personal notes | 1000 chars |
| `user_tags` | string[] | No | Custom tags (max 20 tags, 50 chars each) | - |

**Canonical ID Resolution:**

The `token_identifier` is automatically converted to a canonical format:
- If it looks like an Ethereum address (0x + 40 hex chars) → `address:<chain>:<addr>`
- If it's a CoinGecko ID → `coingecko:<id>`
- If not provided → `symbol:<SYMBOL>`

**Response (201 Created):**
```json
{
  "id": "btc_1705936800"
}
```

**Validation Errors (400 Bad Request):**
```json
{"error": "Token symbol too long (max 20 characters)"}
{"error": "Token name too long (max 100 characters)"}
{"error": "Chain name too long (max 50 characters)"}
{"error": "Note too long (max 1000 characters)"}
{"error": "Too many tags (max 20)"}
{"error": "Tag too long (max 50 characters)"}
```

### Update Position

Update an existing position.

```http
PUT /api/positions/:id
Content-Type: application/json
```

**Request Body:**
```json
{
  "quantity": "0.75",
  "entry_price_usd": "41500.00",
  "entry_date": "2024-01-14",
  "user_note": "Updated after additional purchase",
  "user_tags": ["long-term", "btc", "dca"]
}
```

All fields are optional - only include fields you want to update.

**Response (200 OK):**
```json
{
  "success": true
}
```

### Delete Position

Remove a position from your portfolio.

```http
DELETE /api/positions/:id
```

**Response (200 OK):**
```json
{
  "success": true
}
```

---

## Exposure API

### Get Exposure

Get asset and chain exposure breakdown.

```http
GET /api/exposure
```

**Response:**
```json
{
  "underlying_exposure": [
    {
      "category": "BTC",
      "value_usd": 33500.00,
      "percentage": 45.2,
      "confidence": "High",
      "notes": "Direct BTC holding",
      "confidence_breakdown": [
        {"position_id": "btc_1705936800", "symbol": "BTC", "confidence": "High"}
      ]
    },
    {
      "category": "ETH",
      "value_usd": 25000.00,
      "percentage": 33.7,
      "confidence": "High",
      "notes": "Direct ETH holding; Staked ETH (Lido)",
      "confidence_breakdown": [
        {"position_id": "eth_1705936800", "symbol": "ETH", "confidence": "High"},
        {"position_id": "steth_1705936800", "symbol": "stETH", "confidence": "Medium"}
      ]
    },
    {
      "category": "Stablecoins",
      "value_usd": 10000.00,
      "percentage": 13.5,
      "confidence": "High",
      "notes": "USDC stablecoin",
      "confidence_breakdown": [
        {"position_id": "usdc_1705936800", "symbol": "USDC", "confidence": "High"}
      ]
    },
    {
      "category": "Other",
      "value_usd": 5650.00,
      "percentage": 7.6,
      "confidence": "Low",
      "notes": "Unclassified asset",
      "confidence_breakdown": [
        {"position_id": "xyz_1705936800", "symbol": "XYZ", "confidence": "Low"}
      ]
    }
  ],
  "chain_exposure": [
    {"chain": "Bitcoin", "value_usd": 33500.00, "percentage": 45.2, "chain_type": "L1"},
    {"chain": "Ethereum", "value_usd": 35000.00, "percentage": 47.2, "chain_type": "L1"},
    {"chain": "Polygon", "value_usd": 5650.00, "percentage": 7.6, "chain_type": "L2"}
  ]
}
```

**Confidence Levels:**
- `High`: Direct holding or well-known wrapped token
- `Medium`: Wrapped/staked version with known backing
- `Low`: Unclassified or complex asset

---

## Insights API

### Get Insights

Get AI-generated portfolio insights.

```http
GET /api/insights
```

**Response:**
```json
{
  "insights": [
    {
      "id": "concentration_high",
      "insight_type": "Concentration",
      "priority": "High",
      "title": "High Concentration Risk",
      "description": "BTC represents 45.2% of your portfolio. Consider diversifying to reduce risk.",
      "icon": "⚠️",
      "color": "red",
      "action_label": "View Exposure",
      "action_url": "/exposure",
      "metadata": {
        "concentration": "45.2",
        "asset": "BTC"
      }
    },
    {
      "id": "strong_gains",
      "insight_type": "Performance",
      "priority": "Medium",
      "title": "Strong Portfolio Performance",
      "description": "Your portfolio is up 25.8% ($15,230). Consider taking partial profits to lock in gains.",
      "icon": "🚀",
      "color": "lime",
      "action_label": null,
      "action_url": null,
      "metadata": {
        "pnl_percent": "25.8",
        "pnl_usd": "15230.00"
      }
    }
  ],
  "generated_at": 1705950000,
  "portfolio_health_score": 75
}
```

**Insight Types:**
| Type | Description |
|------|-------------|
| `Concentration` | Portfolio concentration warnings |
| `Performance` | P&L related insights |
| `Diversification` | Chain/asset diversification |
| `Rebalancing` | Suggestions for rebalancing |
| `PriceMovement` | 24h price movement alerts |
| `RiskAlert` | Risk-related warnings |
| `Opportunity` | Potential opportunities |
| `General` | General portfolio health |

**Priority Levels:** `High`, `Medium`, `Low`

**Colors:** `lime` (positive), `blue` (info), `yellow` (warning), `red` (alert)

---

## Pricing API

### Refresh Prices

Trigger a price refresh for all positions from CoinGecko.

```http
POST /api/refresh
```

**Response (200 OK) - Fresh fetch:**
```json
{
  "success": true
}
```

**Response (200 OK) - Cached (within 60 seconds):**
```json
{
  "message": "Prices still fresh",
  "cached": true,
  "next_refresh_in": 45
}
```

The endpoint enforces a 60-second cache TTL:
- If prices were fetched less than 60 seconds ago, returns cached status
- `next_refresh_in` indicates seconds until next allowed refresh
- This prevents excessive CoinGecko API calls

This also creates a daily snapshot if one doesn't exist for today.

### Search Tokens

Search for tokens on CoinGecko.

```http
GET /api/tokens/search?q=bitcoin
```

**Query Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `q` | Yes | Search query (min 1 character) |

The query is URL-encoded before sending to CoinGecko, so special characters like `+`, `/`, `&` are handled safely.

**Response:**
```json
[
  {"id": "bitcoin", "symbol": "btc", "name": "Bitcoin"},
  {"id": "bitcoin-cash", "symbol": "bch", "name": "Bitcoin Cash"},
  {"id": "wrapped-bitcoin", "symbol": "wbtc", "name": "Wrapped Bitcoin"}
]
```

### Get Token Price

Get current price for a specific token by CoinGecko ID.

```http
GET /api/tokens/price/:token_id
```

**Response (200 OK):**
```json
{
  "price_usd": 67000.00,
  "price_change_24h": 2.5
}
```

**Error (404 Not Found):**
```json
{"error": "Price not found"}
```

### Get Top Tokens

Get top 25 tokens by market cap from CoinGecko.

```http
GET /api/tokens/top
```

**Response:**
```json
{
  "tokens": [
    {
      "id": "bitcoin",
      "symbol": "BTC",
      "name": "Bitcoin",
      "current_price": 67000.00,
      "price_change_percentage_24h": 2.5,
      "market_cap_rank": 1
    }
  ]
}
```

---

## Market Data API

### Get Market Data

Get comprehensive market data for all positions.

```http
GET /api/market
```

**Response:**
```json
{
  "chain_tvl": [
    {
      "chain": "Ethereum",
      "tvl": 58000000000,
      "tvl_change_24h": 2.3,
      "protocols_count": 850,
      "timestamp": 1705950000
    }
  ],
  "position_market_data": [
    {
      "position_id": "btc_1705936800",
      "symbol": "BTC",
      "price_usd": 67000.00,
      "price_change_24h": 2.5,
      "volume_24h": 28000000000,
      "liquidity_usd": null,
      "market_cap": 1320000000000,
      "fdv": 1400000000000,
      "price_source": "CoinGecko"
    }
  ],
  "last_updated": 1705950000
}
```

**Price Sources:** `CoinGecko`, `DefiLlama`, `Dexscreener`, `Manual`

### Refresh Chain TVL

Update chain TVL data from DeFi Llama.

```http
POST /api/market/tvl
```

**Response (200 OK):**
```json
{
  "success": true
}
```

TVL data is cached for 5 minutes.

---

## Risk Analysis API

### Get Risk Metrics

Get portfolio risk metrics including correlation matrix, volatility scores, and drawdown analysis.

```http
GET /api/risk/metrics
```

**Response:**
```json
{
  "correlation_matrix": {
    "assets": ["bitcoin", "ethereum"],
    "matrix": [[1.0, 0.85], [0.85, 1.0]]
  },
  "volatility_scores": [
    {
      "asset": "bitcoin",
      "symbol": "BTC",
      "volatility_7d": 42.5,
      "volatility_30d": 38.2,
      "volatility_rank": "medium"
    }
  ],
  "drawdowns": [
    {
      "asset": "bitcoin",
      "symbol": "BTC",
      "current_drawdown": 5.2,
      "max_drawdown_30d": 12.8,
      "peak_price": 71000.00,
      "trough_price": 61900.00
    }
  ],
  "portfolio_volatility": 40.3,
  "risk_score": 45,
  "generated_at": 1705950000
}
```

**Volatility Ranks:** `low` (< 30%), `medium` (30-60%), `high` (60-100%), `extreme` (> 100%)

**Caching:** Results are cached for 60 seconds in the backend. The first call computes fresh metrics (fetching 30 days of historical prices from DeFi Llama, which may take several seconds). Subsequent calls within 60 seconds return the cached result instantly. The `/api/recommendations` endpoint shares this same cache.

---

### Get Recommendations

Get actionable portfolio recommendations based on risk analysis.

```http
GET /api/recommendations
```

**Response:**
```json
{
  "recommendations": [
    {
      "id": "reduce_concentration",
      "priority": "critical",
      "category": "risk",
      "title": "Reduce BTC Concentration",
      "description": "BTC is 55.0% of your portfolio - above the recommended 30% maximum.",
      "impact": "Reduces single-asset risk by 25%",
      "action": {
        "action_type": "rebalance",
        "label": "Rebalance 25% to diversify",
        "from_asset": "BTC",
        "to_asset": null,
        "percentage": 25.0,
        "estimated_value": 18500.00
      }
    }
  ],
  "risk_score": 45,
  "generated_at": 1705950000
}
```

**Priority Levels:** `critical`, `high`, `medium`, `low`

**Categories:** `risk`, `opportunity`, `rebalance`, `alert`

**Action Types:** `rebalance`, `sell`, `buy`

---

### Get Scenarios

Get stress test scenarios showing portfolio impact under various market conditions.

```http
GET /api/scenarios
```

**Response:**
```json
{
  "scenarios": [
    {
      "id": "crypto_winter",
      "name": "Crypto Winter",
      "description": "Major market crash scenario",
      "portfolio_impact": -65.2,
      "affected_assets": [
        {
          "symbol": "BTC",
          "current_value": 33500.00,
          "scenario_value": 13400.00,
          "percent_change": -60.0
        }
      ]
    },
    {
      "id": "bull_run",
      "name": "Bull Market",
      "description": "Major crypto rally",
      "portfolio_impact": 135.0,
      "affected_assets": []
    }
  ],
  "current_value": 74150.00
}
```

**Predefined Scenarios:**
| Scenario | Description |
|----------|-------------|
| `crypto_winter` | Major crash: BTC -60%, ETH -70%, Alt L1s -80% |
| `eth_rally` | ETH outperforms: ETH +50%, BTC -10% |
| `stablecoin_depeg` | Stablecoin risk: Stables -15%, others minor impact |
| `bull_run` | Rally: BTC +100%, ETH +150%, Alt L1s +200% |

---

## News API

### Get News

Retrieve recent crypto news for specified currencies from CryptoPanic.

```http
GET /api/news?currencies=BTC,ETH
```

**Query Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `currencies` | Yes | Comma-separated currency codes (e.g., `BTC`, `BTC,ETH`) |

**Response:**
```json
{
  "news": [
    {
      "title": "Bitcoin Surges Past $100K on Institutional Demand",
      "url": "https://example.com/article",
      "source": "CoinDesk",
      "published_at": "2026-01-27T14:30:00Z",
      "positive_votes": 12,
      "negative_votes": 2
    }
  ]
}
```

**Notes:**
- Returns up to 5 most recent news items
- Uses the CryptoPanic Developer API v2 (`/api/developer/v2/posts/`)
- A built-in fallback API key is used if no custom key is configured
- To set a custom key: `POST /api/config` with `{"cryptopanic_api_key": "your-key"}`
- News items are clickable links in the UI, opening articles in a new tab
- If no API key is available or currencies is empty, returns an empty news array

**Empty response (no key configured and no fallback):**
```json
{
  "news": [],
  "message": "CryptoPanic API key not configured"
}
```

---

## DEX API

### Search DEX Pairs

Search for token pairs on decentralized exchanges.

```http
GET /api/dex/search?q=PEPE
```

**Query Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `q` | Yes | Search query (min 2 characters) |

The query is URL-encoded before sending to Dexscreener, so special characters are handled safely.

**Response:**
```json
{
  "pairs": [
    {
      "pair_address": "0x...",
      "dex_name": "uniswap",
      "chain": "ethereum",
      "base_token_symbol": "PEPE",
      "base_token_name": "Pepe",
      "quote_token_symbol": "WETH",
      "price_usd": 0.00001234,
      "price_change_24h": 15.5,
      "volume_24h": 45000000,
      "liquidity_usd": 12000000,
      "fdv": 5200000000,
      "pair_created_at": 1681234567
    }
  ],
  "query": "PEPE"
}
```

### Get DEX Token Price

Get price for a specific token by contract address.

```http
GET /api/dex/token/:address
```

**Response:**
```json
{
  "price_usd": 0.00001234,
  "price_change_24h": 15.5,
  "market_cap": null,
  "timestamp": 1705950000,
  "is_stale": false,
  "volume_24h": 45000000,
  "liquidity_usd": 12000000,
  "price_source": "Dexscreener",
  "fdv": 5200000000
}
```

---

## Wallet API

### Scan Wallet

Scan an EVM wallet address for token balances across selected chains.

```http
POST /api/wallet/scan
Content-Type: application/json
```

**Request Body:**
```json
{
  "address": "0x1234567890abcdef1234567890abcdef12345678",
  "chains": ["Ethereum", "Arbitrum", "Optimism", "Base", "Polygon", "Avalanche", "BNB Chain"]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | string | Yes | EVM wallet address (0x + 40 hex characters) |
| `chains` | string[] | Yes | List of chains to scan. Valid: Ethereum, Arbitrum, Optimism, Base, Polygon, Avalanche, BNB Chain |

**Response (200 OK):**
```json
{
  "address": "0x1234...5678",
  "tokens": [
    {
      "symbol": "ETH",
      "name": "Ethereum",
      "chain": "Ethereum",
      "balance": "1.500000000000000000",
      "price_usd": "3200.00",
      "value_usd": "4800.00",
      "token_identifier": "ethereum",
      "token_address": null,
      "is_native": true
    },
    {
      "symbol": "USDC",
      "name": "USD Coin",
      "chain": "Ethereum",
      "balance": "5000.000000",
      "price_usd": "1.00",
      "value_usd": "5000.00",
      "token_identifier": null,
      "token_address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      "is_native": false
    }
  ],
  "dust_filtered": 12
}
```

**Notes:**
- Uses Moralis Web3 Data API (`/api/v2.2/wallets/{address}/tokens`) which returns balances with USD prices
- One API call per chain (7 max)
- Tokens with USD value < $1 are filtered as dust (count returned in `dust_filtered`)
- Results sorted by value descending
- Stores wallet address and chains in state for re-sync

**Errors:**
```json
{"error": "Invalid wallet address format"}
{"error": "No chains specified"}
```

---

### Import Wallet Tokens

Import selected tokens from a wallet scan as portfolio positions.

```http
POST /api/wallet/import
Content-Type: application/json
```

**Request Body:**
```json
{
  "tokens": [
    {
      "symbol": "ETH",
      "name": "Ethereum",
      "chain": "Ethereum",
      "balance": "1.500000000000000000",
      "price_usd": "3200.00",
      "value_usd": "4800.00",
      "token_identifier": "ethereum",
      "token_address": null,
      "is_native": true
    }
  ]
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "imported": 5
}
```

**Notes:**
- Each token creates a Position with `source: "wallet"`
- Entry price set to current price at scan time
- Token identifier resolved to canonical ID (CoinGecko ID for known tokens, address-based for others)

---

### Resync Wallet

Re-scan the stored wallet address and update wallet-imported positions.

```http
POST /api/wallet/resync
```

**Response (200 OK):**
```json
{
  "success": true,
  "updated": 3,
  "added": 1,
  "removed": 2
}
```

**Logic:**
- Re-scans all chains from the original scan
- Existing wallet positions: quantity updated to match current balance
- New tokens found: added as new positions
- Tokens no longer found (zero balance): automatically removed

**Errors:**
```json
{"error": "No wallet address stored. Scan a wallet first."}
```

---

### Get Wallet Status

Check if a wallet address is stored for re-sync.

```http
GET /api/wallet/status
```

**Response (200 OK):**
```json
{
  "address": "0x1234...5678",
  "chains": ["Ethereum", "Arbitrum", "Base"]
}
```

**Response (no wallet stored):**
```json
{
  "address": null,
  "chains": []
}
```

---

## Export API

### Export Positions (CSV)

Download all positions as CSV.

```http
GET /api/export/positions
```

**Response:** CSV file download with headers:
```
id,symbol,name,chain,quantity,entry_price_usd,entry_date,user_note,user_tags,created_at
```

### Export Snapshots (CSV)

Download historical snapshots as CSV.

```http
GET /api/export/snapshots
```

**Response:** CSV file download with headers:
```
date,total_value_usd,position_count,timestamp
```

**CSV Safety:**

All exported CSV files follow RFC 4180 with additional security:
- Fields containing commas, quotes, or newlines are properly escaped
- Formula-triggering characters (`=`, `+`, `-`, `@`) are prefixed with `'` to prevent CSV injection
- Safe to open in any spreadsheet application

---

## Demo Portfolio API

### Load Demo Portfolio

Load a preset demo portfolio with 4 positions (BTC, ETH, SOL, USDC).

```http
POST /api/demo/load
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Demo portfolio loaded",
  "positions_added": 4
}
```

**Error (409 Conflict):**
```json
{"error": "Demo portfolio already loaded"}
```

Prices are fetched from CoinGecko for each demo position.

---

### Clear Demo Portfolio

Remove all demo positions (identified by the "demo" tag).

```http
DELETE /api/demo/clear
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Demo portfolio cleared",
  "positions_removed": 4
}
```

**Error (404 Not Found):**
```json
{"error": "No demo positions to clear"}
```

---

## Configuration API

### Set Configuration

Configure application settings such as the CoinGecko API key.

```http
POST /api/config
Content-Type: application/json
```

**Request Body:**
```json
{
  "api_key": "CG-your-api-key-here"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_key` | string | No | CoinGecko API key. Send empty string to clear. |

**Response (200 OK):**
```json
{
  "success": true
}
```

The API key is stored in persistent state and survives restarts.

---

### Get Configuration

Check current configuration status.

```http
GET /api/config
```

**Response:**
```json
{
  "api_key_configured": true,
  "api_key_masked": "CG-2...tizq"
}
```

The API key is masked in the response for security.

---

## Debug API

### Test HTTP Connectivity

Test outbound HTTP connectivity from the Hyperware node.

```http
GET /api/debug/http
```

**Response (200 OK):**
```json
{
  "success": true,
  "status": 200,
  "body_len": 1234,
  "message": "Outbound HTTP works!"
}
```

This endpoint makes a request to httpbin.org to verify the `http-client:distro:sys` capability is working.

---

## Error Responses

All endpoints return errors in this format:

```json
{
  "error": "Error message description"
}
```

**Common HTTP Status Codes:**
| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request (invalid input) |
| 404 | Not Found |
| 500 | Internal Server Error |

---

## Rate Limits

- **CoinGecko**: Subject to CoinGecko API limits (demo key: ~30 calls/min)
- **DeFi Llama**: No strict limits, but be respectful
- **Dexscreener**: No strict limits, but be respectful
- **CryptoPanic**: Developer tier; see [CryptoPanic API docs](https://cryptopanic.com/developers/api/) for limits
- **Moralis**: Free tier: 40,000 requests/month (~2,857 full wallet scans)

**Internal Caching:**

| Data | Cache TTL | Notes |
|------|-----------|-------|
| Prices | 60 seconds | Enforced on POST /api/refresh |
| Chain TVL | 5 minutes | Enforced on POST /api/market/tvl |
| Risk Metrics | 60 seconds | Shared by /api/risk/metrics and /api/recommendations |
| Snapshots | 1 day | One snapshot per calendar day |

Requests within the cache window return immediately with cached data.
