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
      "notes": "Direct BTC holding"
    },
    {
      "category": "ETH",
      "value_usd": 25000.00,
      "percentage": 33.7,
      "confidence": "High",
      "notes": "Direct ETH holding; Staked ETH (Lido)"
    },
    {
      "category": "Stablecoins",
      "value_usd": 10000.00,
      "percentage": 13.5,
      "confidence": "High",
      "notes": "USDC stablecoin"
    },
    {
      "category": "Other",
      "value_usd": 5650.00,
      "percentage": 7.6,
      "confidence": "Low",
      "notes": "Unclassified asset"
    }
  ],
  "chain_exposure": [
    {"chain": "Bitcoin", "value_usd": 33500.00, "percentage": 45.2},
    {"chain": "Ethereum", "value_usd": 35000.00, "percentage": 47.2},
    {"chain": "Polygon", "value_usd": 5650.00, "percentage": 7.6}
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

**Internal Caching:**

| Data | Cache TTL | Notes |
|------|-----------|-------|
| Prices | 60 seconds | Enforced on POST /api/refresh |
| Chain TVL | 5 minutes | Enforced on POST /api/market/tvl |

Requests within the cache window return immediately with cached data.
