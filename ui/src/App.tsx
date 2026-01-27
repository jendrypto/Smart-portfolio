import { useEffect } from 'react';
import usePortfolioStore from './store/portfolio';
import HoldingsTab from './components/HoldingsTab';
import ExposureTab from './components/ExposureTab';
import AddPositionModal from './components/AddPositionModal';
import EditPositionModal from './components/EditPositionModal';
import WalletImportModal from './components/WalletImportModal';
import TickerBanner from './components/TickerBanner';
import InfoFAB from './components/InfoFAB';
import glowLogo from './assets/glow-logo.png';
import { API_BASE } from './utils/api';
import './App.css';

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace('/', '');

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
    editingPositionId,
    isWalletModalOpen,
    setWalletModalOpen,
    fetchWalletStatus,
  } = usePortfolioStore();

  useEffect(() => {
    fetchHoldings();
    fetchWalletStatus();
  }, [fetchHoldings, fetchWalletStatus]);

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
            <img
              src={glowLogo}
              alt="Glow Logo"
              className="header-logo"
            />
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
              onClick={() => setWalletModalOpen(true)}
              className="btn btn-secondary btn-wallet-import"
            >
              Import Wallet
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
              Risk & Analysis
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
      {editingPositionId && <EditPositionModal />}
      {isWalletModalOpen && <WalletImportModal />}
      <InfoFAB isExposureTab={activeTab === 'exposure'} />
      <TickerBanner />
    </div>
  );
}

export default App;
