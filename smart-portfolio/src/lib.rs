//! Smart Portfolio - Hyperware App
//! A local-first portfolio tracker with Holdings and Exposure views.

use hyperware_process_lib::{
    await_message, call_init, get_typed_state, http::{
        self,
        server::{HttpBindingConfig, HttpServer, HttpServerRequest, IncomingHttpRequest},
        Method, StatusCode,
    },
    println, set_state, Address,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

wit_bindgen::generate!({
    path: "../target/wit",
    world: "process-v1",
});

// ============================================================================
// URL Encoding Helper
// ============================================================================

/// URL-encode a string for safe use in query parameters
fn url_encode(input: &str) -> String {
    url::form_urlencoded::byte_serialize(input.as_bytes()).collect()
}

// ============================================================================
// Domain Types - Positions
// ============================================================================

/// Identifies the type/source of a token identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdentifierKind {
    /// CoinGecko API identifier (e.g., "bitcoin", "ethereum")
    Coingecko,
    /// On-chain contract address
    Address,
    /// Unresolved symbol (no price fetch possible)
    Symbol,
}

/// Canonical identifier for consistent storage and lookup
/// Format: "coingecko:<id>" or "address:<chain>:<lowercase_addr>"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalId {
    /// The canonical key used for all lookups
    pub canonical: String,
    /// The type of identifier
    pub kind: IdentifierKind,
    /// Original user input (preserved for display/auditing)
    pub raw_input: String,
    /// CoinGecko ID if resolved
    pub coingecko_id: Option<String>,
    /// Contract address if applicable (lowercase, normalized)
    pub address: Option<String>,
    /// Chain for address-based identifiers
    pub chain: Option<String>,
}

impl CanonicalId {
    /// Create a canonical ID from a CoinGecko identifier
    pub fn from_coingecko(id: &str, raw_input: &str) -> Self {
        Self {
            canonical: format!("coingecko:{}", id.to_lowercase()),
            kind: IdentifierKind::Coingecko,
            raw_input: raw_input.to_string(),
            coingecko_id: Some(id.to_lowercase()),
            address: None,
            chain: None,
        }
    }

    /// Create a canonical ID from a contract address
    pub fn from_address(address: &str, chain: &str, raw_input: &str) -> Self {
        let normalized_addr = address.to_lowercase();
        Self {
            canonical: format!("address:{}:{}", chain.to_lowercase(), normalized_addr),
            kind: IdentifierKind::Address,
            raw_input: raw_input.to_string(),
            coingecko_id: None,
            address: Some(normalized_addr),
            chain: Some(chain.to_lowercase()),
        }
    }

    /// Create an unresolved symbol (placeholder until resolved)
    pub fn from_symbol(symbol: &str) -> Self {
        Self {
            canonical: format!("symbol:{}", symbol.to_uppercase()),
            kind: IdentifierKind::Symbol,
            raw_input: symbol.to_string(),
            coingecko_id: None,
            address: None,
            chain: None,
        }
    }
}

