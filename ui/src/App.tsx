import { useEffect } from 'react';
import usePortfolioStore from './store/portfolio';
import HoldingsTab from './components/HoldingsTab';
import ExposureTab from './components/ExposureTab';
import AddPositionModal from './components/AddPositionModal';
import TickerBanner from './components/TickerBanner';
import './App.css';

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace('/', '');

// Use empty string for same-origin API calls
const API_BASE = '';

function App() {
  const {
    activeTab,
    setActiveTab,
    isLoading,
    error,
    fetchHoldings,
    fetchExposure,
    refreshPrices,
    isAddModalOpen,
    setAddModalOpen,
  } = usePortfolioStore();

  useEffect(() => {
    fetchHoldings();
  }, [fetchHoldings]);

  useEffect(() => {
    if (activeTab === 'exposure') {
      fetchExposure();
    }
  }, [activeTab, fetchExposure]);

  const handleRefresh = async () => {
    await refreshPrices();
    if (activeTab === 'exposure') {
      await fetchExposure();
    }
  };

  const handleExportPositions = () => {
    window.open(`${API_BASE}/api/export/positions`, '_blank');
  };

  const handleExportSnapshots = () => {
    window.open(`${API_BASE}/api/export/snapshots`, '_blank');
  };

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-container">
          <div className="header-brand">
            <div className="header-logo-container">
              <svg width="40" height="40" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
                <rect width="40" height="40" rx="8" fill="url(#logoGradient)" />
                <path d="M12 20L18 14L24 20L18 26L12 20Z" fill="#D9FD65" />
                <path d="M16 20L22 14L28 20L22 26L16 20Z" fill="#D9FD65" fillOpacity="0.5" />
                <defs>
                  <linearGradient id="logoGradient" x1="0" y1="0" x2="40" y2="40">
                    <stop stopColor="#1a1a2e" />
                    <stop offset="1" stopColor="#0f0f1a" />
                  </linearGradient>
                </defs>
              </svg>
            </div>
            <h1 className="header-title">Smart Portfolio</h1>
          </div>
          <div className="header-actions">
            <button
              onClick={handleRefresh}
              disabled={isLoading}
              className="btn btn-secondary hide-mobile"
            >
              <span>&#8635;</span>
              {isLoading ? 'Loading...' : 'Refresh Prices'}
            </button>
            <button
              onClick={() => setAddModalOpen(true)}
              className="btn btn-primary"
            >
              <span>+</span>
              Add Position
            </button>
          </div>
        </div>
      </header>

      <main className="main-content">
        {error && (
          <div className="error-banner">
            {error}
            <button onClick={() => usePortfolioStore.setState({ error: null })}>×</button>
          </div>
        )}

        <div className="tab-nav-container">
          <nav className="tabs">
            <button
              className={`tab ${activeTab === 'holdings' ? 'active' : ''}`}
              onClick={() => setActiveTab('holdings')}
            >
              Holdings
            </button>
            <button
              className={`tab ${activeTab === 'exposure' ? 'active' : ''}`}
              onClick={() => setActiveTab('exposure')}
            >
              Exposure
            </button>
          </nav>
          <div className="tab-actions">
            <button onClick={handleExportPositions} className="btn btn-export">
              Export Positions
            </button>
            <button onClick={handleExportSnapshots} className="btn btn-export-secondary">
              Export Snapshots
            </button>
          </div>
        </div>

        {activeTab === 'holdings' ? <HoldingsTab /> : <ExposureTab />}
      </main>

      {isAddModalOpen && <AddPositionModal />}
      <TickerBanner />
    </div>
  );
}

export default App;
