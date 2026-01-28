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
use chrono::DateTime;
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
    #[serde(default)]
    pub source: Option<String>, // None = manual, Some("wallet") = wallet-imported
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
    AltL1s,
    ProtocolTokens,
    Other,
}

impl ExposureCategory {
    fn as_str(&self) -> &'static str {
        match self {
            ExposureCategory::ETH => "ETH",
            ExposureCategory::BTC => "BTC",
            ExposureCategory::Stablecoins => "Stablecoins",
            ExposureCategory::AltL1s => "Alt L1s",
            ExposureCategory::ProtocolTokens => "Protocol Tokens",
            ExposureCategory::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceBreakdown {
    pub level: ConfidenceLevel,
    pub value_usd: f64,
    pub percentage: f64,  // percentage within this category
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureEntry {
    pub category: String,
    pub value_usd: f64,
    pub percentage: f64,
    pub confidence: ConfidenceLevel,  // Keep for backwards compat (lowest)
    pub confidence_breakdown: Vec<ConfidenceBreakdown>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExposure {
    pub chain: String,
    pub value_usd: f64,
    pub percentage: f64,
    pub chain_type: String,  // "L1" or "L2"
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
// Domain Types - Risk Metrics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: u64,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub assets: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityScore {
    pub asset: String,
    pub symbol: String,
    pub volatility_7d: f64,
    pub volatility_30d: f64,
    pub volatility_rank: String, // "low", "medium", "high", "extreme"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownData {
    pub asset: String,
    pub symbol: String,
    pub current_drawdown: f64,
    pub max_drawdown_30d: f64,
    pub peak_price: f64,
    pub trough_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRiskMetrics {
    pub correlation_matrix: CorrelationMatrix,
    pub volatility_scores: Vec<VolatilityScore>,
    pub drawdowns: Vec<DrawdownData>,
    pub portfolio_volatility: f64,
    pub risk_score: u8,
    pub generated_at: u64,
}

// ============================================================================
// Domain Types - Actionable Recommendations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: String,
    pub label: String,
    pub from_asset: Option<String>,
    pub to_asset: Option<String>,
    pub percentage: Option<f64>,
    pub estimated_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionableRecommendation {
    pub id: String,
    pub priority: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub action: Option<RecommendedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationsResponse {
    pub recommendations: Vec<ActionableRecommendation>,
    pub risk_score: u8,
    pub generated_at: u64,
}

// ============================================================================
// Domain Types - Stress Testing Scenarios
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetImpact {
    pub symbol: String,
    pub current_value: f64,
    pub scenario_value: f64,
    pub percent_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub portfolio_impact: f64,
    pub affected_assets: Vec<AssetImpact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenariosResponse {
    pub scenarios: Vec<Scenario>,
    pub current_value: f64,
}

// ============================================================================
// Domain Types - News (CryptoPanic)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub url: String,
    pub source: String,
    pub published_at: String,
    pub positive_votes: i32,
    pub negative_votes: i32,
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
#[serde(default)]
struct AppState {
    positions: HashMap<String, Position>,
    prices: HashMap<String, PriceData>, // keyed by coingecko id or contract address
    snapshots: Vec<DailySnapshot>,
    last_price_fetch: u64,
    // Enhanced data from additional APIs
    chain_tvl: HashMap<String, ChainTvlData>, // keyed by chain name
    dex_pairs: HashMap<String, Vec<DexPairData>>, // keyed by token symbol/address
    last_tvl_fetch: u64,
    // Configuration (set via POST /api/config)
    api_key: Option<String>, // CoinGecko API key
    cryptopanic_api_key: Option<String>, // CryptoPanic API key
    // Risk metrics cache (avoids 30 HTTP requests per load)
    cached_risk_metrics: Option<PortfolioRiskMetrics>,
    risk_metrics_cached_at: u64,
    // Wallet import
    wallet_address: Option<String>,
    wallet_chains: Vec<String>,
    moralis_api_key: Option<String>,
}

/// Get the configured CoinGecko API key, or empty string if not set
fn get_api_key(state: &AppState) -> String {
    state.api_key.clone().unwrap_or_default()
}

// ============================================================================
// Demo Portfolio Generator
// ============================================================================

fn get_demo_positions() -> Vec<Position> {
    let now = get_current_timestamp();

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
            source: None,
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
            source: None,
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
            source: None,
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
            source: None,
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
    let timestamp = get_current_timestamp() as i64;
    DateTime::from_timestamp(timestamp, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

// ============================================================================
// CoinGecko Integration
// ============================================================================

fn get_exposure_category(symbol: &str, coingecko_id: Option<&str>, token_name: &str) -> (ExposureCategory, ConfidenceLevel, String) {
    let symbol_upper = symbol.to_uppercase();
    let cg_id = coingecko_id.unwrap_or("");
    let name_lower = token_name.to_lowercase();

    // Rule 1: ETH - High Confidence
    if symbol_upper == "ETH" || cg_id == "ethereum" || name_lower == "ethereum" {
        return (ExposureCategory::ETH, ConfidenceLevel::High, "Direct ETH holding".to_string());
    }

    // Rule 2: ETH - Medium Confidence (Wrapped/Derivative)
    let eth_derivatives = ["WETH", "STETH", "RETH", "CBETH", "WSTETH", "FRXETH", "SWETH"];
    if eth_derivatives.contains(&symbol_upper.as_str()) ||
       name_lower.contains("wrapped eth") ||
       name_lower.contains("staked eth") ||
       name_lower.contains("liquid staked eth") {
        return (ExposureCategory::ETH, ConfidenceLevel::Medium, "ETH derivative or wrapper".to_string());
    }

    // Rule 3: BTC - High Confidence
    if symbol_upper == "BTC" || cg_id == "bitcoin" || name_lower == "bitcoin" {
        return (ExposureCategory::BTC, ConfidenceLevel::High, "Direct BTC holding".to_string());
    }

    // Rule 4: BTC - Medium Confidence (Wrapped)
    let btc_wrappers = ["WBTC", "TBTC", "CBBTC", "RENBTC", "HBTC"];
    if btc_wrappers.contains(&symbol_upper.as_str()) ||
       name_lower.contains("wrapped bitcoin") {
        return (ExposureCategory::BTC, ConfidenceLevel::Medium, "BTC wrapper".to_string());
    }

    // Rule 5: Stablecoins - High Confidence
    let major_stables = ["USDC", "USDT", "DAI", "BUSD", "TUSD"];
    if major_stables.contains(&symbol_upper.as_str()) {
        return (ExposureCategory::Stablecoins, ConfidenceLevel::High, "Major stablecoin".to_string());
    }

    // Rule 6: Stablecoins - Medium Confidence
    let other_stables = ["FRAX", "LUSD", "GUSD", "USDP", "PYUSD", "SUSD", "MIM", "CRVUSD", "GHO"];
    if other_stables.contains(&symbol_upper.as_str()) ||
       (name_lower.contains("usd") && name_lower.contains("stable")) {
        return (ExposureCategory::Stablecoins, ConfidenceLevel::Medium, "Non-major stablecoin".to_string());
    }

    // Rule 7: Alt L1s - High Confidence
    let alt_l1s_high = [
        ("SOL", "solana", "Solana"),
        ("AVAX", "avalanche-2", "Avalanche"),
        ("MATIC", "matic-network", "Polygon"),
        ("DOT", "polkadot", "Polkadot"),
        ("ATOM", "cosmos", "Cosmos"),
        ("NEAR", "near", "NEAR Protocol"),
        ("ADA", "cardano", "Cardano"),
        ("FTM", "fantom", "Fantom"),
        ("ALGO", "algorand", "Algorand"),
        ("XLM", "stellar", "Stellar"),
        ("ICP", "internet-computer", "Internet Computer"),
        ("APT", "aptos", "Aptos"),
        ("SUI", "sui", "Sui"),
        ("SEI", "sei-network", "Sei"),
        ("INJ", "injective-protocol", "Injective"),
    ];
    for (sym, cgid, name) in alt_l1s_high.iter() {
        if symbol_upper == *sym || cg_id == *cgid {
            return (ExposureCategory::AltL1s, ConfidenceLevel::High, format!("{} L1 token", name));
        }
    }

    // Rule 8: Protocol Tokens - High Confidence
    let protocol_tokens = [
        ("UNI", "uniswap", "Uniswap"),
        ("AAVE", "aave", "Aave"),
        ("MKR", "maker", "Maker"),
        ("CRV", "curve-dao-token", "Curve"),
        ("COMP", "compound-governance-token", "Compound"),
        ("SNX", "havven", "Synthetix"),
        ("LDO", "lido-dao", "Lido"),
        ("RPL", "rocket-pool", "Rocket Pool"),
        ("GMX", "gmx", "GMX"),
        ("DYDX", "dydx", "dYdX"),
        ("LINK", "chainlink", "Chainlink"),
        ("GRT", "the-graph", "The Graph"),
        ("ENS", "ethereum-name-service", "ENS"),
        ("OP", "optimism", "Optimism"),
        ("ARB", "arbitrum", "Arbitrum"),
        ("PENDLE", "pendle", "Pendle"),
        ("ENA", "ethena", "Ethena"),
        ("EIGEN", "eigenlayer", "EigenLayer"),
    ];
    for (sym, cgid, name) in protocol_tokens.iter() {
        if symbol_upper == *sym || cg_id == *cgid {
            return (ExposureCategory::ProtocolTokens, ConfidenceLevel::High, format!("{} protocol token", name));
        }
    }

    // Rule 9: Other - Low Confidence
    (ExposureCategory::Other, ConfidenceLevel::Low, "Unclassified asset".to_string())
}

fn get_chain_type(chain: &str) -> &'static str {
    let chain_lower = chain.to_lowercase();
    match chain_lower.as_str() {
        "ethereum" | "bitcoin" | "solana" | "avalanche" | "bnb chain" => "L1",
        "arbitrum" | "optimism" | "base" | "polygon" | "zksync" | "linea" | "scroll" => "L2",
        _ => "L1"  // Default to L1 for unknown chains
    }
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

/// Search for tokens on CoinGecko by query string.
///
/// Uses the CoinGecko search API with the configured demo API key.
/// Returns up to 10 matching tokens with their id, symbol, and name.
///
/// Note: Timeout is in milliseconds (30000 = 30 seconds).
fn search_coingecko_tokens(query: &str, api_key: &str) -> Vec<TokenSearchResult> {
    let url = format!(
        "https://api.coingecko.com/api/v3/search?query={}&x_cg_demo_api_key={}",
        url_encode(query), api_key
    );

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            // Note: timeout is in milliseconds, not seconds
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30000,  // 30 seconds in milliseconds
                vec![],
            ) {
                Ok(response) => {
                    let status = response.status();
                    let body = response.body();

                    if status.is_success() {
                        match serde_json::from_slice::<CoinGeckoSearchResponse>(body) {
                            Ok(search_response) => {
                                search_response.coins.into_iter()
                                    .take(10)
                                    .map(|c| TokenSearchResult {
                                        id: c.id,
                                        symbol: c.symbol,
                                        name: c.name,
                                    })
                                    .collect()
                            }
                            Err(e) => {
                                println!("smart-portfolio: CoinGecko search parse error: {:?}", e);
                                vec![]
                            }
                        }
                    } else {
                        println!("smart-portfolio: CoinGecko search failed with status: {:?}", status);
                        vec![]
                    }
                }
                Err(e) => {
                    println!("smart-portfolio: CoinGecko search request failed: {:?}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            println!("smart-portfolio: CoinGecko search URL parse error: {:?}", e);
            vec![]
        }
    }
}

/// Fetch the top 25 tokens by market cap from CoinGecko.
///
/// Uses the CoinGecko markets API with the configured demo API key.
/// Returns tokens with current price, 24h change, and market cap rank.
///
/// Note: Timeout is in milliseconds (30000 = 30 seconds).
fn fetch_top_tokens_from_coingecko(api_key: &str) -> Vec<TopToken> {
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&order=market_cap_desc&per_page=25&page=1&sparkline=false&x_cg_demo_api_key={}",
        api_key
    );

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            // Note: timeout is in milliseconds, not seconds
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30000,  // 30 seconds in milliseconds
                vec![],
            ) {
                Ok(response) => {
                    let status = response.status();
                    let body = response.body();

                    if status.is_success() {
                        match serde_json::from_slice::<Vec<CoinGeckoMarketCoin>>(body) {
                            Ok(coins) => {
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
                            }
                            Err(e) => {
                                println!("smart-portfolio: top tokens parse error: {:?}", e);
                                vec![]
                            }
                        }
                    } else {
                        println!("smart-portfolio: top tokens request failed with status: {:?}", status);
                        vec![]
                    }
                }
                Err(e) => {
                    println!("smart-portfolio: top tokens request failed: {:?}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            println!("smart-portfolio: top tokens URL parse error: {:?}", e);
            vec![]
        }
    }
}

fn fetch_prices_from_coingecko(ids: &[String], api_key: &str) -> HashMap<String, PriceData> {
    if ids.is_empty() {
        return HashMap::new();
    }

    let ids_str = ids.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true&include_market_cap=true&x_cg_demo_api_key={}",
        ids_str, api_key
    );

    let now = get_current_timestamp();

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30000,
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
                30000,
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
                30000,
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
// Historical Price Fetching (DeFi Llama)
// ============================================================================

/// Fetch historical price for multiple tokens at a specific timestamp
fn fetch_historical_prices_at_timestamp(coins: &[String], timestamp: u64) -> HashMap<String, f64> {
    if coins.is_empty() {
        return HashMap::new();
    }

    // Format coins for DeFi Llama: coingecko:bitcoin,coingecko:ethereum
    let coin_ids: String = coins.iter()
        .map(|c| format!("coingecko:{}", c))
        .collect::<Vec<_>>()
        .join(",");

    let url = format!("{}/prices/historical/{}/{}", DEFILLAMA_COINS_API, timestamp, coin_ids);

    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30000,
                vec![],
            ) {
                Ok(response) => {
                    if response.status().is_success() {
                        let price_response: DefiLlamaPriceResponse =
                            serde_json::from_slice(response.body()).unwrap_or(DefiLlamaPriceResponse { coins: HashMap::new() });
                        price_response.coins.into_iter()
                            .filter_map(|(key, p)| {
                                p.price.map(|price| {
                                    // Extract symbol from key (e.g., "coingecko:bitcoin" -> "bitcoin")
                                    let symbol = key.split(':').last().unwrap_or(&key).to_string();
                                    (symbol, price)
                                })
                            })
                            .collect()
                    } else {
                        HashMap::new()
                    }
                }
                Err(_) => HashMap::new(),
            }
        }
        Err(_) => HashMap::new(),
    }
}

/// Fetch 30 days of historical prices for multiple tokens
fn fetch_historical_prices_range(coins: &[String], days: u32) -> HashMap<String, Vec<PricePoint>> {
    let now = get_current_timestamp();
    let mut results: HashMap<String, Vec<PricePoint>> = HashMap::new();

    // Fetch daily prices for the range
    for day in 0..days {
        let timestamp = now - (day as u64 * 86400);
        let prices = fetch_historical_prices_at_timestamp(coins, timestamp);

        for (coin, price) in prices {
            results.entry(coin).or_default().push(PricePoint { timestamp, price });
        }
    }

    // Sort each asset's prices by timestamp (oldest to newest)
    for prices in results.values_mut() {
        prices.sort_by_key(|p| p.timestamp);
    }

    results
}

// ============================================================================
// Risk Metrics Calculation Functions
// ============================================================================

/// Calculate daily returns from price points
fn calculate_returns(prices: &[PricePoint]) -> Vec<f64> {
    prices.windows(2)
        .map(|w| (w[1].price - w[0].price) / w[0].price)
        .collect()
}

/// Calculate Pearson correlation coefficient between two return series
fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 {
        return 0.0;
    }

    let mean_x: f64 = x.iter().take(n).sum::<f64>() / n as f64;
    let mean_y: f64 = y.iter().take(n).sum::<f64>() / n as f64;

    let (mut numerator, mut denom_x, mut denom_y) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        numerator += dx * dy;
        denom_x += dx * dx;
        denom_y += dy * dy;
    }

    let denominator = (denom_x * denom_y).sqrt();
    if denominator == 0.0 { 0.0 } else { numerator / denominator }
}

/// Calculate annualized volatility from returns
fn calculate_volatility(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let n = returns.len() as f64;
    let mean: f64 = returns.iter().sum::<f64>() / n;
    let variance: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    variance.sqrt() * (365.0_f64).sqrt() // Annualized
}

/// Get volatility rank based on annualized volatility
fn get_volatility_rank(volatility: f64) -> String {
    if volatility < 0.3 {
        "low".to_string()
    } else if volatility < 0.6 {
        "medium".to_string()
    } else if volatility < 1.0 {
        "high".to_string()
    } else {
        "extreme".to_string()
    }
}

/// Calculate current and max drawdown from price points
fn calculate_drawdown_metrics(prices: &[PricePoint]) -> (f64, f64, f64, f64) {
    if prices.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let mut peak = prices[0].price;
    let mut max_drawdown = 0.0;
    let mut trough_price = prices[0].price;
    let peak_price = prices.iter().map(|p| p.price).fold(0.0_f64, f64::max);

    for p in prices {
        if p.price > peak {
            peak = p.price;
        }
        let drawdown = (peak - p.price) / peak;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
            trough_price = p.price;
        }
    }

    let current_drawdown = (peak_price - prices.last().unwrap().price) / peak_price;

    (current_drawdown * 100.0, max_drawdown * 100.0, peak_price, trough_price)
}

/// Calculate comprehensive risk metrics for the portfolio
fn calculate_risk_metrics(positions: &[PositionWithDerived]) -> PortfolioRiskMetrics {
    let now = get_current_timestamp();

    // Get unique CoinGecko IDs from positions
    let coins: Vec<String> = positions.iter()
        .filter_map(|p| p.position.token_identifier.clone())
        .collect();

    if coins.is_empty() {
        return PortfolioRiskMetrics {
            correlation_matrix: CorrelationMatrix { assets: vec![], matrix: vec![] },
            volatility_scores: vec![],
            drawdowns: vec![],
            portfolio_volatility: 0.0,
            risk_score: 50,
            generated_at: now,
        };
    }

    // Fetch 30 days of historical prices
    let historical = fetch_historical_prices_range(&coins, 30);
    let assets: Vec<String> = historical.keys().cloned().collect();
    let n = assets.len();

    // Calculate correlation matrix
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                matrix[i][j] = 1.0;
            } else if let (Some(prices_i), Some(prices_j)) =
                (historical.get(&assets[i]), historical.get(&assets[j])) {
                let returns_i = calculate_returns(prices_i);
                let returns_j = calculate_returns(prices_j);
                matrix[i][j] = pearson_correlation(&returns_i, &returns_j);
            }
        }
    }

    // Calculate volatility scores and drawdowns
    let mut volatility_scores = vec![];
    let mut drawdowns = vec![];

    for (asset, prices) in &historical {
        let returns = calculate_returns(prices);
        let vol_30d = calculate_volatility(&returns);
        let vol_7d = if returns.len() >= 7 {
            calculate_volatility(&returns[returns.len().saturating_sub(7)..])
        } else {
            vol_30d
        };

        // Find symbol from positions
        let symbol = positions.iter()
            .find(|p| p.position.token_identifier.as_deref() == Some(asset))
            .map(|p| p.position.token_symbol.clone())
            .unwrap_or_default();

        volatility_scores.push(VolatilityScore {
            asset: asset.clone(),
            symbol: symbol.clone(),
            volatility_7d: vol_7d * 100.0,
            volatility_30d: vol_30d * 100.0,
            volatility_rank: get_volatility_rank(vol_30d),
        });

        let (current_dd, max_dd, peak, trough) = calculate_drawdown_metrics(prices);
        drawdowns.push(DrawdownData {
            asset: asset.clone(),
            symbol,
            current_drawdown: current_dd,
            max_drawdown_30d: max_dd,
            peak_price: peak,
            trough_price: trough,
        });
    }

    // Calculate overall risk score
    let avg_volatility: f64 = volatility_scores.iter()
        .map(|v| v.volatility_30d)
        .sum::<f64>() / volatility_scores.len().max(1) as f64;
    let max_drawdown: f64 = drawdowns.iter()
        .map(|d| d.max_drawdown_30d)
        .fold(0.0, f64::max);
    let risk_score = ((avg_volatility * 0.5 + max_drawdown * 0.5) as u8).min(100);

    PortfolioRiskMetrics {
        correlation_matrix: CorrelationMatrix { assets, matrix },
        volatility_scores,
        drawdowns,
        portfolio_volatility: avg_volatility,
        risk_score,
        generated_at: now,
    }
}

