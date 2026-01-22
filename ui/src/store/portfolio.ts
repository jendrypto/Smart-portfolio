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
} from '../types/Portfolio';

// Use empty string for same-origin API calls - works regardless of deployment path
const API_BASE = '';

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
