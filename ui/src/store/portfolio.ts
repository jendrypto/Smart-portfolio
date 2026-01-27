import { create } from 'zustand';
import {
  HoldingsResponse,
  ExposureResponse,
  InsightsResponse,
  Insight,
  PositionWithDerived,
  PortfolioSummary,
  DailySnapshot,
  TokenSearchResult,
  AddPositionRequest,
  SortField,
  SortDirection,
  MarketDataResponse,
  DexSearchResponse,
  DexPairData,
  ChainTvlData,
  PositionMarketData,
  TopToken,
  PortfolioRiskMetrics,
  ActionableRecommendation,
  Scenario,
} from '../types/Portfolio';
import { API_BASE } from '../utils/api';

export interface PortfolioStore {
  // Data
  positions: PositionWithDerived[];
  summary: PortfolioSummary | null;
  snapshots: DailySnapshot[];
  exposure: ExposureResponse | null;
  tokenSearchResults: TokenSearchResult[];
  insights: Insight[];
  portfolioHealthScore: number;
  // Market data (DeFi Llama + Dexscreener)
  marketData: MarketDataResponse | null;
  dexSearchResults: DexPairData[];
  chainTvl: ChainTvlData[];
  positionMarketData: PositionMarketData[];
  topTokens: TopToken[];

  // Risk Analysis data
  riskMetrics: PortfolioRiskMetrics | null;
  recommendations: ActionableRecommendation[];
  scenarios: Scenario[];
  riskDataLastFetched: number | null;
  isLoadingRisk: boolean;
  isLoadingRecommendations: boolean;
  isLoadingScenarios: boolean;

  // UI State
  activeTab: 'holdings' | 'exposure';
  isLoading: boolean;
  error: string | null;
  sortField: SortField;
  sortDirection: SortDirection;
  isAddModalOpen: boolean;
  editingPositionId: string | null;

  // Actions
  setActiveTab: (tab: 'holdings' | 'exposure') => void;
  setSorting: (field: SortField, direction: SortDirection) => void;
  setAddModalOpen: (open: boolean) => void;
  setEditingPositionId: (id: string | null) => void;

  // API Actions
  fetchHoldings: () => Promise<void>;
  fetchExposure: () => Promise<void>;
  fetchInsights: () => Promise<void>;
  addPosition: (position: AddPositionRequest) => Promise<boolean>;
  updatePosition: (id: string, updates: Partial<AddPositionRequest>) => Promise<boolean>;
  deletePosition: (id: string) => Promise<boolean>;
  refreshPrices: () => Promise<void>;
  searchTokens: (query: string) => Promise<void>;
  clearTokenSearch: () => void;
  fetchTokenPrice: (tokenId: string) => Promise<number | null>;
  // Market data API actions
  fetchMarketData: () => Promise<void>;
  refreshChainTvl: () => Promise<void>;
  searchDex: (query: string) => Promise<void>;
  clearDexSearch: () => void;
  fetchTopTokens: () => Promise<void>;

  // Risk Analysis API actions
  fetchRiskMetrics: () => Promise<void>;
  fetchRecommendations: () => Promise<void>;
  fetchScenarios: () => Promise<void>;
  fetchAllRiskData: () => Promise<void>;

  // Demo portfolio
  loadDemoPortfolio: () => Promise<boolean>;
  clearDemoPortfolio: () => Promise<boolean>;
}