// ============================================================================
// Actionable Recommendations Engine
// ============================================================================

/// Generate actionable recommendations based on portfolio analysis
fn generate_actionable_recommendations(
    positions: &[PositionWithDerived],
    summary: &PortfolioSummary,
    risk_metrics: &PortfolioRiskMetrics,
) -> Vec<ActionableRecommendation> {
    let mut recommendations: Vec<ActionableRecommendation> = Vec::new();

    // 1. CONCENTRATION RISK
    if summary.largest_position_percent > 50.0 {
        if let Some(top) = summary.top_positions.first() {
            let excess = summary.largest_position_percent - 30.0;
            recommendations.push(ActionableRecommendation {
                id: "reduce_concentration".to_string(),
                priority: "critical".to_string(),
                category: "risk".to_string(),
                title: format!("Reduce {} Concentration", top.symbol),
                description: format!(
                    "{} is {:.1}% of your portfolio - above the recommended 30% maximum for any single asset.",
                    top.symbol, summary.largest_position_percent
                ),
                impact: format!("Reduces single-asset risk by {:.0}%", excess),
                action: Some(RecommendedAction {
                    action_type: "rebalance".to_string(),
                    label: format!("Rebalance {:.0}% to diversify", excess),
                    from_asset: Some(top.symbol.clone()),
                    to_asset: None,
                    percentage: Some(excess),
                    estimated_value: Some(summary.total_value_usd * (excess / 100.0)),
                }),
            });
        }
    } else if summary.largest_position_percent > 35.0 {
        if let Some(top) = summary.top_positions.first() {
            recommendations.push(ActionableRecommendation {
                id: "monitor_concentration".to_string(),
                priority: "medium".to_string(),
                category: "alert".to_string(),
                title: format!("Monitor {} Position", top.symbol),
                description: format!(
                    "{} at {:.1}% is approaching concentration threshold. Consider rebalancing soon.",
                    top.symbol, summary.largest_position_percent
                ),
                impact: "Awareness of concentration risk".to_string(),
                action: None,
            });
        }
    }

    // 2. HIGH VOLATILITY POSITIONS
    for vol in &risk_metrics.volatility_scores {
        if vol.volatility_rank == "extreme" {
            if let Some(pos) = positions.iter().find(|p|
                p.position.token_identifier.as_deref() == Some(&vol.asset)
            ) {
                if pos.allocation_percent > 10.0 {
                    recommendations.push(ActionableRecommendation {
                        id: format!("high_vol_{}", vol.asset),
                        priority: "high".to_string(),
                        category: "risk".to_string(),
                        title: format!("{} Extreme Volatility", vol.symbol),
                        description: format!(
                            "{} has {:.0}% annualized volatility with {:.1}% allocation. Consider reducing exposure.",
                            vol.symbol, vol.volatility_30d, pos.allocation_percent
                        ),
                        impact: "Reduces portfolio volatility".to_string(),
                        action: Some(RecommendedAction {
                            action_type: "sell".to_string(),
                            label: "Reduce by 50%".to_string(),
                            from_asset: Some(vol.symbol.clone()),
                            to_asset: Some("USDC".to_string()),
                            percentage: Some(50.0),
                            estimated_value: Some(pos.position_value_usd * 0.5),
                        }),
                    });
                }
            }
        }
    }

    // 3. HIGH CORRELATION WARNING
    let m = &risk_metrics.correlation_matrix;
    for i in 0..m.assets.len() {
        for j in (i + 1)..m.assets.len() {
            if m.matrix[i][j] > 0.85 {
                // Find symbols for display
                let sym_i = positions.iter()
                    .find(|p| p.position.token_identifier.as_deref() == Some(&m.assets[i]))
                    .map(|p| p.position.token_symbol.clone())
                    .unwrap_or_else(|| m.assets[i].clone());
                let sym_j = positions.iter()
                    .find(|p| p.position.token_identifier.as_deref() == Some(&m.assets[j]))
                    .map(|p| p.position.token_symbol.clone())
                    .unwrap_or_else(|| m.assets[j].clone());

                recommendations.push(ActionableRecommendation {
                    id: format!("correlation_{}_{}", i, j),
                    priority: "medium".to_string(),
                    category: "rebalance".to_string(),
                    title: "High Correlation Detected".to_string(),
                    description: format!(
                        "{} and {} have {:.0}% correlation. Holding both provides limited diversification benefit.",
                        sym_i, sym_j, m.matrix[i][j] * 100.0
                    ),
                    impact: "Improve true diversification".to_string(),
                    action: None,
                });
            }
        }
    }

    // 4. DRAWDOWN RECOVERY OPPORTUNITIES
    for dd in &risk_metrics.drawdowns {
        if dd.current_drawdown > 25.0 {
            recommendations.push(ActionableRecommendation {
                id: format!("drawdown_{}", dd.asset),
                priority: "low".to_string(),
                category: "opportunity".to_string(),
                title: format!("{} Recovery Opportunity", dd.symbol),
                description: format!(
                    "{} is {:.0}% below its 30-day high. If fundamentals are intact, consider DCA.",
                    dd.symbol, dd.current_drawdown
                ),
                impact: "May improve cost basis".to_string(),
                action: Some(RecommendedAction {
                    action_type: "buy".to_string(),
                    label: "Consider DCA".to_string(),
                    from_asset: Some("USDC".to_string()),
                    to_asset: Some(dd.symbol.clone()),
                    percentage: None,
                    estimated_value: None,
                }),
            });
        }
    }

    // 5. NO STABLECOINS WARNING
    let has_stables = positions.iter().any(|p| {
        let sym = p.position.token_symbol.to_uppercase();
        ["USDC", "USDT", "DAI", "FRAX", "LUSD"].contains(&sym.as_str())
    });
    if !has_stables && summary.total_value_usd > 1000.0 {
        recommendations.push(ActionableRecommendation {
            id: "no_stables".to_string(),
            priority: "low".to_string(),
            category: "rebalance".to_string(),
            title: "No Stablecoin Reserve".to_string(),
            description: "Consider holding 5-10% in stablecoins for buying opportunities during market dips.".to_string(),
            impact: "Enables opportunistic buying".to_string(),
            action: None,
        });
    }

    // Sort by priority
    recommendations.sort_by(|a, b| {
        let priority_order = |p: &str| match p {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            "low" => 3,
            _ => 4,
        };
        priority_order(&a.priority).cmp(&priority_order(&b.priority))
    });

    recommendations
}