/// Checks if a string looks like an Ethereum-style address (0x + 40 hex chars)
fn is_eth_address(input: &str) -> bool {
    input.len() == 42
        && input.starts_with("0x")
        && input[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Resolves user input into a canonical identifier
fn resolve_canonical_id(
    token_identifier: Option<&str>,
    token_symbol: &str,
    chain: &str,
) -> CanonicalId {
    if let Some(identifier) = token_identifier {
        // If it looks like an address, treat it as such
        if is_eth_address(identifier) {
            return CanonicalId::from_address(identifier, chain, identifier);
        }
        // Otherwise, treat as CoinGecko ID
        return CanonicalId::from_coingecko(identifier, identifier);
    }
    // No identifier provided - use symbol as unresolved
    CanonicalId::from_symbol(token_symbol)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub token_identifier: Option<String>, // CoinGecko ID (kept for backwards compatibility)
    pub canonical_id: Option<CanonicalId>, // NEW: canonical identifier for consistent lookups
    pub token_symbol: String,
    pub token_name: String,
    pub chain: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub quantity: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub entry_price_usd: Decimal,
    pub entry_date: Option<String>, // ISO date string YYYY-MM-DD
    pub user_note: Option<String>,
    pub user_tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

// ============================================================================
// Domain Types - Pricing
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub price_usd: f64,
    pub price_change_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub timestamp: u64,
    pub is_stale: bool,
    // Enhanced data from multiple sources
    pub volume_24h: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub price_source: PriceSource,
    pub fdv: Option<f64>, // Fully diluted valuation
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum PriceSource {
    #[default]
    CoinGecko,
    DefiLlama,
    Dexscreener,
    Manual,
}

// ============================================================================
// Domain Types - Chain/Protocol Data (DeFi Llama)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTvlData {
    pub chain: String,
    pub tvl: f64,
    pub tvl_change_24h: Option<f64>,
    pub protocols_count: Option<u32>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolData {
    pub name: String,
    pub slug: String,
    pub tvl: f64,
    pub chain: String,
    pub category: String,
    pub symbol: Option<String>,
}

// ============================================================================
// Domain Types - DEX Data (Dexscreener)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexPairData {
    pub pair_address: String,
    pub dex_name: String,
    pub chain: String,
    pub base_token_symbol: String,
    pub base_token_name: String,
    pub quote_token_symbol: String,
    pub price_usd: f64,
    pub price_change_24h: Option<f64>,
    pub volume_24h: f64,
    pub liquidity_usd: f64,
    pub fdv: Option<f64>,
    pub pair_created_at: Option<u64>,
}

// ============================================================================
// Domain Types - Snapshots
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySnapshot {
    pub date: String, // YYYY-MM-DD
    pub total_value_usd: f64,
    pub position_count: usize,
    pub timestamp: u64,
}

// ============================================================================
// Domain Types - Analytics / Exposure
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExposureCategory {
    ETH,
    BTC,
    Stablecoins,
    Other,
}

impl ExposureCategory {
    fn as_str(&self) -> &'static str {
        match self {
            ExposureCategory::ETH => "ETH",
            ExposureCategory::BTC => "BTC",
            ExposureCategory::Stablecoins => "Stablecoins",
            ExposureCategory::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureEntry {
    pub category: String,
    pub value_usd: f64,
    pub percentage: f64,
    pub confidence: ConfidenceLevel,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExposure {
    pub chain: String,
    pub value_usd: f64,
    pub percentage: f64,
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionWithDerived {
    pub position: Position,
    pub current_price_usd: f64,
    pub price_change_24h: Option<f64>,
    pub position_value_usd: f64,
    pub unrealized_pnl_usd: f64,
    pub unrealized_pnl_percent: f64,
    pub allocation_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub total_value_usd: f64,
    pub total_unrealized_pnl_usd: f64,
    pub total_unrealized_pnl_percent: f64,
    pub top_positions: Vec<TopPosition>,
    pub largest_position_percent: f64,
    pub chain_summary: Vec<ChainExposure>,
    pub biggest_24h_mover: Option<MoverInfo>,
    pub last_price_update: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopPosition {
    pub symbol: String,
    pub name: String,
    pub allocation_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoverInfo {
    pub symbol: String,
    pub name: String,
    pub change_24h_percent: f64,
    pub weighted_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingsResponse {
    pub positions: Vec<PositionWithDerived>,
    pub summary: PortfolioSummary,
    pub snapshots: Vec<DailySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureResponse {
    pub underlying_exposure: Vec<ExposureEntry>,
    pub chain_exposure: Vec<ChainExposure>,
}

// ============================================================================
// API Response Types - Market Data
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataResponse {
    pub chain_tvl: Vec<ChainTvlData>,
    pub position_market_data: Vec<PositionMarketData>,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMarketData {
    pub position_id: String,
    pub symbol: String,
    pub price_usd: f64,
    pub price_change_24h: Option<f64>,
    pub volume_24h: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub market_cap: Option<f64>,
    pub fdv: Option<f64>,
    pub price_source: PriceSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexSearchResponse {
    pub pairs: Vec<DexPairData>,
    pub query: String,
}

// ============================================================================
// Domain Types - Insights
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InsightType {
    Concentration,      // Portfolio concentration warnings/info
    Performance,        // P&L related insights
    Diversification,    // Chain/asset diversification
    Rebalancing,        // Suggestions for rebalancing
    PriceMovement,      // 24h price movement alerts
    RiskAlert,          // Risk-related warnings
    Opportunity,        // Potential opportunities
    General,            // General portfolio health
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InsightPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: String,
    pub insight_type: InsightType,
    pub priority: InsightPriority,
    pub title: String,
    pub description: String,
    pub icon: String,           // Emoji or icon identifier
    pub color: String,          // lime, blue, red, yellow
    pub action_label: Option<String>,
    pub action_url: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsResponse {
    pub insights: Vec<Insight>,
    pub generated_at: u64,
    pub portfolio_health_score: u8,  // 0-100 score
}

// ============================================================================
// API Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct AddPositionRequest {
    token_identifier: Option<String>,
    token_symbol: String,
    token_name: String,
    chain: String,
    quantity: String, // String for decimal parsing
    entry_price_usd: String,
    entry_date: Option<String>,
    user_note: Option<String>,
    user_tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct UpdatePositionRequest {
    quantity: Option<String>,
    entry_price_usd: Option<String>,
    entry_date: Option<String>,
    user_note: Option<String>,
    user_tags: Option<Vec<String>>,
}

/// Validates input bounds for position creation
fn validate_add_position_input(request: &AddPositionRequest) -> Result<(), &'static str> {
    if request.token_symbol.len() > MAX_SYMBOL_LENGTH {
        return Err("Token symbol too long (max 20 characters)");
    }
    if request.token_name.len() > MAX_NAME_LENGTH {
        return Err("Token name too long (max 100 characters)");
    }
    if request.chain.len() > MAX_CHAIN_LENGTH {
        return Err("Chain name too long (max 50 characters)");
    }
    if let Some(note) = &request.user_note {
        if note.len() > MAX_NOTE_LENGTH {
            return Err("Note too long (max 1000 characters)");
        }
    }
    if let Some(tags) = &request.user_tags {
        if tags.len() > MAX_TAGS_COUNT {
            return Err("Too many tags (max 20)");
        }
        for tag in tags {
            if tag.len() > MAX_TAG_LENGTH {
                return Err("Tag too long (max 50 characters)");
            }
        }
    }
    Ok(())
}

/// Validates input bounds for position update
fn validate_update_position_input(request: &UpdatePositionRequest) -> Result<(), &'static str> {
    if let Some(note) = &request.user_note {
        if note.len() > MAX_NOTE_LENGTH {
            return Err("Note too long (max 1000 characters)");
        }
    }
    if let Some(tags) = &request.user_tags {
        if tags.len() > MAX_TAGS_COUNT {
            return Err("Too many tags (max 20)");
        }
        for tag in tags {
            if tag.len() > MAX_TAG_LENGTH {
                return Err("Tag too long (max 50 characters)");
            }
        }
    }
    Ok(())
}

/// Escapes a field for CSV output following RFC 4180
/// Also prefixes formula-triggering characters to prevent CSV injection
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

    let needs_escape = sanitized.contains(',')
        || sanitized.contains('"')
        || sanitized.contains('\n')
        || sanitized.contains('\r');

    if needs_escape {
        // Wrap in quotes and escape internal quotes
        format!("\"{}\"", sanitized.replace('"', "\"\""))
    } else {
        sanitized
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenSearchResult {
    id: String,
    symbol: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TopToken {
    id: String,
    symbol: String,
    name: String,
    current_price: f64,
    price_change_percentage_24h: f64,
    market_cap_rank: u32,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoMarketCoin {
    id: String,
    symbol: String,
    name: String,
    current_price: Option<f64>,
    price_change_percentage_24h: Option<f64>,
    market_cap_rank: Option<u32>,
}

// ============================================================================
// State
// ============================================================================

const COINGECKO_CACHE_TTL_SECS: u64 = 60; // 60 second cache
const COINGECKO_API_KEY: &str = "CG-2qKB1Q4fDxnQBLt6rYR7tizq";

// DeFi Llama API (free, no key needed)
const DEFILLAMA_API_BASE: &str = "https://api.llama.fi";
const DEFILLAMA_COINS_API: &str = "https://coins.llama.fi";

// Dexscreener API (free, no key needed)
const DEXSCREENER_API_BASE: &str = "https://api.dexscreener.com/latest/dex";

// Input validation limits
const MAX_SYMBOL_LENGTH: usize = 20;
const MAX_NAME_LENGTH: usize = 100;
const MAX_CHAIN_LENGTH: usize = 50;
const MAX_NOTE_LENGTH: usize = 1000;
const MAX_TAG_LENGTH: usize = 50;
const MAX_TAGS_COUNT: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AppState {
    positions: HashMap<String, Position>,
    prices: HashMap<String, PriceData>, // keyed by coingecko id or contract address
    snapshots: Vec<DailySnapshot>,
    last_price_fetch: u64,
    // Enhanced data from additional APIs
    chain_tvl: HashMap<String, ChainTvlData>, // keyed by chain name
    dex_pairs: HashMap<String, Vec<DexPairData>>, // keyed by token symbol/address
    last_tvl_fetch: u64,
}

// ============================================================================
// Demo Portfolio Generator
// ============================================================================

fn get_demo_positions() -> Vec<Position> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    vec![
        Position {
            id: "demo_btc".to_string(),
            token_identifier: Some("bitcoin".to_string()),
            canonical_id: Some(CanonicalId::from_coingecko("bitcoin", "bitcoin")),
            token_symbol: "BTC".to_string(),
            token_name: "Bitcoin".to_string(),
            chain: "Bitcoin".to_string(),
            quantity: Decimal::from_str("0.5").unwrap(),
            entry_price_usd: Decimal::from_str("42000").unwrap(),
            entry_date: Some("2024-01-15".to_string()),
            user_note: Some("Demo position".to_string()),
            user_tags: vec!["demo".to_string()],
            created_at: now,
            updated_at: now,
        },
        Position {
            id: "demo_eth".to_string(),
            token_identifier: Some("ethereum".to_string()),
            canonical_id: Some(CanonicalId::from_coingecko("ethereum", "ethereum")),
            token_symbol: "ETH".to_string(),
            token_name: "Ethereum".to_string(),
            chain: "Ethereum".to_string(),
            quantity: Decimal::from_str("5.0").unwrap(),
            entry_price_usd: Decimal::from_str("2200").unwrap(),
            entry_date: Some("2024-02-01".to_string()),
            user_note: Some("Demo position".to_string()),
            user_tags: vec!["demo".to_string()],
            created_at: now,
            updated_at: now,
        },
        Position {
            id: "demo_sol".to_string(),
            token_identifier: Some("solana".to_string()),
            canonical_id: Some(CanonicalId::from_coingecko("solana", "solana")),
            token_symbol: "SOL".to_string(),
            token_name: "Solana".to_string(),
            chain: "Solana".to_string(),
            quantity: Decimal::from_str("50").unwrap(),
            entry_price_usd: Decimal::from_str("85").unwrap(),
            entry_date: Some("2024-03-10".to_string()),
            user_note: Some("Demo position".to_string()),
            user_tags: vec!["demo".to_string()],
            created_at: now,
            updated_at: now,
        },
        Position {
            id: "demo_usdc".to_string(),
            token_identifier: Some("usd-coin".to_string()),
            canonical_id: Some(CanonicalId::from_coingecko("usd-coin", "usd-coin")),
            token_symbol: "USDC".to_string(),
            token_name: "USD Coin".to_string(),
            chain: "Ethereum".to_string(),
            quantity: Decimal::from_str("1000").unwrap(),
            entry_price_usd: Decimal::from_str("1").unwrap(),
            entry_date: Some("2024-01-01".to_string()),
            user_note: Some("Demo position".to_string()),
            user_tags: vec!["demo".to_string()],
            created_at: now,
            updated_at: now,
        },
    ]
}

fn load_state() -> AppState {
    get_typed_state(|bytes| bincode::deserialize(bytes)).unwrap_or_default()
}

fn save_state(state: &AppState) {
    if let Ok(bytes) = bincode::serialize(state) {
        set_state(&bytes);
    }
}

/// Migrates existing positions to include canonical IDs
fn migrate_positions_to_canonical(state: &mut AppState) {
    let mut migrated = false;
    for position in state.positions.values_mut() {
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

fn get_current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn get_current_date() -> String {
    let now = get_current_timestamp();
    let days_since_epoch = now / 86400;
    // Simple date calculation (approximate, but good enough for daily snapshots)
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(28))
}

// ============================================================================
// CoinGecko Integration
// ============================================================================

fn get_exposure_category(symbol: &str, coingecko_id: Option<&str>) -> (ExposureCategory, ConfidenceLevel, String) {
    let symbol_upper = symbol.to_uppercase();
    let cg_id = coingecko_id.unwrap_or("");

    // ETH exposure
    if symbol_upper == "ETH" || cg_id == "ethereum" {
        return (ExposureCategory::ETH, ConfidenceLevel::High, "Direct ETH holding".to_string());
    }
    if symbol_upper == "WETH" || cg_id == "weth" {
        return (ExposureCategory::ETH, ConfidenceLevel::Medium, "Wrapped ETH (1:1 backing)".to_string());
    }
    if symbol_upper == "STETH" || cg_id == "staked-ether" {
        return (ExposureCategory::ETH, ConfidenceLevel::Medium, "Staked ETH (Lido)".to_string());
    }
    if symbol_upper == "RETH" || cg_id == "rocket-pool-eth" {
        return (ExposureCategory::ETH, ConfidenceLevel::Medium, "Staked ETH (Rocket Pool)".to_string());
    }

    // BTC exposure
    if symbol_upper == "BTC" || cg_id == "bitcoin" {
        return (ExposureCategory::BTC, ConfidenceLevel::High, "Direct BTC holding".to_string());
    }
    if symbol_upper == "WBTC" || cg_id == "wrapped-bitcoin" {
        return (ExposureCategory::BTC, ConfidenceLevel::Medium, "Wrapped BTC (custodial backing)".to_string());
    }

    // Stablecoins
    let stablecoins = ["USDC", "USDT", "DAI", "FRAX", "TUSD", "BUSD", "LUSD", "GUSD", "USDP", "PYUSD"];
    if stablecoins.contains(&symbol_upper.as_str()) {
        return (ExposureCategory::Stablecoins, ConfidenceLevel::High, format!("{} stablecoin", symbol_upper));
    }

    // Default: Other
    (ExposureCategory::Other, ConfidenceLevel::Low, "Unclassified asset".to_string())
}

fn is_stablecoin(symbol: &str) -> bool {
    let stablecoins = ["USDC", "USDT", "DAI", "FRAX", "TUSD", "BUSD", "LUSD", "GUSD", "USDP", "PYUSD"];
    stablecoins.contains(&symbol.to_uppercase().as_str())
}

#[derive(Debug, Deserialize)]
struct CoinGeckoSearchResponse {
    coins: Vec<CoinGeckoSearchCoin>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoSearchCoin {
    id: String,
    symbol: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoPrice {
    usd: Option<f64>,
    usd_24h_change: Option<f64>,
    usd_market_cap: Option<f64>,
}

fn search_coingecko_tokens(query: &str) -> Vec<TokenSearchResult> {
    let url = format!(
        "https://api.coingecko.com/api/v3/search?query={}&x_cg_demo_api_key={}",
        url_encode(query), COINGECKO_API_KEY
    );

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30,
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let search_response: CoinGeckoSearchResponse =
                            serde_json::from_slice(response.body()).unwrap_or(CoinGeckoSearchResponse { coins: vec![] });
                        search_response.coins.into_iter()
                            .take(10)
                            .map(|c| TokenSearchResult {
                                id: c.id,
                                symbol: c.symbol,
                                name: c.name,
                            })
                            .collect()
                    } else {
                        println!("CoinGecko search error: {}", response.status());
                        vec![]
                    }
                }
                Err(e) => {
                    println!("HTTP request error: {:?}", e);
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

fn fetch_top_tokens_from_coingecko() -> Vec<TopToken> {
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=25&page=1&sparkline=false&x_cg_demo_api_key={}",
        COINGECKO_API_KEY
    );

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30,
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let coins: Vec<CoinGeckoMarketCoin> =
                            serde_json::from_slice(response.body()).unwrap_or_default();
                        coins.into_iter()
                            .filter_map(|c| {
                                Some(TopToken {
                                    id: c.id,
                                    symbol: c.symbol.to_uppercase(),
                                    name: c.name,
                                    current_price: c.current_price?,
                                    price_change_percentage_24h: c.price_change_percentage_24h.unwrap_or(0.0),
                                    market_cap_rank: c.market_cap_rank.unwrap_or(0),
                                })
                            })
                            .collect()
                    } else {
                        vec![]
                    }
                }
                Err(_) => vec![],
            }
        }
        Err(_) => vec![],
    }
}

fn fetch_prices_from_coingecko(ids: &[String]) -> HashMap<String, PriceData> {
    if ids.is_empty() {
        return HashMap::new();
    }

    let ids_str = ids.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true&include_market_cap=true&x_cg_demo_api_key={}",
        ids_str, COINGECKO_API_KEY
    );

    let now = get_current_timestamp();

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30,
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let prices: HashMap<String, CoinGeckoPrice> =
                            serde_json::from_slice(response.body()).unwrap_or_default();
                        prices.into_iter()
                            .filter_map(|(id, p)| {
                                p.usd.map(|price_usd| (id, PriceData {
                                    price_usd,
                                    price_change_24h: p.usd_24h_change,
                                    market_cap: p.usd_market_cap,
                                    timestamp: now,
                                    is_stale: false,
                                    volume_24h: None,
                                    liquidity_usd: None,
                                    price_source: PriceSource::CoinGecko,
                                    fdv: None,
                                }))
                            })
                            .collect()
                    } else {
                        println!("CoinGecko API error: {}", response.status());
                        HashMap::new()
                    }
                }
                Err(e) => {
                    println!("HTTP request error: {:?}", e);
                    HashMap::new()
                }
            }
        }
        Err(e) => {
            println!("URL parse error: {:?}", e);
            HashMap::new()
        }
    }
}

// ============================================================================
// DeFi Llama Integration
// ============================================================================

#[derive(Debug, Deserialize)]
struct DefiLlamaChain {
    gecko_id: Option<String>,
    tvl: Option<f64>,
    #[serde(rename = "tokenSymbol")]
    token_symbol: Option<String>,
    name: String,
    #[serde(rename = "chainId")]
    chain_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct DefiLlamaPriceResponse {
    coins: HashMap<String, DefiLlamaPrice>,
}

#[derive(Debug, Deserialize)]
struct DefiLlamaPrice {
    price: Option<f64>,
    symbol: Option<String>,
    timestamp: Option<u64>,
    confidence: Option<f64>,
}

fn fetch_chain_tvl_from_defillama() -> HashMap<String, ChainTvlData> {
    let url = format!("{}/v2/chains", DEFILLAMA_API_BASE);
    let now = get_current_timestamp();

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30,
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let chains: Vec<DefiLlamaChain> =
                            serde_json::from_slice(response.body()).unwrap_or_default();
                        chains.into_iter()
                            .filter_map(|c| {
                                c.tvl.map(|tvl| (c.name.to_lowercase(), ChainTvlData {
                                    chain: c.name.clone(),
                                    tvl,
                                    tvl_change_24h: None,
                                    protocols_count: None,
                                    timestamp: now,
                                }))
                            })
                            .collect()
                    } else {
                        println!("DeFi Llama chains API error: {}", response.status());
                        HashMap::new()
                    }
                }
                Err(e) => {
                    println!("DeFi Llama HTTP error: {:?}", e);
                    HashMap::new()
                }
            }
        }
        Err(e) => {
            println!("DeFi Llama URL parse error: {:?}", e);
            HashMap::new()
        }
    }
}

