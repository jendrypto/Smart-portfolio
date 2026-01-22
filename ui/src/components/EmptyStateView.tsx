import usePortfolioStore from '../store/portfolio';

interface FeatureCardProps {
  icon: string;
  color: 'lime' | 'blue';
  title: string;
  description: string;
}

function FeatureCard({ icon, color, title, description }: FeatureCardProps) {
  return (
    <div className="insight-card glass-card">
      <div className={`insight-icon ${color}`}>{icon}</div>
      <div className="insight-content">
        <h4>{title}</h4>
        <p>{description}</p>
      </div>
    </div>
  );
}

function EmptyStateView() {
  const { setAddModalOpen, loadDemoPortfolio, isLoading } = usePortfolioStore();

  const handleLoadDemo = async () => {
    await loadDemoPortfolio();
  };

  return (
    <div className="empty-state-container">
      {/* Hero Section */}
      <div className="empty-state-hero glass-panel">
        <div className="empty-state-hero-content">
          <h1>Welcome to Smart Portfolio</h1>
          <p>
            Track your crypto holdings, analyze exposure, and gain insights across multiple chains.
            All your data stays local and private.
          </p>
          <div className="empty-state-actions">
            <button
              className="btn btn-primary"
              onClick={() => setAddModalOpen(true)}
            >
              <span>+</span> Add Your First Position
            </button>
            <button
              className="btn btn-secondary"
              onClick={handleLoadDemo}
              disabled={isLoading}
            >
              {isLoading ? 'Loading...' : 'Try Demo Portfolio'}
            </button>
          </div>
        </div>
      </div>

      {/* Feature Showcase Cards - reuses insight-cards grid */}
      <div className="insight-cards">
        <FeatureCard
          icon="📊"
          color="lime"
          title="Multi-Chain Tracking"
          description="Track assets across Ethereum, Bitcoin, Solana, and more in one unified view."
        />
        <FeatureCard
          icon="🎯"
          color="blue"
          title="Real Exposure Analysis"
          description="Understand your true ETH, BTC, and stablecoin exposure across wrapped tokens."
        />
        <FeatureCard
          icon="💡"
          color="lime"
          title="Smart Insights"
          description="Get actionable insights on concentration risk, portfolio health, and opportunities."
        />
      </div>

      {/* Preview hint */}
      <div className="empty-state-preview glass-panel">
        <p>
          <span className="preview-icon">💡</span>
          <strong>Tip:</strong> Try the demo portfolio to see all features in action, then clear it to add your own positions.
        </p>
      </div>
    </div>
  );
}

export default EmptyStateView;
