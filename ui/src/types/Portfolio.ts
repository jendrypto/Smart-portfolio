// Position types
export interface Position {
  id: string;
  token_identifier: string | null;
  token_symbol: string;
  token_name: string;
  chain: string;
  quantity: string;
  entry_price_usd: string;
  entry_date: string | null;
  user_note: string | null;
  user_tags: string[];
  created_at: number;
  updated_at: number;
}

export interface PositionWithDerived {
  position: Position;
  current_price_usd: number;
  price_change_24h: number | null;
  position_value_usd: number;
  unrealized_pnl_usd: number;
  unrealized_pnl_percent: number;
  allocation_percent: number;
}

// Summary types
export interface TopPosition {
  symbol: string;
  name: string;
  allocation_percent: number;
}

export interface MoverInfo {
  symbol: string;
  name: string;
  change_24h_percent: number;
  weighted_impact: number;
}

export interface ChainExposure {
  chain: string;
  value_usd: number;
  percentage: number;
}

export interface PortfolioSummary {
  total_value_usd: number;
  total_unrealized_pnl_usd: number;
  total_unrealized_pnl_percent: number;
  top_positions: TopPosition[];
  largest_position_percent: number;
  chain_summary: ChainExposure[];
  biggest_24h_mover: MoverInfo | null;
  last_price_update: number | null;
}

// Snapshot types
export interface DailySnapshot {
  date: string;
  total_value_usd: number;
  position_count: number;
  timestamp: number;
}

// Holdings response
export interface HoldingsResponse {
  positions: PositionWithDerived[];
  summary: PortfolioSummary;
  snapshots: DailySnapshot[];
}

// Exposure types
export type ConfidenceLevel = 'High' | 'Medium' | 'Low';

export interface ExposureEntry {
  category: string;
  value_usd: number;
  percentage: number;
  confidence: ConfidenceLevel;
  notes: string;
}

export interface ExposureResponse {
  underlying_exposure: ExposureEntry[];
  chain_exposure: ChainExposure[];
}

// Token search
export interface TokenSearchResult {
  id: string;
  symbol: string;
  name: string;
}

// Add position request
export interface AddPositionRequest {
  token_identifier?: string;
  token_symbol: string;
  token_name: string;
  chain: string;
  quantity: string;
  entry_price_usd: string;
  entry_date?: string;
  user_note?: string;
  user_tags?: string[];
}

// Sort options
export type SortField = 'value' | 'pnl' | 'allocation';
export type SortDirection = 'asc' | 'desc';

// Insight types
export type InsightType =
  | 'Concentration'
  | 'Performance'
  | 'Diversification'
  | 'Rebalancing'
  | 'PriceMovement'
  | 'RiskAlert'
  | 'Opportunity'
  | 'General';

export type InsightPriority = 'High' | 'Medium' | 'Low';

export interface Insight {
  id: string;
  insight_type: InsightType;
  priority: InsightPriority;
  title: string;
  description: string;
  icon: string;
  color: string;
  action_label: string | null;
  action_url: string | null;
  metadata: Record<string, string>;
}

export interface InsightsResponse {
  insights: Insight[];
  generated_at: number;
  portfolio_health_score: number;
}

// ============================================================================
// Market Data Types (DeFi Llama + Dexscreener)
// ============================================================================

export type PriceSource = 'CoinGecko' | 'DefiLlama' | 'Dexscreener' | 'Manual';

export interface ChainTvlData {
  chain: string;
  tvl: number;
  tvl_change_24h: number | null;
  protocols_count: number | null;
  timestamp: number;
}

export interface PositionMarketData {
  position_id: string;
  symbol: string;
  price_usd: number;
  price_change_24h: number | null;
  volume_24h: number | null;
  liquidity_usd: number | null;
  market_cap: number | null;
  fdv: number | null;
  price_source: PriceSource;
}

export interface MarketDataResponse {
  chain_tvl: ChainTvlData[];
  position_market_data: PositionMarketData[];
  last_updated: number;
}

export interface DexPairData {
  pair_address: string;
  dex_name: string;
  chain: string;
  base_token_symbol: string;
  base_token_name: string;
  quote_token_symbol: string;
  price_usd: number;
  price_change_24h: number | null;
  volume_24h: number;
  liquidity_usd: number;
  fdv: number | null;
  pair_created_at: number | null;
}

export interface DexSearchResponse {
  pairs: DexPairData[];
  query: string;
}

export interface TopToken {
  id: string;
  symbol: string;
  name: string;
  current_price: number;
  price_change_percentage_24h: number;
  market_cap_rank: number;
}