fn fetch_prices_from_defillama(coins: &[(String, String)]) -> HashMap<String, PriceData> {
    // coins is vec of (chain, address) tuples, e.g., ("ethereum", "0x...")
    if coins.is_empty() {
        return HashMap::new();
    }

    let coin_ids: Vec<String> = coins.iter()
        .map(|(chain, addr)| format!("{}:{}", chain, addr))
        .collect();
    let coins_param = coin_ids.join(",");

    let url = format!("{}/prices/current/{}", DEFILLAMA_COINS_API, coins_param);
    let now = get_current_timestamp();

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30,
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let price_response: DefiLlamaPriceResponse =
                            serde_json::from_slice(response.body()).unwrap_or(DefiLlamaPriceResponse { coins: HashMap::new() });
                        price_response.coins.into_iter()
                            .filter_map(|(id, p)| {
                                p.price.map(|price_usd| (id, PriceData {
                                    price_usd,
                                    price_change_24h: None,
                                    market_cap: None,
                                    timestamp: now,
                                    is_stale: false,
                                    volume_24h: None,
                                    liquidity_usd: None,
                                    price_source: PriceSource::DefiLlama,
                                    fdv: None,
                                }))
                            })
                            .collect()
                    } else {
                        println!("DeFi Llama price API error: {}", response.status());
                        HashMap::new()
                    }
                }
                Err(e) => {
                    println!("DeFi Llama HTTP error: {:?}", e);
                    HashMap::new()
                }
            }
        }
        Err(e) => {
            println!("DeFi Llama URL parse error: {:?}", e);
            HashMap::new()
        }
    }
}

// ============================================================================
// Dexscreener Integration
// ============================================================================

#[derive(Debug, Deserialize)]
struct DexscreenerResponse {
    pairs: Option<Vec<DexscreenerPair>>,
}

#[derive(Debug, Deserialize)]
struct DexscreenerPair {
    #[serde(rename = "chainId")]
    chain_id: String,
    #[serde(rename = "dexId")]
    dex_id: String,
    #[serde(rename = "pairAddress")]
    pair_address: String,
    #[serde(rename = "baseToken")]
    base_token: DexscreenerToken,
    #[serde(rename = "quoteToken")]
    quote_token: DexscreenerToken,
    #[serde(rename = "priceUsd")]
    price_usd: Option<String>,
    #[serde(rename = "priceChange")]
    price_change: Option<DexscreenerPriceChange>,
    volume: Option<DexscreenerVolume>,
    liquidity: Option<DexscreenerLiquidity>,
    fdv: Option<f64>,
    #[serde(rename = "pairCreatedAt")]
    pair_created_at: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct DexscreenerToken {
    address: String,
    name: String,
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct DexscreenerPriceChange {
    h24: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DexscreenerVolume {
    h24: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DexscreenerLiquidity {
    usd: Option<f64>,
}

fn search_dexscreener(query: &str) -> Vec<DexPairData> {
    let url = format!("{}/search?q={}", DEXSCREENER_API_BASE, url_encode(query));

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30,
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let dex_response: DexscreenerResponse =
                            serde_json::from_slice(response.body()).unwrap_or(DexscreenerResponse { pairs: None });

                        dex_response.pairs.unwrap_or_default().into_iter()
                            .take(10)
                            .map(|p| DexPairData {
                                pair_address: p.pair_address,
                                dex_name: p.dex_id,
                                chain: p.chain_id,
                                base_token_symbol: p.base_token.symbol,
                                base_token_name: p.base_token.name,
                                quote_token_symbol: p.quote_token.symbol,
                                price_usd: p.price_usd.and_then(|s| s.parse().ok()).unwrap_or(0.0),
                                price_change_24h: p.price_change.and_then(|pc| pc.h24),
                                volume_24h: p.volume.and_then(|v| v.h24).unwrap_or(0.0),
                                liquidity_usd: p.liquidity.and_then(|l| l.usd).unwrap_or(0.0),
                                fdv: p.fdv,
                                pair_created_at: p.pair_created_at,
                            })
                            .collect()
                    } else {
                        println!("Dexscreener search API error: {}", response.status());
                        vec![]
                    }
                }
                Err(e) => {
                    println!("Dexscreener HTTP error: {:?}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            println!("Dexscreener URL parse error: {:?}", e);
            vec![]
        }
    }
}

fn fetch_token_from_dexscreener(token_address: &str) -> Option<PriceData> {
    let url = format!("{}/tokens/{}", DEXSCREENER_API_BASE, url_encode(token_address));
    let now = get_current_timestamp();

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30,
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let dex_response: DexscreenerResponse =
                            serde_json::from_slice(response.body()).unwrap_or(DexscreenerResponse { pairs: None });

                        // Get the pair with highest liquidity
                        dex_response.pairs
                            .unwrap_or_default()
                            .into_iter()
                            .max_by(|a, b| {
                                let liq_a = a.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
                                let liq_b = b.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
                                liq_a.partial_cmp(&liq_b).unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .and_then(|p| {
                                p.price_usd.and_then(|price_str| price_str.parse::<f64>().ok())
                                    .map(|price_usd| PriceData {
                                        price_usd,
                                        price_change_24h: p.price_change.and_then(|pc| pc.h24),
                                        market_cap: None,
                                        timestamp: now,
                                        is_stale: false,
                                        volume_24h: p.volume.and_then(|v| v.h24),
                                        liquidity_usd: p.liquidity.and_then(|l| l.usd),
                                        price_source: PriceSource::Dexscreener,
                                        fdv: p.fdv,
                                    })
                            })
                    } else {
                        println!("Dexscreener token API error: {}", response.status());
                        None
                    }
                }
                Err(e) => {
                    println!("Dexscreener HTTP error: {:?}", e);
                    None
                }
            }
        }
        Err(e) => {
            println!("Dexscreener URL parse error: {:?}", e);
            None
        }
    }
}

