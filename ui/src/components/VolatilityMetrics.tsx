import usePortfolioStore from '../store/portfolio';

function getVolatilityClass(rank: string): string {
  switch (rank) {
    case 'low': return 'vol-low';
    case 'medium': return 'vol-medium';
    case 'high': return 'vol-high';
    case 'extreme': return 'vol-extreme';
    default: return '';
  }
}

function VolatilityMetrics() {
  const { riskMetrics } = usePortfolioStore();

  if (!riskMetrics || !riskMetrics.volatility_scores.length) {
    return (
      <div className="volatility-empty">
        <p>No volatility data available.</p>
      </div>
    );
  }

  // Sort by 30d volatility descending
  const sortedScores = [...riskMetrics.volatility_scores]
    .sort((a, b) => b.volatility_30d - a.volatility_30d);

  return (
    <div className="volatility-metrics">
      <div className="volatility-header">
        <span className="vol-label">Portfolio Volatility</span>
        <span className="vol-value">{riskMetrics.portfolio_volatility.toFixed(1)}%</span>
      </div>

      <div className="volatility-list">
        {sortedScores.map((score) => (
          <div key={score.asset} className="volatility-row">
            <div className="vol-asset">
              <span className="vol-symbol">{score.symbol}</span>
              <span className={`vol-rank-badge ${getVolatilityClass(score.volatility_rank)}`}>
                {score.volatility_rank}
              </span>
            </div>
            <div className="vol-bars">
              <div className="vol-bar-container">
                <div className="vol-bar-label">7d</div>
                <div className="vol-bar-track">
                  <div
                    className={`vol-bar-fill ${getVolatilityClass(score.volatility_rank)}`}
                    style={{ width: `${Math.min(score.volatility_7d, 100)}%` }}
                  />
                </div>
                <div className="vol-bar-value">{score.volatility_7d.toFixed(0)}%</div>
              </div>
              <div className="vol-bar-container">
                <div className="vol-bar-label">30d</div>
                <div className="vol-bar-track">
                  <div
                    className={`vol-bar-fill ${getVolatilityClass(score.volatility_rank)}`}
                    style={{ width: `${Math.min(score.volatility_30d, 100)}%` }}
                  />
                </div>
                <div className="vol-bar-value">{score.volatility_30d.toFixed(0)}%</div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Drawdown section */}
      {riskMetrics.drawdowns.length > 0 && (
        <div className="drawdowns-section">
          <h4>Max Drawdowns (30d)</h4>
          <div className="drawdowns-list">
            {riskMetrics.drawdowns
              .filter(d => d.max_drawdown_30d > 5)
              .sort((a, b) => b.max_drawdown_30d - a.max_drawdown_30d)
              .slice(0, 5)
              .map((dd) => (
                <div key={dd.asset} className="drawdown-row">
                  <span className="dd-symbol">{dd.symbol}</span>
                  <span className="dd-current">
                    Current: <span className={dd.current_drawdown > 20 ? 'dd-severe' : ''}>
                      -{dd.current_drawdown.toFixed(1)}%
                    </span>
                  </span>
                  <span className="dd-max">
                    Max: -{dd.max_drawdown_30d.toFixed(1)}%
                  </span>
                </div>
              ))}
          </div>
        </div>
      )}
    </div>
  );
}

export default VolatilityMetrics;
