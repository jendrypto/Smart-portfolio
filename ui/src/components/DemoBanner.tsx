import usePortfolioStore from '../store/portfolio';

function DemoBanner() {
  const { clearDemoPortfolio, isLoading } = usePortfolioStore();

  return (
    <div className="demo-banner">
      <div className="demo-banner-text">
        <span>🎭</span>
        <span>You're viewing demo data. Add your own positions or clear the demo to start fresh.</span>
      </div>
      <button
        className="btn btn-outline"
        onClick={clearDemoPortfolio}
        disabled={isLoading}
      >
        {isLoading ? 'Clearing...' : 'Clear Demo'}
      </button>
    </div>
  );
}

export default DemoBanner;
