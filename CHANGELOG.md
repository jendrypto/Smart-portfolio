# Changelog

All notable changes to Smart Portfolio are documented in this file.

Format: [Semantic Versioning](https://semver.org/) with date/time stamps.

---

## [0.2.0] - 2026-01-27

### Summary

Three bug fixes and UX improvements addressing news panel failures, slow risk analysis loading, and cluttered edit controls in the positions table. Also added CryptoPanic news integration with the v2 developer API.

### Added

- **CryptoPanic News Integration** (2026-01-27 ~15:30 UTC)
  - Integrated CryptoPanic Developer API v2 for real-time crypto news
  - News panel in the Exposure side panel shows the 5 most recent articles per token/category
  - Each news item is a clickable link that opens the source article in a new tab
  - Displays source name, publication date, and community vote counts (+/-)
  - Hardcoded fallback API key so news works out of the box without manual configuration
  - API endpoint: `GET /api/news?currencies=BTC,ETH`

- **Risk Metrics Backend Caching** (2026-01-27 ~15:45 UTC)
  - Added 60-second in-memory cache for `PortfolioRiskMetrics` in `AppState`
  - New fields: `cached_risk_metrics: Option<PortfolioRiskMetrics>`, `risk_metrics_cached_at: u64`
  - Both `GET /api/risk/metrics` and `GET /api/recommendations` share the same cache
  - Eliminates redundant 30 sequential HTTP requests to DeFi Llama on repeated loads
  - Cache is persisted to state so it survives across API calls within the same session
  - Helper function `get_or_compute_risk_metrics()` manages cache lifecycle

- **Edit Mode Toggle for Positions Table** (2026-01-27 ~16:00 UTC)
  - Replaced always-visible per-row pencil icons with a top-level "Edit" / "Done" toggle button
  - Button located in the positions card header, next to "Last price update"
  - When edit mode is OFF: pencil column and actions column are completely hidden (cleaner default view)
  - When edit mode is ON: pencil icons appear per row, allowing edit/delete actions as before
  - Toggling off edit mode automatically clears any active row selection and pending delete confirmations

### Fixed

- **News Panel Showing No News** (2026-01-27 ~15:30 UTC)
  - **Root cause 1**: `AppState` was serialized with `bincode`, but the `cryptopanic_api_key` field (added later) had no `#[serde(default)]` attribute. Old serialized state missing this field would fail to deserialize, causing `load_state()` to fall back to `unwrap_or_default()` which reset ALL state.
  - **Fix**: Added `#[serde(default)]` at the struct level on `AppState` so all fields gracefully default when missing from serialized data.
  - **Root cause 2**: The CryptoPanic API URL was wrong. Code used the v1 public endpoint (`/api/v1/posts/`) but the developer API key requires the v2 developer endpoint.
  - **Fix**: Updated URL to `https://cryptopanic.com/api/developer/v2/posts/`
  - Added debug logging in `handle_get_news` (currencies requested, key length, items fetched)

- **Risk Analysis Slow on Every Load** (2026-01-27 ~15:45 UTC)
  - **Root cause**: `calculate_risk_metrics()` calls `fetch_historical_prices_range()` which makes 30 sequential HTTP requests (one per day) to DeFi Llama. This ran on every single `/api/risk/metrics` AND `/api/recommendations` call with no caching.
  - **Fix**: Added 60-second backend cache. First call computes and caches; subsequent calls within 60s return cached result instantly.

### Changed

- `handle_get_risk_metrics` signature changed from `&AppState` to `&mut AppState` (to support caching writes)
- `handle_get_recommendations` signature changed from `&AppState` to `&mut AppState` (to support caching reads/writes)
- News results limited from 10 items to 5 items (backend `.take(5)` and frontend `.slice(0, 5)`)

### Files Modified

| File | Changes |
|------|---------|
| `smart-portfolio/src/lib.rs` | Added `#[serde(default)]` to AppState; added `cached_risk_metrics` and `risk_metrics_cached_at` fields; added `CRYPTOPANIC_FALLBACK_KEY` constant; added `RISK_CACHE_TTL_SECS` constant; added `get_or_compute_risk_metrics()` helper; updated `handle_get_news` with fallback key and logging; updated `handle_get_risk_metrics` and `handle_get_recommendations` to use cache; fixed CryptoPanic API URL to v2 developer endpoint; reduced news limit from 10 to 5 |
| `ui/src/components/PositionsTable.tsx` | Added `editMode` state; added Edit/Done toggle button in header; wrapped pencil `<th>`/`<td>` and actions `<th>`/`<td>` in `{editMode && ...}` conditionals; clear `activeRowId` and `confirmDelete` when exiting edit mode; added `.slice(0, 5)` safety limit on news rendering |
| `ui/src/components/ExposureSidePanel.tsx` | Added `.slice(0, 5)` to news item rendering loop |

---

## [0.1.0] - Initial Release

### Features

- **Holdings Tracking**: Add/edit/delete crypto positions across multiple chains
- **Portfolio Analytics**: Summary dashboard, historical snapshots, P&L calculations
- **Exposure Analysis**: Asset category breakdown (BTC, ETH, Stablecoins, Other) and chain exposure
- **Smart Insights**: AI-powered portfolio analysis with concentration warnings, performance feedback, price alerts, diversification analysis, liquidity warnings, and health scoring (0-100)
- **Risk Analysis**: Correlation matrix (Pearson), volatility tracking (7d/30d annualized), drawdown analysis, aggregate risk score (0-100)
- **Stress Testing**: Scenario simulations (crypto winter, bull run, ETH rally, stablecoin depeg)
- **Actionable Recommendations**: Prioritized suggestions for rebalancing, risk mitigation, and opportunities
- **Multi-Source Pricing**: CoinGecko, DeFi Llama, and Dexscreener integration
- **Data Export**: CSV export for positions and snapshots with injection prevention
- **Demo Portfolio**: Built-in 4-position demo (BTC, ETH, SOL, USDC) for testing
- **Canonical Identifier System**: Consistent token identification across data sources
- **Glass-Morphism UI**: Dark theme with semi-transparent panels, backdrop blur, gradient accents
- **Security**: Input validation, CSV injection prevention, URL encoding, API key masking
- **Local-First**: All data stored on user's Hyperware node, no external database

### Technology Stack

- Backend: Rust (WASM), hyperware_process_lib, serde, bincode, rust_decimal
- Frontend: React 18, TypeScript, Zustand, Recharts, Vite
- APIs: CoinGecko, DeFi Llama, Dexscreener, CryptoPanic