// ============================================================================
// Stress Testing Scenarios
// ============================================================================

/// Calculate portfolio impact under various market scenarios
fn calculate_scenarios(positions: &[PositionWithDerived]) -> ScenariosResponse {
    let total_value: f64 = positions.iter().map(|p| p.position_value_usd).sum();

    // Define predefined scenarios
    let scenario_definitions = vec![
        (
            "crypto_winter",
            "Crypto Winter",
            "Major market crash scenario",
            vec![
                ("ETH", -70.0), ("BTC", -60.0), ("Alt L1s", -80.0),
                ("Protocol Tokens", -85.0), ("Stablecoins", 0.0), ("Other", -75.0)
            ]
        ),
        (
            "eth_rally",
            "ETH Rally",
            "ETH outperforms BTC",
            vec![
                ("ETH", 50.0), ("BTC", -10.0), ("Protocol Tokens", 30.0),
                ("Alt L1s", 0.0), ("Stablecoins", 0.0), ("Other", 10.0)
            ]
        ),
        (
            "stablecoin_depeg",
            "Stablecoin Risk",
            "Major stablecoin depegs",
            vec![
                ("BTC", 5.0), ("ETH", 5.0), ("Stablecoins", -15.0),
                ("Alt L1s", -5.0), ("Protocol Tokens", -10.0), ("Other", -10.0)
            ]
        ),
        (
            "bull_run",
            "Bull Market",
            "Major crypto rally",
            vec![
                ("BTC", 100.0), ("ETH", 150.0), ("Alt L1s", 200.0),
                ("Protocol Tokens", 180.0), ("Stablecoins", 0.0), ("Other", 120.0)
            ]
        ),
    ];

    let scenarios: Vec<Scenario> = scenario_definitions.iter().map(|(id, name, desc, changes)| {
        let mut affected_assets = Vec::new();
        let mut new_total = 0.0;

        for pos in positions {
            let (category, _, _) = get_exposure_category(
                &pos.position.token_symbol,
                pos.position.token_identifier.as_deref(),
                &pos.position.token_name
            );

            let change = changes.iter()
                .find(|(cat, _)| *cat == category.as_str())
                .map(|(_, v)| *v)
                .unwrap_or(0.0);

            let scenario_value = pos.position_value_usd * (1.0 + change / 100.0);
            new_total += scenario_value;

            affected_assets.push(AssetImpact {
                symbol: pos.position.token_symbol.clone(),
                current_value: pos.position_value_usd,
                scenario_value,
                percent_change: change,
            });
        }

        let portfolio_impact = if total_value > 0.0 {
            ((new_total - total_value) / total_value) * 100.0
        } else {
            0.0
        };

        Scenario {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            portfolio_impact,
            affected_assets,
        }
    }).collect();

    ScenariosResponse {
        scenarios,
        current_value: total_value,
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
                30000,
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
                30000,
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

/// Look up 24h price change using the same resolution order as get_position_price()
fn get_position_price_change_24h(position: &Position, state: &AppState) -> Option<f64> {
    // Try canonical ID first (preferred for new positions)
    if let Some(ref canonical) = position.canonical_id {
        if let Some(price_data) = state.prices.get(&canonical.canonical) {
            return price_data.price_change_24h;
        }
        if let Some(ref cg_id) = canonical.coingecko_id {
            if let Some(price_data) = state.prices.get(cg_id) {
                return price_data.price_change_24h;
            }
        }
    }

    // Legacy fallback: check old token_identifier field
    if let Some(ref cg_id) = position.token_identifier {
        if let Some(price_data) = state.prices.get(cg_id) {
            return price_data.price_change_24h;
        }
    }

    None
}

fn calculate_positions_with_derived(state: &AppState) -> Vec<PositionWithDerived> {
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

        let price_change_24h = get_position_price_change_24h(position, state);

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
            chain_type: get_chain_type(&chain).to_string(),
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

    // Track values by category and confidence level
    let mut category_data: HashMap<ExposureCategory, HashMap<ConfidenceLevel, (f64, Vec<String>)>> = HashMap::new();

    for p in positions {
        let (category, confidence, note) = get_exposure_category(
            &p.position.token_symbol,
            p.position.token_identifier.as_deref(),
            &p.position.token_name
        );

        let cat_entry = category_data.entry(category).or_insert_with(HashMap::new);
        let conf_entry = cat_entry.entry(confidence).or_insert((0.0, vec![]));
        conf_entry.0 += p.position_value_usd;
        if !note.is_empty() && !conf_entry.1.contains(&note) {
            conf_entry.1.push(note);
        }
    }

    // Build exposure entries with confidence breakdown
    let underlying_exposure: Vec<ExposureEntry> = category_data.into_iter()
        .map(|(category, confidence_map)| {
            let total_cat_value: f64 = confidence_map.values().map(|(v, _)| v).sum();
            let all_notes: Vec<String> = confidence_map.values()
                .flat_map(|(_, notes)| notes.clone())
                .collect();

            // Build confidence breakdown
            let mut breakdown: Vec<ConfidenceBreakdown> = confidence_map.iter()
                .map(|(level, (value, _))| ConfidenceBreakdown {
                    level: level.clone(),
                    value_usd: *value,
                    percentage: if total_cat_value > 0.0 { (*value / total_cat_value) * 100.0 } else { 0.0 },
                })
                .collect();
            breakdown.sort_by(|a, b| b.value_usd.partial_cmp(&a.value_usd).unwrap_or(std::cmp::Ordering::Equal));

            // Overall confidence is the lowest present
            let overall_confidence = if confidence_map.contains_key(&ConfidenceLevel::Low) {
                ConfidenceLevel::Low
            } else if confidence_map.contains_key(&ConfidenceLevel::Medium) {
                ConfidenceLevel::Medium
            } else {
                ConfidenceLevel::High
            };

            ExposureEntry {
                category: category.as_str().to_string(),
                value_usd: total_cat_value,
                percentage: if total_value > 0.0 { (total_cat_value / total_value) * 100.0 } else { 0.0 },
                confidence: overall_confidence,
                confidence_breakdown: breakdown,
                notes: all_notes.join("; "),
            }
        })
        .collect();

    // Chain exposure with L1/L2 type
    let mut chain_values: HashMap<String, f64> = HashMap::new();
    for p in positions {
        *chain_values.entry(p.position.chain.clone()).or_insert(0.0) += p.position_value_usd;
    }
    let chain_exposure: Vec<ChainExposure> = chain_values.into_iter()
        .map(|(chain, value)| ChainExposure {
            chain_type: get_chain_type(&chain).to_string(),
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

// ============================================================================
// Risk Analysis API Handlers
// ============================================================================

const RISK_CACHE_TTL_SECS: u64 = 60;

fn get_or_compute_risk_metrics(state: &mut AppState) -> PortfolioRiskMetrics {
    let now = get_current_timestamp();
    if let Some(ref cached) = state.cached_risk_metrics {
        if now.saturating_sub(state.risk_metrics_cached_at) < RISK_CACHE_TTL_SECS {
            return cached.clone();
        }
    }
    let positions = calculate_positions_with_derived(state);
    let risk_metrics = calculate_risk_metrics(&positions);
    state.cached_risk_metrics = Some(risk_metrics.clone());
    state.risk_metrics_cached_at = now;
    save_state(state);
    risk_metrics
}

fn handle_get_risk_metrics(state: &mut AppState) {
    let risk_metrics = get_or_compute_risk_metrics(state);
    send_json_response(StatusCode::OK, risk_metrics);
}

fn handle_get_recommendations(state: &mut AppState) {
    let positions = calculate_positions_with_derived(state);
    let summary = calculate_portfolio_summary(&positions, state);
    let risk_metrics = get_or_compute_risk_metrics(state);
    let recommendations = generate_actionable_recommendations(&positions, &summary, &risk_metrics);

    let response = RecommendationsResponse {
        recommendations,
        risk_score: risk_metrics.risk_score,
        generated_at: get_current_timestamp(),
    };
    send_json_response(StatusCode::OK, response);
}

fn handle_get_scenarios(state: &AppState) {
    let positions = calculate_positions_with_derived(state);
    let scenarios = calculate_scenarios(&positions);
    send_json_response(StatusCode::OK, scenarios);
}

fn fetch_cryptopanic_news(currencies: &str, api_key: &str) -> Vec<NewsItem> {
    if currencies.is_empty() || api_key.is_empty() {
        return Vec::new();
    }

    let encoded_currencies = url_encode(currencies);
    let url = format!(
        "https://cryptopanic.com/api/developer/v2/posts/?auth_token={}&currencies={}&kind=news&public=true",
        url_encode(api_key),
        encoded_currencies
    );

    let parsed_url = match url::Url::parse(&url) {
        Ok(u) => u,
        Err(e) => {
            println!("smart-portfolio: invalid CryptoPanic URL: {:?}", e);
            return Vec::new();
        }
    };

    match http::client::send_request_await_response(
        Method::GET,
        parsed_url,
        None,
        30000,
        vec![],
    ) {
        Ok(response) => {
            let status = response.status();
            if status.as_u16() != 200 {
                println!("smart-portfolio: CryptoPanic API returned {}", status);
                return Vec::new();
            }
            let body = response.body();
            match serde_json::from_slice::<serde_json::Value>(body) {
                Ok(json) => {
                    let mut items = Vec::new();
                    if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
                        for result in results.iter().take(5) {
                            let title = result.get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let url = result.get("url")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let source = result.get("source")
                                .and_then(|v| v.get("title"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            let published_at = result.get("published_at")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let votes = result.get("votes").unwrap_or(&serde_json::Value::Null);
                            let positive_votes = votes.get("positive")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0) as i32;
                            let negative_votes = votes.get("negative")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0) as i32;

                            items.push(NewsItem {
                                title,
                                url,
                                source,
                                published_at,
                                positive_votes,
                                negative_votes,
                            });
                        }
                    }
                    items
                }
                Err(e) => {
                    println!("smart-portfolio: CryptoPanic parse error: {:?}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            println!("smart-portfolio: CryptoPanic fetch error: {:?}", e);
            Vec::new()
        }
    }
}

const CRYPTOPANIC_FALLBACK_KEY: &str = "eaa4e721ce93028b6b5086ad7456fb0b7f48a970";

fn handle_get_news(state: &AppState, currencies: &str) {
    let api_key = state.cryptopanic_api_key.clone()
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| CRYPTOPANIC_FALLBACK_KEY.to_string());

    println!("smart-portfolio: fetching news for currencies={}, key_len={}", currencies, api_key.len());
    let news = fetch_cryptopanic_news(currencies, &api_key);
    println!("smart-portfolio: fetched {} news items", news.len());
    send_json_response(StatusCode::OK, serde_json::json!({ "news": news }));
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
        source: None,
    };

    // Fetch price for new position if CoinGecko ID provided
    if let Some(cg_id) = &position.token_identifier {
        if !state.prices.contains_key(cg_id) {
            let api_key = get_api_key(state);
            let prices = fetch_prices_from_coingecko(&[cg_id.clone()], &api_key);
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

fn handle_search_tokens(state: &AppState, query: &str) {
    let api_key = get_api_key(state);
    let results = search_coingecko_tokens(query, &api_key);
    send_json_response(StatusCode::OK, results);
}

fn handle_get_token_price(state: &AppState, token_id: &str) {
    let api_key = get_api_key(state);
    let prices = fetch_prices_from_coingecko(&[token_id.to_string()], &api_key);
    if let Some(price_data) = prices.get(token_id) {
        send_json_response(StatusCode::OK, serde_json::json!({
            "price_usd": price_data.price_usd,
            "price_change_24h": price_data.price_change_24h
        }));
    } else {
        send_error_response(StatusCode::NOT_FOUND, "Price not found");
    }
}

fn handle_get_top_tokens(state: &AppState) {
    let api_key = get_api_key(state);
    let tokens = fetch_top_tokens_from_coingecko(&api_key);
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
        let api_key = get_api_key(state);
        let prices = fetch_prices_from_coingecko(&coingecko_ids, &api_key);
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
    let api_key = get_api_key(state);
    let demo_positions = get_demo_positions();
    for position in demo_positions {
        if let Some(cg_id) = &position.token_identifier {
            if !state.prices.contains_key(cg_id) {
                let prices = fetch_prices_from_coingecko(&[cg_id.clone()], &api_key);
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

    state.last_price_fetch = get_current_timestamp();
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

/// Debug endpoint to test outbound HTTP connectivity.
///
/// Verifies that the Hyperware http-client capability is working correctly.
/// Requires `http-client:distro:sys` capability in manifest.json.
///
/// Usage: GET /api/debug/http
fn handle_debug_http() {
    let url = "https://httpbin.org/get";

    match url::Url::parse(url) {
        Ok(parsed_url) => {
            // Note: timeout is in milliseconds (30000 = 30 seconds)
            match http::client::send_request_await_response(
                Method::GET,
                parsed_url,
                None,
                30000,  // 30 seconds in milliseconds
                vec![],
            ) {
                Ok(response) => {
                    let status = response.status();
                    let body_len = response.body().len();
                    send_json_response(StatusCode::OK, serde_json::json!({
                        "success": true,
                        "status": status.as_u16(),
                        "body_len": body_len,
                        "message": "Outbound HTTP works!"
                    }));
                }
                Err(e) => {
                    send_json_response(StatusCode::OK, serde_json::json!({
                        "success": false,
                        "error": format!("{:?}", e),
                        "message": "Outbound HTTP failed - check http-client capability"
                    }));
                }
            }
        }
        Err(e) => {
            send_error_response(StatusCode::BAD_REQUEST, &format!("URL parse error: {:?}", e));
        }
    }
}

// ============================================================================
// Moralis Wallet API Integration
// ============================================================================

fn moralis_chain_id(chain: &str) -> &str {
    match chain {
        "Ethereum" => "eth",
        "Arbitrum" => "arbitrum",
        "Optimism" => "optimism",
        "Base" => "base",
        "Polygon" => "polygon",
        "Avalanche" => "avalanche",
        "BNB Chain" => "bsc",
        _ => "eth",
    }
}

fn chain_native_token(chain: &str) -> (&str, &str, &str) {
    // Returns (symbol, name, coingecko_id)
    match chain {
        "Ethereum" | "Arbitrum" | "Optimism" | "Base" => ("ETH", "Ethereum", "ethereum"),
        "Polygon" => ("MATIC", "Polygon", "matic-network"),
        "Avalanche" => ("AVAX", "Avalanche", "avalanche-2"),
        "BNB Chain" => ("BNB", "BNB", "binancecoin"),
        _ => ("ETH", "Ethereum", "ethereum"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletToken {
    symbol: String,
    name: String,
    chain: String,
    balance: String,
    price_usd: String,
    value_usd: String,
    token_identifier: Option<String>,
    token_address: Option<String>,
    is_native: bool,
}

/// Response item from Moralis /wallets/{address}/tokens endpoint
/// This endpoint returns balances WITH prices, so we don't need separate CoinGecko lookups.
#[derive(Debug, Deserialize)]
struct MoralisWalletToken {
    token_address: Option<String>,
    symbol: Option<String>,
    name: Option<String>,
    balance: Option<String>,
    decimals: Option<u32>,
    usd_price: Option<f64>,
    usd_value: Option<f64>,
    native_token: Option<bool>,
    portfolio_percentage: Option<f64>,
}

/// Paginated response from Moralis /wallets/{address}/tokens
#[derive(Debug, Deserialize)]
struct MoralisWalletTokensResponse {
    result: Vec<MoralisWalletToken>,
}

/// Fetch all token balances (ERC-20 + native) with prices from Moralis in a single call per chain.
/// Uses the /wallets/{address}/tokens endpoint which returns prices directly.
fn fetch_wallet_tokens_with_prices(address: &str, chain: &str, api_key: &str) -> Vec<MoralisWalletToken> {
    let moralis_chain = moralis_chain_id(chain);
    let url_str = format!(
        "https://deep-index.moralis.io/api/v2.2/wallets/{}/tokens?chain={}&exclude_spam=true",
        address, moralis_chain
    );

    let parsed_url = match url::Url::parse(&url_str) {
        Ok(u) => u,
        Err(e) => {
            println!("smart-portfolio: Moralis URL parse error: {:?}", e);
            return vec![];
        }
    };

    let mut headers = HashMap::new();
    headers.insert("X-API-Key".to_string(), api_key.to_string());

    match http::client::send_request_await_response(
        Method::GET,
        parsed_url,
        Some(headers),
        30000,
        vec![],
    ) {
        Ok(response) => {
            if response.status().is_success() {
                // Try paginated format first { result: [...] }
                if let Ok(paginated) = serde_json::from_slice::<MoralisWalletTokensResponse>(response.body()) {
                    return paginated.result;
                }
                // Fall back to plain array format
                serde_json::from_slice(response.body()).unwrap_or_else(|e| {
                    println!("smart-portfolio: Moralis token parse error for {}: {:?}", chain, e);
                    // Log first 500 bytes of response for debugging
                    let body_preview = String::from_utf8_lossy(
                        &response.body()[..response.body().len().min(500)]
                    );
                    println!("smart-portfolio: Response body preview: {}", body_preview);
                    vec![]
                })
            } else {
                let body_preview = String::from_utf8_lossy(
                    &response.body()[..response.body().len().min(500)]
                );
                println!("smart-portfolio: Moralis tokens error for {}: {} - {}", chain, response.status(), body_preview);
                vec![]
            }
        }
        Err(e) => {
            println!("smart-portfolio: Moralis tokens request error for {}: {:?}", chain, e);
            vec![]
        }
    }
}

/// Convert a raw balance string (in smallest unit) to a human-readable decimal
fn format_token_balance(raw_balance: &str, decimals: u32) -> String {
    let raw = match Decimal::from_str(raw_balance) {
        Ok(d) => d,
        Err(_) => return "0".to_string(),
    };

    if decimals == 0 {
        return raw.to_string();
    }

    let divisor = Decimal::from(10u64.pow(decimals.min(18)));
    let result = raw / divisor;

    format!("{}", result.round_dp(8).normalize())
}

const MORALIS_API_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJub25jZSI6IjQxNjczOTQ0LTcyMmQtNDVmNS05MzEwLWRjOTI3ZTYwMGY5MyIsIm9yZ0lkIjoiNDk3MTAzIiwidXNlcklkIjoiNTExNTI1IiwidHlwZUlkIjoiYzM4ZjAzMmEtY2FkNC00MGI1LWExNjEtMzk3OGQwNTg5ZWJhIiwidHlwZSI6IlBST0pFQ1QiLCJpYXQiOjE3Njk1Mzg2ODEsImV4cCI6NDkyNTI5ODY4MX0.Cq3XuRN2KTcrGsFjWOo4RGBBWTRM67p1QRrlykpHBGk";

/// Scan a wallet across multiple chains and return discovered tokens.
/// Uses Moralis /wallets/{address}/tokens which returns prices directly (1 call per chain).
fn scan_wallet(
    address: &str,
    chains: &[String],
    moralis_api_key: &str,
    _coingecko_api_key: &str,
) -> (Vec<WalletToken>, usize) {
    let mut tokens = Vec::new();
    let mut dust_count = 0usize;

    for chain in chains {
        println!("smart-portfolio: scanning {} for wallet {}", chain, address);

        let moralis_tokens = fetch_wallet_tokens_with_prices(address, chain, moralis_api_key);
        println!("smart-portfolio: found {} tokens on {}", moralis_tokens.len(), chain);

        for mt in &moralis_tokens {
            let symbol = mt.symbol.clone().unwrap_or_default();
            let name = mt.name.clone().unwrap_or_default();
            let raw_balance = mt.balance.clone().unwrap_or_default();
            let decimals = mt.decimals.unwrap_or(18);
            let is_native = mt.native_token.unwrap_or(false);

            if raw_balance == "0" || raw_balance.is_empty() {
                continue;
            }

            let balance = format_token_balance(&raw_balance, decimals);

            let price = mt.usd_price.unwrap_or(0.0);
            let value = mt.usd_value.unwrap_or_else(|| {
                let bal: f64 = balance.parse().unwrap_or(0.0);
                bal * price
            });

            // Determine token identifier
            let token_identifier = if is_native {
                let (_, _, cg_id) = chain_native_token(chain);
                Some(cg_id.to_string())
            } else {
                // Use contract address as identifier for now; CoinGecko resolution
                // happens at import time, not scan time
                None
            };

            let token_address = if is_native {
                None
            } else {
                mt.token_address.clone()
            };

            tokens.push(WalletToken {
                symbol: symbol.to_uppercase(),
                name,
                chain: chain.clone(),
                balance,
                price_usd: format!("{}", Decimal::from_f64(price).unwrap_or(Decimal::ZERO).round_dp(6).normalize()),
                value_usd: format!("{}", Decimal::from_f64(value).unwrap_or(Decimal::ZERO).round_dp(2).normalize()),
                token_identifier,
                token_address,
                is_native,
            });
        }
    }

    // Filter dust (< $1)
    let all_tokens = tokens;
    tokens = Vec::new();
    for t in all_tokens {
        let value: f64 = t.value_usd.parse().unwrap_or(0.0);
        if value >= 1.0 {
            tokens.push(t);
        } else if value > 0.0 {
            dust_count += 1;
        }
    }

    // Sort by value descending
    tokens.sort_by(|a, b| {
        let va: f64 = a.value_usd.parse().unwrap_or(0.0);
        let vb: f64 = b.value_usd.parse().unwrap_or(0.0);
        vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
    });

    (tokens, dust_count)
}

// ============================================================================
// Wallet Import Handlers
// ============================================================================

fn handle_wallet_scan(state: &mut AppState, body: &[u8]) {
    #[derive(Deserialize)]
    struct ScanRequest {
        address: String,
        chains: Vec<String>,
    }

    let request: ScanRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => return send_error_response(StatusCode::BAD_REQUEST, "Invalid JSON"),
    };

    // Validate address format
    if !is_eth_address(&request.address) {
        return send_error_response(StatusCode::BAD_REQUEST, "Invalid wallet address. Expected 0x + 40 hex characters.");
    }

    let moralis_key = MORALIS_API_KEY.to_string();
    let coingecko_key = get_api_key(state);

    let valid_chains = ["Ethereum", "Arbitrum", "Optimism", "Base", "Polygon", "Avalanche", "BNB Chain"];
    let chains: Vec<String> = request.chains.iter()
        .filter(|c| valid_chains.contains(&c.as_str()))
        .cloned()
        .collect();

    if chains.is_empty() {
        return send_error_response(StatusCode::BAD_REQUEST, "No valid chains specified");
    }

    // Store wallet info for re-sync
    state.wallet_address = Some(request.address.clone());
    state.wallet_chains = chains.clone();
    save_state(state);

    let (tokens, dust_count) = scan_wallet(&request.address, &chains, &moralis_key, &coingecko_key);

    send_json_response(StatusCode::OK, serde_json::json!({
        "address": request.address,
        "tokens": tokens,
        "dust_filtered": dust_count,
    }));
}

fn handle_wallet_import(state: &mut AppState, body: &[u8]) {
    #[derive(Deserialize)]
    struct ImportRequest {
        tokens: Vec<WalletToken>,
    }

    let request: ImportRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => return send_error_response(StatusCode::BAD_REQUEST, "Invalid JSON"),
    };

    let now = get_current_timestamp();
    let today = {
        let dt = DateTime::from_timestamp(now as i64, 0)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        dt.format("%Y-%m-%d").to_string()
    };

    let mut imported_count = 0;

    for token in &request.tokens {
        let quantity: Decimal = match Decimal::from_str(&token.balance) {
            Ok(q) if q > Decimal::ZERO => q,
            _ => continue,
        };

        let entry_price: Decimal = Decimal::from_str(&token.price_usd).unwrap_or(Decimal::ZERO);

        let id = format!("wallet_{}_{}", token.symbol.to_lowercase(), now + imported_count as u64);

        let canonical_id = if let Some(ref cg_id) = token.token_identifier {
            Some(CanonicalId::from_coingecko(cg_id, cg_id))
        } else if let Some(ref addr) = token.token_address {
            Some(CanonicalId::from_address(addr, &token.chain, addr))
        } else {
            Some(CanonicalId::from_symbol(&token.symbol))
        };

        let position = Position {
            id: id.clone(),
            token_identifier: token.token_identifier.clone(),
            canonical_id,
            token_symbol: token.symbol.to_uppercase(),
            token_name: token.name.clone(),
            chain: token.chain.clone(),
            quantity,
            entry_price_usd: entry_price,
            entry_date: Some(today.clone()),
            user_note: None,
            user_tags: vec![],
            created_at: now,
            updated_at: now,
            source: Some("wallet".to_string()),
        };

        // Store price data
        if let Some(ref cg_id) = token.token_identifier {
            // Native tokens with CoinGecko IDs - fetch from CoinGecko for canonical storage
            if !state.prices.contains_key(cg_id) {
                let api_key = get_api_key(state);
                let prices = fetch_prices_from_coingecko(&[cg_id.clone()], &api_key);
                for (pid, price_data) in prices {
                    state.prices.insert(pid.clone(), price_data.clone());
                    let canonical_key = format!("coingecko:{}", pid.to_lowercase());
                    state.prices.insert(canonical_key, price_data);
                }
            }
        } else if let Some(ref addr) = token.token_address {
            // ERC-20 tokens without CoinGecko ID - store Moralis price under address key
            let price_usd: f64 = token.price_usd.parse().unwrap_or(0.0);
            if price_usd > 0.0 {
                let canonical_key = format!("address:{}:{}", token.chain.to_lowercase(), addr.to_lowercase());
                state.prices.insert(canonical_key.clone(), PriceData {
                    price_usd,
                    price_change_24h: None,
                    market_cap: None,
                    timestamp: now,
                    is_stale: false,
                    volume_24h: None,
                    liquidity_usd: None,
                    price_source: PriceSource::Manual,
                    fdv: None,
                });
                // Also store under raw address for backwards compat
                state.prices.insert(addr.to_lowercase(), PriceData {
                    price_usd,
                    price_change_24h: None,
                    market_cap: None,
                    timestamp: now,
                    is_stale: false,
                    volume_24h: None,
                    liquidity_usd: None,
                    price_source: PriceSource::Manual,
                    fdv: None,
                });
            }
        }

        state.positions.insert(id, position);
        imported_count += 1;
    }

    save_state(state);

    send_json_response(StatusCode::OK, serde_json::json!({
        "success": true,
        "imported": imported_count,
    }));
}

fn handle_wallet_resync(state: &mut AppState) {
    let address = match &state.wallet_address {
        Some(addr) => addr.clone(),
        None => return send_error_response(StatusCode::BAD_REQUEST, "No wallet address configured. Scan a wallet first."),
    };

    let chains = state.wallet_chains.clone();
    if chains.is_empty() {
        return send_error_response(StatusCode::BAD_REQUEST, "No chains configured for wallet scan.");
    }

    let moralis_key = MORALIS_API_KEY.to_string();
    let coingecko_key = get_api_key(state);

    let (tokens, _dust_count) = scan_wallet(&address, &chains, &moralis_key, &coingecko_key);

    let now = get_current_timestamp();
    let today = {
        let dt = DateTime::from_timestamp(now as i64, 0)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        dt.format("%Y-%m-%d").to_string()
    };

    let mut updated = 0usize;
    let mut added = 0usize;
    let mut removed = 0usize;

    // Build a lookup for scanned tokens by canonical key
    let mut scanned_keys: HashMap<String, &WalletToken> = HashMap::new();
    for t in &tokens {
        let key = if let Some(ref cg_id) = t.token_identifier {
            format!("cg:{}", cg_id)
        } else {
            format!("sym:{}:{}", t.symbol.to_uppercase(), t.chain)
        };
        scanned_keys.insert(key, t);
    }

    // Track which scanned tokens matched existing positions
    let mut matched_scan_keys: Vec<String> = Vec::new();

    // Update existing wallet-sourced positions
    let position_ids: Vec<String> = state.positions.keys().cloned().collect();
    for pid in &position_ids {
        let pos = match state.positions.get(pid) {
            Some(p) => p,
            None => continue,
        };

        // Only touch wallet-sourced positions
        if pos.source.as_deref() != Some("wallet") {
            continue;
        }

        // Find matching scanned token
        let pos_key = if let Some(ref cid) = pos.canonical_id {
            if let Some(ref cg_id) = cid.coingecko_id {
                format!("cg:{}", cg_id)
            } else {
                format!("sym:{}:{}", pos.token_symbol, pos.chain)
            }
        } else {
            format!("sym:{}:{}", pos.token_symbol, pos.chain)
        };

        if let Some(scanned_token) = scanned_keys.get(&pos_key) {
            // Update quantity
            if let Ok(new_qty) = Decimal::from_str(&scanned_token.balance) {
                let position = state.positions.get_mut(pid).unwrap();
                position.quantity = new_qty;
                position.updated_at = now;
                updated += 1;
            }
            matched_scan_keys.push(pos_key);
        } else {
            // Token not found in scan → remove (zero balance)
            state.positions.remove(pid);
            removed += 1;
        }
    }

    // Add new tokens not matched to existing positions
    for t in &tokens {
        let key = if let Some(ref cg_id) = t.token_identifier {
            format!("cg:{}", cg_id)
        } else {
            format!("sym:{}:{}", t.symbol.to_uppercase(), t.chain)
        };

        if matched_scan_keys.contains(&key) {
            continue;
        }

        let quantity: Decimal = match Decimal::from_str(&t.balance) {
            Ok(q) if q > Decimal::ZERO => q,
            _ => continue,
        };

        let entry_price = Decimal::from_str(&t.price_usd).unwrap_or(Decimal::ZERO);
        let id = format!("wallet_{}_{}", t.symbol.to_lowercase(), now + added as u64);

        let canonical_id = if let Some(ref cg_id) = t.token_identifier {
            Some(CanonicalId::from_coingecko(cg_id, cg_id))
        } else if let Some(ref addr) = t.token_address {
            Some(CanonicalId::from_address(addr, &t.chain, addr))
        } else {
            Some(CanonicalId::from_symbol(&t.symbol))
        };

        let position = Position {
            id: id.clone(),
            token_identifier: t.token_identifier.clone(),
            canonical_id,
            token_symbol: t.symbol.to_uppercase(),
            token_name: t.name.clone(),
            chain: t.chain.clone(),
            quantity,
            entry_price_usd: entry_price,
            entry_date: Some(today.clone()),
            user_note: None,
            user_tags: vec![],
            created_at: now,
            updated_at: now,
            source: Some("wallet".to_string()),
        };

        state.positions.insert(id, position);
        added += 1;
    }

    save_state(state);

    send_json_response(StatusCode::OK, serde_json::json!({
        "success": true,
        "updated": updated,
        "added": added,
        "removed": removed,
    }));
}

fn handle_wallet_status(state: &AppState) {
    send_json_response(StatusCode::OK, serde_json::json!({
        "address": state.wallet_address,
        "chains": state.wallet_chains,
    }));
}

// ============================================================================
// Configuration Handlers
// ============================================================================

fn handle_set_config(state: &mut AppState, body: &[u8]) {
    #[derive(Deserialize)]
    struct ConfigRequest {
        api_key: Option<String>,
        cryptopanic_api_key: Option<String>,
        moralis_api_key: Option<String>,
    }

    let request: ConfigRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(_) => return send_error_response(StatusCode::BAD_REQUEST, "Invalid JSON"),
    };

    if let Some(key) = request.api_key {
        if key.is_empty() {
            state.api_key = None;
        } else {
            state.api_key = Some(key);
        }
    }

    if let Some(key) = request.cryptopanic_api_key {
        if key.is_empty() {
            state.cryptopanic_api_key = None;
        } else {
            state.cryptopanic_api_key = Some(key);
        }
    }

    if let Some(key) = request.moralis_api_key {
        if key.is_empty() {
            state.moralis_api_key = None;
        } else {
            state.moralis_api_key = Some(key);
        }
    }

    save_state(state);
    send_json_response(StatusCode::OK, serde_json::json!({ "success": true }));
}

fn handle_get_config(state: &AppState) {
    let masked_key = state.api_key.as_ref().map(|k| {
        if k.len() > 8 {
            format!("{}...{}", &k[..4], &k[k.len()-4..])
        } else {
            "****".to_string()
        }
    });

    let masked_cryptopanic_key = state.cryptopanic_api_key.as_ref().map(|k| {
        if k.len() > 8 {
            format!("{}...{}", &k[..4], &k[k.len()-4..])
        } else {
            "****".to_string()
        }
    });

    let masked_moralis_key = state.moralis_api_key.as_ref().map(|k| {
        if k.len() > 8 {
            format!("{}...{}", &k[..4], &k[k.len()-4..])
        } else {
            "****".to_string()
        }
    });

    send_json_response(StatusCode::OK, serde_json::json!({
        "api_key_configured": state.api_key.is_some(),
        "api_key_masked": masked_key,
        "cryptopanic_api_key_configured": state.cryptopanic_api_key.is_some(),
        "cryptopanic_api_key_masked": masked_cryptopanic_key,
        "moralis_api_key_configured": state.moralis_api_key.is_some(),
        "moralis_api_key_masked": masked_moralis_key,
    }));
}

// ============================================================================
// HTTP Request Router
// ============================================================================

/// Main HTTP request handler that routes requests to appropriate handlers.
///
/// Important: Uses Hyperware's `req.query_params()` for accessing query parameters
/// instead of manually parsing the query string. This ensures proper URL decoding
/// and consistent parameter handling.
fn handle_http_request(state: &mut AppState, req: &IncomingHttpRequest, body: &[u8]) {
    let raw_path = req.path().unwrap_or_else(|_| "/".to_string());
    let method = req.method().unwrap_or(Method::GET);

    // Use Hyperware's built-in query parameter parsing.
    // This properly handles URL encoding and provides a HashMap<String, String>.
    let query_params = req.query_params();

    // Extract path without query string for route matching
    let (path, _query_string) = match raw_path.split_once('?') {
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

        // Risk Analysis APIs
        (Method::GET, ["api", "risk", "metrics"]) => handle_get_risk_metrics(state),
        (Method::GET, ["api", "recommendations"]) => handle_get_recommendations(state),
        (Method::GET, ["api", "scenarios"]) => handle_get_scenarios(state),

        // News API (CryptoPanic)
        (Method::GET, ["api", "news"]) => {
            let currencies = query_params.get("currencies").map(|s| s.as_str()).unwrap_or("");
            handle_get_news(state, currencies);
        }

        // Pricing API
        (Method::POST, ["api", "refresh"]) => handle_refresh_prices(state),
        (Method::GET, ["api", "tokens", "search"]) => {
            // Use Hyperware's parsed query params
            let query = query_params.get("q").map(|s| s.as_str()).unwrap_or("");
            handle_search_tokens(state, query);
        }
        (Method::GET, ["api", "tokens", "price", token_id]) => handle_get_token_price(state, token_id),
        (Method::GET, ["api", "tokens", "top"]) => handle_get_top_tokens(state),

        // Market Data API (DeFi Llama + Dexscreener)
        (Method::GET, ["api", "market"]) => handle_get_market_data(state),
        (Method::POST, ["api", "market", "tvl"]) => handle_refresh_chain_tvl(state),
        (Method::GET, ["api", "dex", "search"]) => {
            // Use Hyperware's parsed query params
            let query = query_params.get("q").map(|s| s.as_str()).unwrap_or("");
            handle_search_dex(query);
        }
        (Method::GET, ["api", "dex", "token", address]) => handle_fetch_dex_price(state, address),

        // Export API
        (Method::GET, ["api", "export", "positions"]) => handle_export_positions_csv(state),
        (Method::GET, ["api", "export", "snapshots"]) => handle_export_snapshots_csv(state),

        // Demo Portfolio API
        (Method::POST, ["api", "demo", "load"]) => handle_load_demo(state),
        (Method::DELETE, ["api", "demo", "clear"]) => handle_clear_demo(state),

        // Wallet Import API
        (Method::POST, ["api", "wallet", "scan"]) => handle_wallet_scan(state, body),
        (Method::POST, ["api", "wallet", "import"]) => handle_wallet_import(state, body),
        (Method::POST, ["api", "wallet", "resync"]) => handle_wallet_resync(state),
        (Method::GET, ["api", "wallet", "status"]) => handle_wallet_status(state),

        // Configuration API
        (Method::POST, ["api", "config"]) => handle_set_config(state, body),
        (Method::GET, ["api", "config"]) => handle_get_config(state),

        // Debug HTTP endpoint to test outbound requests
        (Method::GET, ["api", "debug", "http"]) => handle_debug_http(),

        // Fallback
        (_, ["api", ..]) => send_error_response(StatusCode::NOT_FOUND, "API endpoint not found"),
        _ => send_error_response(StatusCode::NOT_FOUND, "Not found"),
    }
}

// ============================================================================
// Main Loop
// ============================================================================

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
    hyperware_process_lib::homepage::add_to_homepage("Smart Portfolio", None, None, None); 
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
        // Risk Analysis APIs
        "/api/risk/metrics",
        "/api/recommendations",
        "/api/scenarios",
        // News API
        "/api/news",
        // Demo portfolio APIs
        "/api/demo/load",
        "/api/demo/clear",
        // Wallet Import API
        "/api/wallet/scan",
        "/api/wallet/import",
        "/api/wallet/resync",
        "/api/wallet/status",
        // Configuration API
        "/api/config",
        // Debug endpoint
        "/api/debug/http",
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