const usePortfolioStore = create<PortfolioStore>((set, get) => ({
  // Initial state
  positions: [],
  summary: null,
  snapshots: [],
  exposure: null,
  tokenSearchResults: [],
  insights: [],
  portfolioHealthScore: 0,
  // Market data initial state
  marketData: null,
  dexSearchResults: [],
  chainTvl: [],
  positionMarketData: [],
  topTokens: [],

  // Risk Analysis initial state
  riskMetrics: null,
  recommendations: [],
  scenarios: [],
  riskDataLastFetched: null,
  isLoadingRisk: false,
  isLoadingRecommendations: false,
  isLoadingScenarios: false,

  activeTab: 'holdings',
  isLoading: false,
  error: null,
  sortField: 'value',
  sortDirection: 'desc',
  isAddModalOpen: false,
  editingPositionId: null,

  // UI Actions
  setActiveTab: (tab) => set({ activeTab: tab }),
  setSorting: (field, direction) => set({ sortField: field, sortDirection: direction }),
  setAddModalOpen: (open) => set({ isAddModalOpen: open }),
  setEditingPositionId: (id) => set({ editingPositionId: id }),

  // API Actions
  fetchHoldings: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${API_BASE}/api/holdings`);
      if (!response.ok) throw new Error('Failed to fetch holdings');
      const data: HoldingsResponse = await response.json();
      set({
        positions: data.positions,
        summary: data.summary,
        snapshots: data.snapshots,
        isLoading: false,
      });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Unknown error', isLoading: false });
    }
  },

  fetchExposure: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${API_BASE}/api/exposure`);
      if (!response.ok) throw new Error('Failed to fetch exposure');
      const data: ExposureResponse = await response.json();
      set({ exposure: data, isLoading: false });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Unknown error', isLoading: false });
    }
  },

  fetchInsights: async () => {
    try {
      const response = await fetch(`${API_BASE}/api/insights`);
      if (!response.ok) throw new Error('Failed to fetch insights');
      const data: InsightsResponse = await response.json();
      set({
        insights: data.insights,
        portfolioHealthScore: data.portfolio_health_score,
      });
    } catch (e) {
      console.error('Insights fetch error:', e);
      // Don't set error state for insights - they're optional
    }
  },

  addPosition: async (position) => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${API_BASE}/api/positions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(position),
      });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to add position');
      }
      set({ isLoading: false, isAddModalOpen: false });
      await get().fetchHoldings();
      return true;
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Unknown error', isLoading: false });
      return false;
    }
  },

  updatePosition: async (id, updates) => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${API_BASE}/api/positions/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates),
      });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to update position');
      }
      set({ isLoading: false, editingPositionId: null });
      await get().fetchHoldings();
      return true;
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Unknown error', isLoading: false });
      return false;
    }
  },

  deletePosition: async (id) => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${API_BASE}/api/positions/${id}`, {
        method: 'DELETE',
      });
      if (!response.ok) throw new Error('Failed to delete position');
      set({ isLoading: false });
      await get().fetchHoldings();
      return true;
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Unknown error', isLoading: false });
      return false;
    }
  },

  refreshPrices: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${API_BASE}/api/refresh`, { method: 'POST' });
      if (!response.ok) throw new Error('Failed to refresh prices');
      set({ isLoading: false });
      await get().fetchHoldings();
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Unknown error', isLoading: false });
    }
  },

  searchTokens: async (query) => {
    if (!query || query.length < 1) {
      set({ tokenSearchResults: [] });
      return;
    }
    try {
      const response = await fetch(`${API_BASE}/api/tokens/search?q=${encodeURIComponent(query)}`);
      if (!response.ok) throw new Error('Search failed');
      const results: TokenSearchResult[] = await response.json();
      set({ tokenSearchResults: results });
    } catch (e) {
      console.error('Token search error:', e);
      set({ tokenSearchResults: [] });
    }
  },

  clearTokenSearch: () => set({ tokenSearchResults: [] }),

  fetchTokenPrice: async (tokenId: string) => {
    try {
      const response = await fetch(`${API_BASE}/api/tokens/price/${encodeURIComponent(tokenId)}`);
      if (!response.ok) return null;
      const data = await response.json();
      return data.price_usd ?? null;
    } catch (e) {
      console.error('Token price fetch error:', e);
      return null;
    }
  },

  // Market Data API Actions
  fetchMarketData: async () => {
    try {
      const response = await fetch(`${API_BASE}/api/market`);
      if (!response.ok) throw new Error('Failed to fetch market data');
      const data: MarketDataResponse = await response.json();
      set({
        marketData: data,
        chainTvl: data.chain_tvl,
        positionMarketData: data.position_market_data,
      });
    } catch (e) {
      console.error('Market data fetch error:', e);
    }
  },

  refreshChainTvl: async () => {
    try {
      const response = await fetch(`${API_BASE}/api/market/tvl`, { method: 'POST' });
      if (!response.ok) throw new Error('Failed to refresh chain TVL');
      // Fetch updated market data after refresh
      await get().fetchMarketData();
    } catch (e) {
      console.error('Chain TVL refresh error:', e);
    }
  },

  searchDex: async (query) => {
    if (!query || query.length < 2) {
      set({ dexSearchResults: [] });
      return;
    }
    try {
      const response = await fetch(`${API_BASE}/api/dex/search?q=${encodeURIComponent(query)}`);
      if (!response.ok) throw new Error('DEX search failed');
      const data: DexSearchResponse = await response.json();
      set({ dexSearchResults: data.pairs });
    } catch (e) {
      console.error('DEX search error:', e);
      set({ dexSearchResults: [] });
    }
  },

  clearDexSearch: () => set({ dexSearchResults: [] }),

  fetchTopTokens: async () => {
    try {
      const response = await fetch(`${API_BASE}/api/tokens/top`);
      if (!response.ok) throw new Error('Failed to fetch top tokens');
      const data = await response.json();
      set({ topTokens: data.tokens });
    } catch (e) {
      console.error('Top tokens fetch error:', e);
    }
  },

  // Risk Analysis API Actions
  fetchRiskMetrics: async () => {
    try {
      const response = await fetch(`${API_BASE}/api/risk/metrics`);
      if (!response.ok) throw new Error('Failed to fetch risk metrics');
      const data: PortfolioRiskMetrics = await response.json();
      set({ riskMetrics: data });
    } catch (e) {
      console.error('Risk metrics fetch error:', e);
    }
  },

  fetchRecommendations: async () => {
    try {
      const response = await fetch(`${API_BASE}/api/recommendations`);
      if (!response.ok) throw new Error('Failed to fetch recommendations');
      const data = await response.json();
      set({ recommendations: data.recommendations });
    } catch (e) {
      console.error('Recommendations fetch error:', e);
    }
  },

  fetchScenarios: async () => {
    try {
      const response = await fetch(`${API_BASE}/api/scenarios`);
      if (!response.ok) throw new Error('Failed to fetch scenarios');
      const data = await response.json();
      set({ scenarios: data.scenarios });
    } catch (e) {
      console.error('Scenarios fetch error:', e);
    }
  },

  fetchAllRiskData: async () => {
    const { riskDataLastFetched, riskMetrics } = get();
    // Skip if fetched within last 5 minutes and data exists
    if (riskDataLastFetched && riskMetrics && Date.now() - riskDataLastFetched < 5 * 60 * 1000) {
      return;
    }

    set({ isLoadingRisk: true, isLoadingRecommendations: true, isLoadingScenarios: true });

    const results = await Promise.allSettled([
      fetch(`${API_BASE}/api/risk/metrics`).then(async (r) => {
        if (!r.ok) throw new Error('Failed to fetch risk metrics');
        const data: PortfolioRiskMetrics = await r.json();
        set({ riskMetrics: data, isLoadingRisk: false });
      }),
      fetch(`${API_BASE}/api/recommendations`).then(async (r) => {
        if (!r.ok) throw new Error('Failed to fetch recommendations');
        const data = await r.json();
        set({ recommendations: data.recommendations, isLoadingRecommendations: false });
      }),
      fetch(`${API_BASE}/api/scenarios`).then(async (r) => {
        if (!r.ok) throw new Error('Failed to fetch scenarios');
        const data = await r.json();
        set({ scenarios: data.scenarios, isLoadingScenarios: false });
      }),
    ]);

    // Clear loading flags for any that failed
    results.forEach((result, i) => {
      if (result.status === 'rejected') {
        console.error('Risk data fetch error:', result.reason);
        if (i === 0) set({ isLoadingRisk: false });
        if (i === 1) set({ isLoadingRecommendations: false });
        if (i === 2) set({ isLoadingScenarios: false });
      }
    });

    set({ riskDataLastFetched: Date.now() });
  },

  loadDemoPortfolio: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${API_BASE}/api/demo/load`, { method: 'POST' });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to load demo portfolio');
      }
      set({ isLoading: false });
      await get().fetchHoldings();
      return true;
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Unknown error', isLoading: false });
      return false;
    }
  },

  clearDemoPortfolio: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${API_BASE}/api/demo/clear`, { method: 'DELETE' });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to clear demo portfolio');
      }
      set({ isLoading: false });
      await get().fetchHoldings();
      return true;
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Unknown error', isLoading: false });
      return false;
    }
  },
}));

export default usePortfolioStore;
