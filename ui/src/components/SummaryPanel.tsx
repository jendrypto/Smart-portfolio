import { PortfolioSummary, DailySnapshot } from '../types/Portfolio';
import PortfolioChart from './PortfolioChart';

interface Props {
  summary: PortfolioSummary;
  snapshots: DailySnapshot[];
}

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
}

function formatPercent(value: number): string {
  const sign = value >= 0 ? '+' : '';
  return `${sign}${value.toFixed(2)}%`;
}

function getChainColor(chain: string): string {
  const chainLower = chain.toLowerCase();
  if (chainLower.includes('bitcoin') || chainLower === 'btc') return 'bitcoin';
  if (chainLower.includes('ethereum') || chainLower === 'eth') return 'ethereum';
  return 'other';
}

function SummaryPanel({ summary, snapshots }: Props) {
  const pnlIsPositive = summary.total_unrealized_pnl_usd >= 0;

  // Calculate 24h growth if we have snapshots
  const growth24h = snapshots.length >= 2
    ? ((summary.total_value_usd - snapshots[snapshots.length - 2]?.total_value_usd) / snapshots[snapshots.length - 2]?.total_value_usd * 100) || 0
    : summary.total_unrealized_pnl_percent;

  return (
    <>
      <div className="dashboard-grid">
        {/* Main Portfolio Value Card with Chart */}
        <div className="portfolio-value-card glass-panel">
          <div className="gradient-bg" />
          <div className="glow-circle" />

          <div className="portfolio-header">
            <div>
              <p className="portfolio-value-label">Total Portfolio Value</p>
              <div className="portfolio-value glow-lime">
                {formatCurrency(summary.total_value_usd)}
              </div>
            </div>
            <div className="portfolio-growth">
              <p className="portfolio-growth-label">24h Growth</p>
              <div className={`portfolio-growth-value ${growth24h < 0 ? 'negative' : ''}`}>
                {growth24h >= 0 ? '↑' : '↓'} {formatPercent(growth24h)}
              </div>
            </div>
          </div>

          <div className="chart-container">
            <PortfolioChart snapshots={snapshots} />
          </div>
        </div>

        {/* Stats Cards */}
        <div className="stats-cards">
          {/* Unrealized P&L Card */}
          <div className="stat-card glass-card">
            <div className="glow" />
            <h3 className="stat-label">Unrealized P&L</h3>
            <div className="stat-value-row">
              <span className={`stat-value ${pnlIsPositive ? 'positive' : 'negative'}`}>
                {formatCurrency(summary.total_unrealized_pnl_usd)}
              </span>
              <span className={`stat-subvalue ${pnlIsPositive ? '' : 'negative'}`}>
                {formatPercent(summary.total_unrealized_pnl_percent)}
              </span>
            </div>
          </div>

          {/* Concentration Card */}
          <div className="stat-card glass-card">
            <h3 className="stat-label">Concentration</h3>
            <div>
              <span className="stat-value" style={{ color: 'white' }}>
                {summary.largest_position_percent.toFixed(1)}%
              </span>
              <p className="stat-sublabel">
                Largest position: {summary.top_positions[0]?.symbol || 'N/A'}
              </p>
            </div>
            <div className="concentration-bar">
              <div
                className="concentration-bar-fill"
                style={{ width: `${Math.min(summary.largest_position_percent, 100)}%` }}
              />
            </div>
          </div>

          {/* Chain Mix Card */}
          <div className="stat-card glass-card">
            <h3 className="stat-label">Chain Mix</h3>
            <div className="chain-mix-list">
              {summary.chain_summary.slice(0, 4).map((chain, i) => (
                <div key={i} className="chain-mix-item">
                  <div className="chain-mix-left">
                    <span className={`chain-mix-dot ${getChainColor(chain.chain)}`} />
                    <span className="chain-mix-name">{chain.chain}</span>
                  </div>
                  <span className="chain-mix-percent">{chain.percentage.toFixed(1)}%</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

export default SummaryPanel;