// ============================================================================
// Analytics - Portfolio Calculations
// ============================================================================

fn get_position_price(position: &Position, state: &AppState) -> f64 {
    // Try canonical ID first (preferred for new positions)
    if let Some(ref canonical) = position.canonical_id {
        if let Some(price_data) = state.prices.get(&canonical.canonical) {
            return price_data.price_usd;
        }
        // Fallback: try coingecko_id directly if canonical lookup fails
        if let Some(ref cg_id) = canonical.coingecko_id {
            if let Some(price_data) = state.prices.get(cg_id) {
                return price_data.price_usd;
            }
        }
    }

    // Legacy fallback: check old token_identifier field (for backwards compatibility)
    if let Some(ref cg_id) = position.token_identifier {
        if let Some(price_data) = state.prices.get(cg_id) {
            return price_data.price_usd;
        }
    }

    // Stablecoin fallback
    if is_stablecoin(&position.token_symbol) {
        return 1.0;
    }

    // Default to entry price if no current price available
    position.entry_price_usd.to_f64().unwrap_or(0.0)
}

fn calculate_positions_with_derived(state: &AppState) -> Vec<PositionWithDerived> {
    let now = get_current_timestamp();
    let stale_threshold = 5 * 60; // 5 minutes

    // First pass: calculate total value
    let total_value: f64 = state.positions.values()
        .map(|p| {
            let price = get_position_price(p, state);
            p.quantity.to_f64().unwrap_or(0.0) * price
        })
        .sum();

    // Second pass: build derived data
    state.positions.values().map(|position| {
        let current_price = get_position_price(position, state);
        let quantity = position.quantity.to_f64().unwrap_or(0.0);
        let entry_price = position.entry_price_usd.to_f64().unwrap_or(0.0);
        let position_value = quantity * current_price;
        let cost_basis = quantity * entry_price;
        let unrealized_pnl = position_value - cost_basis;
        let unrealized_pnl_percent = if cost_basis > 0.0 {
            (unrealized_pnl / cost_basis) * 100.0
        } else {
            0.0
        };
        let allocation_percent = if total_value > 0.0 {
            (position_value / total_value) * 100.0
        } else {
            0.0
        };

        let price_change_24h = position.token_identifier.as_ref()
            .and_then(|cg_id| state.prices.get(cg_id))
            .and_then(|p| p.price_change_24h);

        PositionWithDerived {
            position: position.clone(),
            current_price_usd: current_price,
            price_change_24h,
            position_value_usd: position_value,
            unrealized_pnl_usd: unrealized_pnl,
            unrealized_pnl_percent,
            allocation_percent,
        }
    }).collect()
}

fn calculate_portfolio_summary(positions: &[PositionWithDerived], state: &AppState) -> PortfolioSummary {
    let total_value: f64 = positions.iter().map(|p| p.position_value_usd).sum();
    let total_cost: f64 = positions.iter().map(|p| {
        let qty = p.position.quantity.to_f64().unwrap_or(0.0);
        let entry = p.position.entry_price_usd.to_f64().unwrap_or(0.0);
        qty * entry
    }).sum();
    let total_pnl = total_value - total_cost;
    let total_pnl_percent = if total_cost > 0.0 {
        (total_pnl / total_cost) * 100.0
    } else {
        0.0
    };

    // Top 3 positions
    let mut sorted_positions = positions.to_vec();
    sorted_positions.sort_by(|a, b| b.allocation_percent.partial_cmp(&a.allocation_percent).unwrap_or(std::cmp::Ordering::Equal));
    let top_positions: Vec<TopPosition> = sorted_positions.iter()
        .take(3)
        .map(|p| TopPosition {
            symbol: p.position.token_symbol.clone(),
            name: p.position.token_name.clone(),
            allocation_percent: p.allocation_percent,
        })
        .collect();

    let largest_position_percent = sorted_positions.first()
        .map(|p| p.allocation_percent)
        .unwrap_or(0.0);

    // Chain summary
    let mut chain_values: HashMap<String, f64> = HashMap::new();
    for p in positions {
        *chain_values.entry(p.position.chain.clone()).or_insert(0.0) += p.position_value_usd;
    }
    let chain_summary: Vec<ChainExposure> = chain_values.into_iter()
        .map(|(chain, value)| ChainExposure {
            chain,
            value_usd: value,
            percentage: if total_value > 0.0 { (value / total_value) * 100.0 } else { 0.0 },
        })
        .collect();

    // Biggest 24h mover (weighted by allocation)
    let biggest_mover = positions.iter()
        .filter_map(|p| {
            p.price_change_24h.map(|change| {
                let weighted_impact = change * (p.allocation_percent / 100.0);
                MoverInfo {
                    symbol: p.position.token_symbol.clone(),
                    name: p.position.token_name.clone(),
                    change_24h_percent: change,
                    weighted_impact,
                }
            })
        })
        .max_by(|a, b| a.weighted_impact.abs().partial_cmp(&b.weighted_impact.abs()).unwrap_or(std::cmp::Ordering::Equal));

    // Last price update
    let last_price_update = state.prices.values()
        .map(|p| p.timestamp)
        .max();

    PortfolioSummary {
        total_value_usd: total_value,
        total_unrealized_pnl_usd: total_pnl,
        total_unrealized_pnl_percent: total_pnl_percent,
        top_positions,
        largest_position_percent,
        chain_summary,
        biggest_24h_mover: biggest_mover,
        last_price_update,
    }
}

// ============================================================================
// Analytics - Insight Generation
// ============================================================================

fn generate_insights(
    positions: &[PositionWithDerived],
    summary: &PortfolioSummary,
    exposure: &ExposureResponse,
    snapshots: &[DailySnapshot],
    chain_tvl: &HashMap<String, ChainTvlData>,
    prices: &HashMap<String, PriceData>,
) -> InsightsResponse {
    let mut insights: Vec<Insight> = Vec::new();
    let now = get_current_timestamp();
    let mut health_score: i32 = 100;

    // -------------------------------------------------------------------------
    // 1. CONCENTRATION INSIGHTS
    // -------------------------------------------------------------------------

    // High concentration warning (single position > 50%)
    if summary.largest_position_percent > 50.0 {
        let top_symbol = summary.top_positions.first()
            .map(|p| p.symbol.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        insights.push(Insight {
            id: "concentration_high".to_string(),
            insight_type: InsightType::Concentration,
            priority: InsightPriority::High,
            title: "High Concentration Risk".to_string(),
            description: format!(
                "{} represents {:.1}% of your portfolio. Consider diversifying to reduce risk.",
                top_symbol,
                summary.largest_position_percent
            ),
            icon: "⚠️".to_string(),
            color: "red".to_string(),
            action_label: Some("View Exposure".to_string()),
            action_url: Some("/exposure".to_string()),
            metadata: {
                let mut m = HashMap::new();
                m.insert("concentration".to_string(), format!("{:.1}", summary.largest_position_percent));
                m.insert("asset".to_string(), top_symbol);
                m
            },
        });
        health_score -= 15;
    } else if summary.largest_position_percent > 35.0 {
        let top_symbol = summary.top_positions.first()
            .map(|p| p.symbol.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        insights.push(Insight {
            id: "concentration_moderate".to_string(),
            insight_type: InsightType::Concentration,
            priority: InsightPriority::Medium,
            title: "Moderate Concentration".to_string(),
            description: format!(
                "Your {} position is {:.1}% of portfolio. This is within acceptable range but worth monitoring.",
                top_symbol,
                summary.largest_position_percent
            ),
            icon: "💡".to_string(),
            color: "lime".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
        health_score -= 5;
    } else if positions.len() >= 3 {
        insights.push(Insight {
            id: "well_diversified".to_string(),
            insight_type: InsightType::Diversification,
            priority: InsightPriority::Low,
            title: "Well Diversified".to_string(),
            description: format!(
                "Your portfolio is spread across {} positions with no single asset dominating.",
                positions.len()
            ),
            icon: "✅".to_string(),
            color: "lime".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
    }

    // -------------------------------------------------------------------------
    // 2. PERFORMANCE INSIGHTS
    // -------------------------------------------------------------------------

    // Strong gains
    if summary.total_unrealized_pnl_percent > 20.0 {
        insights.push(Insight {
            id: "strong_gains".to_string(),
            insight_type: InsightType::Performance,
            priority: InsightPriority::Medium,
            title: "Strong Portfolio Performance".to_string(),
            description: format!(
                "Your portfolio is up {:.1}% (${:.2}). Consider taking partial profits to lock in gains.",
                summary.total_unrealized_pnl_percent,
                summary.total_unrealized_pnl_usd
            ),
            icon: "🚀".to_string(),
            color: "lime".to_string(),
            action_label: None,
            action_url: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("pnl_percent".to_string(), format!("{:.1}", summary.total_unrealized_pnl_percent));
                m.insert("pnl_usd".to_string(), format!("{:.2}", summary.total_unrealized_pnl_usd));
                m
            },
        });
    } else if summary.total_unrealized_pnl_percent < -15.0 {
        insights.push(Insight {
            id: "significant_loss".to_string(),
            insight_type: InsightType::Performance,
            priority: InsightPriority::High,
            title: "Portfolio Down Significantly".to_string(),
            description: format!(
                "Your portfolio is down {:.1}%. Review your positions and consider if your investment thesis still holds.",
                summary.total_unrealized_pnl_percent.abs()
            ),
            icon: "📉".to_string(),
            color: "red".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
        health_score -= 10;
    } else if summary.total_unrealized_pnl_percent > 5.0 {
        insights.push(Insight {
            id: "positive_performance".to_string(),
            insight_type: InsightType::Performance,
            priority: InsightPriority::Low,
            title: "Positive Returns".to_string(),
            description: format!(
                "You're in profit by {:.1}%. Keep monitoring and stick to your strategy.",
                summary.total_unrealized_pnl_percent
            ),
            icon: "📈".to_string(),
            color: "blue".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
    }

    // -------------------------------------------------------------------------
    // 3. 24H PRICE MOVEMENT INSIGHTS
    // -------------------------------------------------------------------------

    if let Some(mover) = &summary.biggest_24h_mover {
        if mover.change_24h_percent.abs() > 10.0 {
            let direction = if mover.change_24h_percent > 0.0 { "up" } else { "down" };
            let icon = if mover.change_24h_percent > 0.0 { "🔥" } else { "❄️" };
            let color = if mover.change_24h_percent > 0.0 { "lime" } else { "red" };

            insights.push(Insight {
                id: "significant_mover".to_string(),
                insight_type: InsightType::PriceMovement,
                priority: InsightPriority::Medium,
                title: format!("{} Moved {:.1}%", mover.symbol, mover.change_24h_percent.abs()),
                description: format!(
                    "{} is {} {:.1}% in the last 24 hours. This impacts your portfolio by approximately {:.2}%.",
                    mover.symbol,
                    direction,
                    mover.change_24h_percent.abs(),
                    mover.weighted_impact.abs()
                ),
                icon: icon.to_string(),
                color: color.to_string(),
                action_label: None,
                action_url: None,
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("symbol".to_string(), mover.symbol.clone());
                    m.insert("change_24h".to_string(), format!("{:.1}", mover.change_24h_percent));
                    m
                },
            });
        }
    }

    // -------------------------------------------------------------------------
    // 4. CHAIN DIVERSIFICATION INSIGHTS
    // -------------------------------------------------------------------------

    if summary.chain_summary.len() == 1 {
        let chain = summary.chain_summary.first()
            .map(|c| c.chain.clone())
            .unwrap_or_else(|| "one chain".to_string());

        insights.push(Insight {
            id: "single_chain".to_string(),
            insight_type: InsightType::Diversification,
            priority: InsightPriority::Medium,
            title: "Single Chain Exposure".to_string(),
            description: format!(
                "All your holdings are on {}. Consider spreading across multiple chains to reduce platform risk.",
                chain
            ),
            icon: "🔗".to_string(),
            color: "yellow".to_string(),
            action_label: Some("Add Position".to_string()),
            action_url: Some("/add".to_string()),
            metadata: HashMap::new(),
        });
        health_score -= 10;
    } else if summary.chain_summary.len() >= 3 {
        insights.push(Insight {
            id: "multi_chain".to_string(),
            insight_type: InsightType::Diversification,
            priority: InsightPriority::Low,
            title: "Multi-Chain Strategy".to_string(),
            description: format!(
                "Your portfolio spans {} different blockchains, reducing single-point-of-failure risk.",
                summary.chain_summary.len()
            ),
            icon: "🌐".to_string(),
            color: "lime".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
        health_score += 5;
    }

    // -------------------------------------------------------------------------
    // 5. ASSET CATEGORY INSIGHTS
    // -------------------------------------------------------------------------

    // Check stablecoin allocation
    let stablecoin_pct = exposure.underlying_exposure.iter()
        .find(|e| e.category == "Stablecoins")
        .map(|e| e.percentage)
        .unwrap_or(0.0);

    if stablecoin_pct > 50.0 {
        insights.push(Insight {
            id: "high_stablecoin".to_string(),
            insight_type: InsightType::Opportunity,
            priority: InsightPriority::Low,
            title: "High Stablecoin Allocation".to_string(),
            description: format!(
                "{:.0}% of your portfolio is in stablecoins. You're positioned defensively - good for uncertain markets.",
                stablecoin_pct
            ),
            icon: "🛡️".to_string(),
            color: "blue".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
    } else if stablecoin_pct == 0.0 && summary.total_value_usd > 1000.0 {
        insights.push(Insight {
            id: "no_stablecoins".to_string(),
            insight_type: InsightType::Rebalancing,
            priority: InsightPriority::Low,
            title: "No Stablecoin Reserve".to_string(),
            description: "Consider holding some stablecoins for buying opportunities during dips.".to_string(),
            icon: "💵".to_string(),
            color: "blue".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
    }

    // Check BTC/ETH exposure (majors)
    let btc_pct = exposure.underlying_exposure.iter()
        .find(|e| e.category == "BTC")
        .map(|e| e.percentage)
        .unwrap_or(0.0);
    let eth_pct = exposure.underlying_exposure.iter()
        .find(|e| e.category == "ETH")
        .map(|e| e.percentage)
        .unwrap_or(0.0);
    let majors_pct = btc_pct + eth_pct;

    if majors_pct > 80.0 && positions.len() >= 2 {
        insights.push(Insight {
            id: "blue_chip_focus".to_string(),
            insight_type: InsightType::General,
            priority: InsightPriority::Low,
            title: "Blue Chip Focused".to_string(),
            description: format!(
                "{:.0}% of your portfolio is in BTC and ETH - a conservative, lower-risk approach.",
                majors_pct
            ),
            icon: "🏛️".to_string(),
            color: "lime".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
        health_score += 5;
    }

    // -------------------------------------------------------------------------
    // 6. PORTFOLIO GROWTH INSIGHTS (based on snapshots)
    // -------------------------------------------------------------------------

    if snapshots.len() >= 7 {
        let recent_snapshots: Vec<&DailySnapshot> = snapshots.iter().rev().take(7).collect();
        if let (Some(newest), Some(oldest)) = (recent_snapshots.first(), recent_snapshots.last()) {
            let weekly_change = if oldest.total_value_usd > 0.0 {
                ((newest.total_value_usd - oldest.total_value_usd) / oldest.total_value_usd) * 100.0
            } else {
                0.0
            };

            if weekly_change > 10.0 {
                insights.push(Insight {
                    id: "weekly_growth".to_string(),
                    insight_type: InsightType::Performance,
                    priority: InsightPriority::Medium,
                    title: "Strong Weekly Growth".to_string(),
                    description: format!(
                        "Your portfolio grew {:.1}% over the past week. Excellent momentum!",
                        weekly_change
                    ),
                    icon: "📊".to_string(),
                    color: "lime".to_string(),
                    action_label: None,
                    action_url: None,
                    metadata: HashMap::new(),
                });
            } else if weekly_change < -10.0 {
                insights.push(Insight {
                    id: "weekly_decline".to_string(),
                    insight_type: InsightType::Performance,
                    priority: InsightPriority::Medium,
                    title: "Weekly Decline".to_string(),
                    description: format!(
                        "Your portfolio declined {:.1}% this week. Market conditions may be challenging.",
                        weekly_change.abs()
                    ),
                    icon: "📉".to_string(),
                    color: "yellow".to_string(),
                    action_label: None,
                    action_url: None,
                    metadata: HashMap::new(),
                });
            }
        }
    }

    // -------------------------------------------------------------------------
    // 7. SMALL POSITION INSIGHTS
    // -------------------------------------------------------------------------

    let dust_positions: Vec<&PositionWithDerived> = positions.iter()
        .filter(|p| p.allocation_percent < 1.0 && p.position_value_usd < 50.0)
        .collect();

    if dust_positions.len() >= 3 {
        insights.push(Insight {
            id: "dust_positions".to_string(),
            insight_type: InsightType::Rebalancing,
            priority: InsightPriority::Low,
            title: "Small Positions Detected".to_string(),
            description: format!(
                "You have {} positions worth less than $50 each. Consider consolidating for easier management.",
                dust_positions.len()
            ),
            icon: "🧹".to_string(),
            color: "blue".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
    }

    // -------------------------------------------------------------------------
    // 8. PORTFOLIO SIZE INSIGHTS
    // -------------------------------------------------------------------------

    if positions.is_empty() {
        insights.push(Insight {
            id: "empty_portfolio".to_string(),
            insight_type: InsightType::General,
            priority: InsightPriority::High,
            title: "Get Started".to_string(),
            description: "Add your first position to start tracking your crypto portfolio.".to_string(),
            icon: "🚀".to_string(),
            color: "blue".to_string(),
            action_label: Some("Add Position".to_string()),
            action_url: Some("/add".to_string()),
            metadata: HashMap::new(),
        });
        health_score = 0;
    } else if positions.len() == 1 {
        insights.push(Insight {
            id: "single_position".to_string(),
            insight_type: InsightType::Diversification,
            priority: InsightPriority::Medium,
            title: "Single Position Portfolio".to_string(),
            description: "Your portfolio has only one position. Adding more assets can help reduce risk.".to_string(),
            icon: "🎯".to_string(),
            color: "yellow".to_string(),
            action_label: Some("Add Position".to_string()),
            action_url: Some("/add".to_string()),
            metadata: HashMap::new(),
        });
        health_score -= 20;
    }

    // -------------------------------------------------------------------------
    // 9. LIQUIDITY INSIGHTS (from Dexscreener data)
    // -------------------------------------------------------------------------

    // Find positions with low liquidity (warning for potential slippage)
    let low_liquidity_positions: Vec<&PositionWithDerived> = positions.iter()
        .filter(|p| {
            if let Some(cg_id) = &p.position.token_identifier {
                if let Some(price_data) = prices.get(cg_id) {
                    // Check if position value > 10% of liquidity
                    if let Some(liq) = price_data.liquidity_usd {
                        return liq > 0.0 && p.position_value_usd > liq * 0.1;
                    }
                }
            }
            false
        })
        .collect();

    if !low_liquidity_positions.is_empty() {
        let symbols: Vec<String> = low_liquidity_positions.iter()
            .take(3)
            .map(|p| p.position.token_symbol.clone())
            .collect();

        insights.push(Insight {
            id: "low_liquidity".to_string(),
            insight_type: InsightType::RiskAlert,
            priority: InsightPriority::High,
            title: "Low Liquidity Warning".to_string(),
            description: format!(
                "{} may have liquidity issues. Large sells could cause significant slippage.",
                symbols.join(", ")
            ),
            icon: "💧".to_string(),
            color: "red".to_string(),
            action_label: None,
            action_url: None,
            metadata: HashMap::new(),
        });
        health_score -= 10;
    }

    // -------------------------------------------------------------------------
    // 10. CHAIN TVL INSIGHTS (from DeFi Llama data)
    // -------------------------------------------------------------------------

    // Check if user is on high-TVL chains
    if !chain_tvl.is_empty() {
        let user_chains: Vec<String> = summary.chain_summary.iter()
            .map(|c| c.chain.to_lowercase())
            .collect();

        // Find top TVL chains
        let mut tvl_vec: Vec<(&String, &ChainTvlData)> = chain_tvl.iter().collect();
        tvl_vec.sort_by(|a, b| b.1.tvl.partial_cmp(&a.1.tvl).unwrap_or(std::cmp::Ordering::Equal));
        let top_5_chains: Vec<String> = tvl_vec.iter().take(5).map(|(k, _)| k.to_lowercase()).collect();

        // Check if user's chains are in top 5 TVL
        let on_top_chains = user_chains.iter().filter(|c| top_5_chains.contains(&c.to_lowercase())).count();

        if on_top_chains == user_chains.len() && !user_chains.is_empty() {
            insights.push(Insight {
                id: "top_chains".to_string(),
                insight_type: InsightType::General,
                priority: InsightPriority::Low,
                title: "Premium Chain Exposure".to_string(),
                description: "All your holdings are on top-TVL chains with strong DeFi ecosystems.".to_string(),
                icon: "🏆".to_string(),
                color: "lime".to_string(),
                action_label: None,
                action_url: None,
                metadata: HashMap::new(),
            });
            health_score += 5;
        }
    }

    // -------------------------------------------------------------------------
    // 11. VOLUME INSIGHTS (high volume = high interest)
    // -------------------------------------------------------------------------

    let high_volume_positions: Vec<(&PositionWithDerived, f64)> = positions.iter()
        .filter_map(|p| {
            p.position.token_identifier.as_ref()
                .and_then(|cg_id| prices.get(cg_id))
                .and_then(|price_data| price_data.volume_24h)
                .filter(|vol| *vol > 10_000_000.0) // $10M+ volume
                .map(|vol| (p, vol))
        })
        .collect();

    if !high_volume_positions.is_empty() {
        let highest = high_volume_positions.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((pos, vol)) = highest {
            let vol_display = if *vol >= 1_000_000_000.0 {
                format!("${:.1}B", vol / 1_000_000_000.0)
            } else {
                format!("${:.0}M", vol / 1_000_000.0)
            };

            insights.push(Insight {
                id: "high_volume".to_string(),
                insight_type: InsightType::General,
                priority: InsightPriority::Low,
                title: "High Trading Activity".to_string(),
                description: format!(
                    "{} has {} in 24h trading volume, indicating strong market interest.",
                    pos.position.token_symbol,
                    vol_display
                ),
                icon: "📈".to_string(),
                color: "blue".to_string(),
                action_label: None,
                action_url: None,
                metadata: HashMap::new(),
            });
        }
    }

    // -------------------------------------------------------------------------
    // SORT AND LIMIT INSIGHTS
    // -------------------------------------------------------------------------

    // Sort by priority (High > Medium > Low)
    insights.sort_by(|a, b| {
        let priority_order = |p: &InsightPriority| match p {
            InsightPriority::High => 0,
            InsightPriority::Medium => 1,
            InsightPriority::Low => 2,
        };
        priority_order(&a.priority).cmp(&priority_order(&b.priority))
    });

    // Limit to top 6 insights
    insights.truncate(6);

    // Ensure health score is within bounds
    let final_health_score = health_score.max(0).min(100) as u8;

    InsightsResponse {
        insights,
        generated_at: now,
        portfolio_health_score: final_health_score,
    }
}

fn calculate_exposure(positions: &[PositionWithDerived]) -> ExposureResponse {
    let total_value: f64 = positions.iter().map(|p| p.position_value_usd).sum();

    // Underlying exposure by category
    let mut category_values: HashMap<ExposureCategory, (f64, ConfidenceLevel, Vec<String>)> = HashMap::new();

    for p in positions {
        let (category, confidence, note) = get_exposure_category(
            &p.position.token_symbol,
            p.position.token_identifier.as_deref()
        );

        let entry = category_values.entry(category.clone()).or_insert((0.0, confidence.clone(), vec![]));
        entry.0 += p.position_value_usd;
        // Use lowest confidence if mixed
        if confidence == ConfidenceLevel::Low || (confidence == ConfidenceLevel::Medium && entry.1 == ConfidenceLevel::High) {
            entry.1 = confidence;
        }
        if !note.is_empty() && !entry.2.contains(&note) {
            entry.2.push(note);
        }
    }

    let underlying_exposure: Vec<ExposureEntry> = category_values.into_iter()
        .map(|(category, (value, confidence, notes))| ExposureEntry {
            category: category.as_str().to_string(),
            value_usd: value,
            percentage: if total_value > 0.0 { (value / total_value) * 100.0 } else { 0.0 },
            confidence,
            notes: notes.join("; "),
        })
        .collect();

    // Chain exposure
    let mut chain_values: HashMap<String, f64> = HashMap::new();
    for p in positions {
        *chain_values.entry(p.position.chain.clone()).or_insert(0.0) += p.position_value_usd;
    }
    let chain_exposure: Vec<ChainExposure> = chain_values.into_iter()
        .map(|(chain, value)| ChainExposure {
            chain,
            value_usd: value,
            percentage: if total_value > 0.0 { (value / total_value) * 100.0 } else { 0.0 },
        })
        .collect();

    ExposureResponse {
        underlying_exposure,
        chain_exposure,
    }
}

// ============================================================================
// HTTP Response Helpers
// ============================================================================

fn send_json_response(status: StatusCode, body: impl Serialize) {
    let body_bytes = serde_json::to_vec(&body).unwrap_or_default();
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    http::server::send_response(status, Some(headers), body_bytes);
}

fn send_error_response(status: StatusCode, message: &str) {
    send_json_response(status, serde_json::json!({ "error": message }));
}

fn send_success_response() {
    send_json_response(StatusCode::OK, serde_json::json!({ "success": true }));
}

// ============================================================================
// Route Handlers
// ============================================================================

fn handle_get_holdings(state: &AppState) {
    let positions = calculate_positions_with_derived(state);
    let summary = calculate_portfolio_summary(&positions, state);

    let response = HoldingsResponse {
        positions,
        summary,
        snapshots: state.snapshots.clone(),
    };

    send_json_response(StatusCode::OK, response);
}

fn handle_get_exposure(state: &AppState) {
    let positions = calculate_positions_with_derived(state);
    let exposure = calculate_exposure(&positions);
    send_json_response(StatusCode::OK, exposure);
}

fn handle_get_insights(state: &AppState) {
    let positions = calculate_positions_with_derived(state);
    let summary = calculate_portfolio_summary(&positions, state);
    let exposure = calculate_exposure(&positions);
    let insights = generate_insights(
        &positions,
        &summary,
        &exposure,
        &state.snapshots,
        &state.chain_tvl,
        &state.prices,
    );
    send_json_response(StatusCode::OK, insights);
}

fn handle_add_position(state: &mut AppState, body: &[u8]) {
    let request: AddPositionRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(e) => return send_error_response(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e)),
    };

    // Validate input bounds
    if let Err(msg) = validate_add_position_input(&request) {
        return send_error_response(StatusCode::BAD_REQUEST, msg);
    }

    // Parse decimal values
    let quantity: Decimal = match request.quantity.parse() {
        Ok(q) if q > Decimal::ZERO => q,
        _ => return send_error_response(StatusCode::BAD_REQUEST, "Quantity must be greater than 0"),
    };

    let entry_price: Decimal = match request.entry_price_usd.parse() {
        Ok(p) if p >= Decimal::ZERO => p,
        _ => return send_error_response(StatusCode::BAD_REQUEST, "Entry price must be >= 0"),
    };

    let now = get_current_timestamp();
    let id = format!("{}_{}", request.token_symbol.to_lowercase(), now);

    // Resolve canonical identifier for consistent price lookups
    let canonical_id = resolve_canonical_id(
        request.token_identifier.as_deref(),
        &request.token_symbol,
        &request.chain,
    );

    let position = Position {
        id: id.clone(),
        token_identifier: request.token_identifier.clone(),
        canonical_id: Some(canonical_id),
        token_symbol: request.token_symbol.to_uppercase(),
        token_name: request.token_name,
        chain: request.chain,
        quantity,
        entry_price_usd: entry_price,
        entry_date: request.entry_date,
        user_note: request.user_note,
        user_tags: request.user_tags.unwrap_or_default(),
        created_at: now,
        updated_at: now,
    };

    // Fetch price for new position if CoinGecko ID provided
    if let Some(cg_id) = &position.token_identifier {
        if !state.prices.contains_key(cg_id) {
            let prices = fetch_prices_from_coingecko(&[cg_id.clone()]);
            for (id, price_data) in prices {
                // Store under raw CoinGecko ID (backwards compatibility)
                state.prices.insert(id.clone(), price_data.clone());
                // Also store under canonical key
                let canonical_key = format!("coingecko:{}", id.to_lowercase());
                state.prices.insert(canonical_key, price_data);
            }
            state.last_price_fetch = now;
        }
    }

    state.positions.insert(id.clone(), position);
    save_state(state);

    send_json_response(StatusCode::CREATED, serde_json::json!({ "id": id }));
}

fn handle_update_position(state: &mut AppState, position_id: &str, body: &[u8]) {
    let request: UpdatePositionRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(e) => return send_error_response(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e)),
    };

    // Validate input bounds
    if let Err(msg) = validate_update_position_input(&request) {
        return send_error_response(StatusCode::BAD_REQUEST, msg);
    }

    let position = match state.positions.get_mut(position_id) {
        Some(p) => p,
        None => return send_error_response(StatusCode::NOT_FOUND, "Position not found"),
    };

    if let Some(qty_str) = request.quantity {
        match qty_str.parse::<Decimal>() {
            Ok(q) if q > Decimal::ZERO => position.quantity = q,
            _ => return send_error_response(StatusCode::BAD_REQUEST, "Quantity must be greater than 0"),
        }
    }

    if let Some(price_str) = request.entry_price_usd {
        match price_str.parse::<Decimal>() {
            Ok(p) if p >= Decimal::ZERO => position.entry_price_usd = p,
            _ => return send_error_response(StatusCode::BAD_REQUEST, "Entry price must be >= 0"),
        }
    }

    if let Some(date) = request.entry_date {
        position.entry_date = Some(date);
    }

    if let Some(note) = request.user_note {
        position.user_note = Some(note);
    }

    if let Some(tags) = request.user_tags {
        position.user_tags = tags;
    }

    position.updated_at = get_current_timestamp();
    save_state(state);

    send_success_response();
}

fn handle_delete_position(state: &mut AppState, position_id: &str) {
    if state.positions.remove(position_id).is_some() {
        save_state(state);
        send_success_response();
    } else {
        send_error_response(StatusCode::NOT_FOUND, "Position not found");
    }
}

fn handle_search_tokens(query: &str) {
    let results = search_coingecko_tokens(query);
    send_json_response(StatusCode::OK, results);
}

fn handle_get_token_price(token_id: &str) {
    let prices = fetch_prices_from_coingecko(&[token_id.to_string()]);
    if let Some(price_data) = prices.get(token_id) {
        send_json_response(StatusCode::OK, serde_json::json!({
            "price_usd": price_data.price_usd,
            "price_change_24h": price_data.price_change_24h
        }));
    } else {
        send_error_response(StatusCode::NOT_FOUND, "Price not found");
    }
}

fn handle_get_top_tokens() {
    let tokens = fetch_top_tokens_from_coingecko();
    send_json_response(StatusCode::OK, serde_json::json!({ "tokens": tokens }));
}

fn handle_refresh_prices(state: &mut AppState) {
    let now = get_current_timestamp();

    // Check cache TTL - skip refresh if prices are still fresh
    if state.last_price_fetch > 0 && now - state.last_price_fetch < COINGECKO_CACHE_TTL_SECS {
        send_json_response(StatusCode::OK, serde_json::json!({
            "message": "Prices still fresh",
            "cached": true,
            "next_refresh_in": COINGECKO_CACHE_TTL_SECS - (now - state.last_price_fetch)
        }));
        return;
    }

    // Collect all CoinGecko IDs from positions
    let coingecko_ids: Vec<String> = state.positions.values()
        .filter_map(|p| p.token_identifier.clone())
        .collect();

    if !coingecko_ids.is_empty() {
        let prices = fetch_prices_from_coingecko(&coingecko_ids);
        for (id, price_data) in prices {
            // Store under raw CoinGecko ID (backwards compatibility)
            state.prices.insert(id.clone(), price_data.clone());
            // Also store under canonical key for new lookups
            let canonical_key = format!("coingecko:{}", id.to_lowercase());
            state.prices.insert(canonical_key, price_data);
        }
        state.last_price_fetch = now;
    }

    // Create daily snapshot if we don't have one for today
    let today = get_current_date();
    let has_today_snapshot = state.snapshots.iter().any(|s| s.date == today);

    if !has_today_snapshot {
        let positions = calculate_positions_with_derived(state);
        let total_value: f64 = positions.iter().map(|p| p.position_value_usd).sum();

        state.snapshots.push(DailySnapshot {
            date: today,
            total_value_usd: total_value,
            position_count: state.positions.len(),
            timestamp: now,
        });

        // Keep only last 365 snapshots
        if state.snapshots.len() > 365 {
            state.snapshots.remove(0);
        }
    }

    save_state(state);
    send_success_response();
}

fn handle_export_positions_csv(state: &AppState) {
    let mut csv = String::from("id,symbol,name,chain,quantity,entry_price_usd,entry_date,user_note,user_tags,created_at\n");

    for position in state.positions.values() {
        let tags = position.user_tags.join(";");
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            escape_csv_field(&position.id),
            escape_csv_field(&position.token_symbol),
            escape_csv_field(&position.token_name),
            escape_csv_field(&position.chain),
            position.quantity,
            position.entry_price_usd,
            escape_csv_field(&position.entry_date.clone().unwrap_or_default()),
            escape_csv_field(&position.user_note.clone().unwrap_or_default()),
            escape_csv_field(&tags),
            position.created_at
        ));
    }

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/csv; charset=utf-8".to_string());
    headers.insert("Content-Disposition".to_string(), "attachment; filename=positions.csv".to_string());
    http::server::send_response(StatusCode::OK, Some(headers), csv.into_bytes());
}

fn handle_export_snapshots_csv(state: &AppState) {
    let mut csv = String::from("date,total_value_usd,position_count,timestamp\n");

    for snapshot in &state.snapshots {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            snapshot.date,
            snapshot.total_value_usd,
            snapshot.position_count,
            snapshot.timestamp
        ));
    }

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/csv".to_string());
    headers.insert("Content-Disposition".to_string(), "attachment; filename=snapshots.csv".to_string());
    http::server::send_response(StatusCode::OK, Some(headers), csv.into_bytes());
}

fn handle_get_market_data(state: &AppState) {
    // Build position market data from stored prices
    let position_market_data: Vec<PositionMarketData> = state.positions.values()
        .map(|position| {
            let price_data = position.token_identifier.as_ref()
                .and_then(|cg_id| state.prices.get(cg_id));

            PositionMarketData {
                position_id: position.id.clone(),
                symbol: position.token_symbol.clone(),
                price_usd: price_data.map(|p| p.price_usd).unwrap_or_else(|| {
                    if is_stablecoin(&position.token_symbol) { 1.0 }
                    else { position.entry_price_usd.to_f64().unwrap_or(0.0) }
                }),
                price_change_24h: price_data.and_then(|p| p.price_change_24h),
                volume_24h: price_data.and_then(|p| p.volume_24h),
                liquidity_usd: price_data.and_then(|p| p.liquidity_usd),
                market_cap: price_data.and_then(|p| p.market_cap),
                fdv: price_data.and_then(|p| p.fdv),
                price_source: price_data.map(|p| p.price_source.clone()).unwrap_or(PriceSource::Manual),
            }
        })
        .collect();

    // Convert chain TVL to vec
    let chain_tvl: Vec<ChainTvlData> = state.chain_tvl.values().cloned().collect();

    let response = MarketDataResponse {
        chain_tvl,
        position_market_data,
        last_updated: state.last_price_fetch,
    };

    send_json_response(StatusCode::OK, response);
}

fn handle_refresh_chain_tvl(state: &mut AppState) {
    let now = get_current_timestamp();

    // Only fetch if stale (5 minutes cache)
    if now - state.last_tvl_fetch < 300 && !state.chain_tvl.is_empty() {
        send_success_response();
        return;
    }

    let tvl_data = fetch_chain_tvl_from_defillama();
    state.chain_tvl = tvl_data;
    state.last_tvl_fetch = now;
    save_state(state);

    send_success_response();
}

fn handle_search_dex(query: &str) {
    let pairs = search_dexscreener(query);
    let response = DexSearchResponse {
        pairs,
        query: query.to_string(),
    };
    send_json_response(StatusCode::OK, response);
}

fn handle_fetch_dex_price(state: &mut AppState, token_address: &str) {
    if let Some(price_data) = fetch_token_from_dexscreener(token_address) {
        // Store the price data
        state.prices.insert(token_address.to_lowercase(), price_data.clone());
        save_state(state);

        send_json_response(StatusCode::OK, price_data);
    } else {
        send_error_response(StatusCode::NOT_FOUND, "Token not found on DEXes");
    }
}

fn handle_load_demo(state: &mut AppState) {
    // Check if demo already loaded
    if state.positions.values().any(|p| p.user_tags.contains(&"demo".to_string())) {
        return send_error_response(StatusCode::CONFLICT, "Demo portfolio already loaded");
    }

    // Add demo positions
    let demo_positions = get_demo_positions();
    for position in demo_positions {
        if let Some(cg_id) = &position.token_identifier {
            if !state.prices.contains_key(cg_id) {
                let prices = fetch_prices_from_coingecko(&[cg_id.clone()]);
                for (id, price_data) in prices {
                    // Store under raw CoinGecko ID (backwards compatibility)
                    state.prices.insert(id.clone(), price_data.clone());
                    // Also store under canonical key
                    let canonical_key = format!("coingecko:{}", id.to_lowercase());
                    state.prices.insert(canonical_key, price_data);
                }
            }
        }
        state.positions.insert(position.id.clone(), position);
    }

    state.last_price_fetch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    save_state(state);

    send_json_response(StatusCode::OK, serde_json::json!({
        "success": true,
        "message": "Demo portfolio loaded",
        "positions_added": 4
    }));
}

fn handle_clear_demo(state: &mut AppState) {
    let demo_ids: Vec<String> = state.positions.values()
        .filter(|p| p.user_tags.contains(&"demo".to_string()))
        .map(|p| p.id.clone())
        .collect();

    if demo_ids.is_empty() {
        return send_error_response(StatusCode::NOT_FOUND, "No demo positions to clear");
    }

    let count = demo_ids.len();
    for id in demo_ids {
        state.positions.remove(&id);
    }

    save_state(state);

    send_json_response(StatusCode::OK, serde_json::json!({
        "success": true,
        "message": "Demo portfolio cleared",
        "positions_removed": count
    }));
}

// ============================================================================
// HTTP Request Router
// ============================================================================

fn handle_http_request(state: &mut AppState, req: &IncomingHttpRequest, body: &[u8]) {
    let raw_path = req.path().unwrap_or_else(|_| "/".to_string());
    let method = req.method().unwrap_or(Method::GET);

    // Parse path and query string from raw path
    let (path, query_string) = match raw_path.split_once('?') {
        Some((p, q)) => (p.to_string(), Some(q.to_string())),
        None => (raw_path, None),
    };

    let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    match (method, path_parts.as_slice()) {
        // Holdings API
        (Method::GET, ["api", "holdings"]) => handle_get_holdings(state),
        (Method::POST, ["api", "positions"]) => handle_add_position(state, body),
        (Method::PUT, ["api", "positions", id]) => handle_update_position(state, id, body),
        (Method::DELETE, ["api", "positions", id]) => handle_delete_position(state, id),

        // Exposure API
        (Method::GET, ["api", "exposure"]) => handle_get_exposure(state),

        // Insights API
        (Method::GET, ["api", "insights"]) => handle_get_insights(state),

        // Pricing API
        (Method::POST, ["api", "refresh"]) => handle_refresh_prices(state),
        (Method::GET, ["api", "tokens", "search"]) => {
            // Parse query from query string
            let query = query_string
                .as_ref()
                .and_then(|qs| {
                    qs.split('&')
                        .find(|p| p.starts_with("q="))
                        .map(|p| p.trim_start_matches("q="))
                })
                .unwrap_or("");
            handle_search_tokens(query);
        }
        (Method::GET, ["api", "tokens", "price", token_id]) => handle_get_token_price(token_id),
        (Method::GET, ["api", "tokens", "top"]) => handle_get_top_tokens(),

        // Market Data API (DeFi Llama + Dexscreener)
        (Method::GET, ["api", "market"]) => handle_get_market_data(state),
        (Method::POST, ["api", "market", "tvl"]) => handle_refresh_chain_tvl(state),
        (Method::GET, ["api", "dex", "search"]) => {
            let query = query_string
                .as_ref()
                .and_then(|qs| {
                    qs.split('&')
                        .find(|p| p.starts_with("q="))
                        .map(|p| p.trim_start_matches("q="))
                })
                .unwrap_or("");
            handle_search_dex(query);
        }
        (Method::GET, ["api", "dex", "token", address]) => handle_fetch_dex_price(state, address),

        // Export API
        (Method::GET, ["api", "export", "positions"]) => handle_export_positions_csv(state),
        (Method::GET, ["api", "export", "snapshots"]) => handle_export_snapshots_csv(state),

        // Demo Portfolio API
        (Method::POST, ["api", "demo", "load"]) => handle_load_demo(state),
        (Method::DELETE, ["api", "demo", "clear"]) => handle_clear_demo(state),

        // Fallback
        (_, ["api", ..]) => send_error_response(StatusCode::NOT_FOUND, "API endpoint not found"),
        _ => send_error_response(StatusCode::NOT_FOUND, "Not found"),
    }
}

// ============================================================================
// Main Loop
// ============================================================================

// EXTENSION POINT: Future wallet import integration
// fn import_from_wallet(wallet_address: &str, chain: &str) -> Vec<Position> { ... }

// EXTENSION POINT: Future advanced exposure mapping
// fn get_protocol_exposure(position: &Position) -> Vec<ProtocolExposure> { ... }

// EXTENSION POINT: Future AI narrative summaries
// fn generate_portfolio_narrative(summary: &PortfolioSummary) -> String { ... }

// EXTENSION POINT: Future historical exposure snapshots
// fn store_exposure_snapshot(exposure: &ExposureResponse) { ... }

call_init!(init);
fn init(_our: Address) {
    println!("smart-portfolio: starting...");

    let mut state = load_state();

    // Migrate existing positions to canonical IDs (one-time migration)
    migrate_positions_to_canonical(&mut state);

    let mut server = HttpServer::new(5);

    // Serve UI
    if let Err(e) = server.serve_ui("ui", vec!["/"], HttpBindingConfig::default()) {
        println!("smart-portfolio: failed to serve UI: {:?}", e);
    }

    // Bind API routes
    let api_config = HttpBindingConfig::default();
    let api_paths = [
        "/api/holdings",
        "/api/positions",
        "/api/positions/:id",
        "/api/exposure",
        "/api/insights",
        "/api/refresh",
        "/api/tokens/search",
        "/api/tokens/price/:token_id",
        "/api/tokens/top",
        "/api/export/positions",
        "/api/export/snapshots",
        // Market data APIs (DeFi Llama + Dexscreener)
        "/api/market",
        "/api/market/tvl",
        "/api/dex/search",
        "/api/dex/token/:address",
        // Demo portfolio APIs
        "/api/demo/load",
        "/api/demo/clear",
    ];

    for path in api_paths {
        if let Err(e) = server.bind_http_path(path, api_config.clone()) {
            println!("smart-portfolio: failed to bind {}: {:?}", path, e);
        }
    }

    println!("smart-portfolio: server ready");

    loop {
        match await_message() {
            Ok(message) => {
                if message.is_request() {
                    match serde_json::from_slice::<HttpServerRequest>(message.body()) {
                        Ok(HttpServerRequest::Http(http_request)) => {
                            let body = message.blob().map(|b| b.bytes.clone()).unwrap_or_default();
                            handle_http_request(&mut state, &http_request, &body);
                        }
                        Ok(_) => {
                            // WebSocket or other request type, ignore for now
                        }
                        Err(e) => {
                            println!("smart-portfolio: failed to parse request: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("smart-portfolio: error receiving message: {:?}", e);
            }
        }
    }
}
