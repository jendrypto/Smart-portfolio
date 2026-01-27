import { useState, useEffect } from 'react';
import usePortfolioStore from '../store/portfolio';
import { ConfidenceLevel } from '../types/Portfolio';
import ExposureDonutChart from './ExposureDonutChart';
import ExposureSidePanel from './ExposureSidePanel';
import ActionableRecommendations from './ActionableRecommendations';
import CorrelationMatrix from './CorrelationMatrix';
import VolatilityMetrics from './VolatilityMetrics';
import RiskScenarios from './RiskScenarios';
import InfoTooltipBadge from './InfoTooltipBadge';

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
}

function getConfidenceClass(confidence: ConfidenceLevel): string {
  switch (confidence) {
    case 'High':
      return 'confidence-high';
    case 'Medium':
      return 'confidence-medium';
    case 'Low':
      return 'confidence-low';
    default:
      return '';
  }
}

function getConfidenceTooltip(confidence: ConfidenceLevel): string {
  switch (confidence) {
    case 'High':
      return 'Direct holding with clear underlying exposure.';
    case 'Medium':
      return 'Wrapped or derivative asset with known dependency.';
    case 'Low':
      return 'Exposure could not be confidently determined.';
    default:
      return '';
  }
}

function ExposureTab() {
  const {
    exposure,
    isLoading,
    positions,
    riskMetrics,
    recommendations,
    scenarios,
    isLoadingRisk,
    isLoadingRecommendations,
    isLoadingScenarios,
    fetchAllRiskData,
  } = usePortfolioStore();
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);

  // Fetch all risk data in parallel when component mounts or positions change
  useEffect(() => {
    if (positions.length > 0) {
      fetchAllRiskData();
    }
  }, [positions.length, fetchAllRiskData]);

  if (isLoading && !exposure) {
    return <div className="loading">Loading risk analysis...</div>;
  }

  if (positions.length === 0) {
    return (
      <div className="empty-state">
        <h2>No positions yet</h2>
        <p>Add positions to see your risk analysis.</p>
      </div>
    );
  }

  if (!exposure) {
    return (
      <div className="empty-state">
        <h2>Risk data unavailable</h2>
        <p>Try refreshing the page.</p>
      </div>
    );
  }

  const selectedExposure = exposure.underlying_exposure.find(e => e.category === selectedCategory) || null;
  const totalValue = exposure.underlying_exposure.reduce((sum, e) => sum + e.value_usd, 0);

  return (
    <div className="exposure-tab risk-tab">
      {/* Risk Score Header */}
      {isLoadingRisk && !riskMetrics ? (
        <div className="risk-score-banner glass-panel">
          <div className="skeleton-block" style={{ width: '80px', height: '48px', borderRadius: '8px' }} />
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            <div className="skeleton-block" style={{ width: '140px', height: '16px', borderRadius: '4px' }} />
            <div className="skeleton-block" style={{ width: '80px', height: '20px', borderRadius: '4px' }} />
          </div>
        </div>
      ) : riskMetrics ? (
        <div className="risk-score-banner glass-panel">
          <div className="risk-score-main">
            <span className="risk-score-number">{riskMetrics.risk_score}</span>
            <span className="risk-score-max">/100</span>
          </div>
          <div className="risk-score-info">
            <span className="risk-label">Portfolio Risk Score <InfoTooltipBadge text="0-100 score measuring overall portfolio risk. Higher = riskier, based on volatility and drawdowns." /></span>
            <span className={`risk-level ${riskMetrics.risk_score > 70 ? 'high' : riskMetrics.risk_score > 40 ? 'medium' : 'low'}`}>
              {riskMetrics.risk_score > 70 ? 'High Risk' : riskMetrics.risk_score > 40 ? 'Moderate' : 'Low Risk'}
            </span>
          </div>
        </div>
      ) : null}

      {/* Actionable Recommendations */}
      {isLoadingRecommendations && recommendations.length === 0 ? (
        <section className="risk-section glass-panel">
          <h3>Recommendations</h3>
          <div className="skeleton-block" style={{ height: '80px', borderRadius: '12px', marginBottom: '8px' }} />
          <div className="skeleton-block" style={{ height: '80px', borderRadius: '12px' }} />
        </section>
      ) : recommendations.length > 0 ? (
        <section className="risk-section glass-panel">
          <h3>Recommendations <InfoTooltipBadge text="Priority-ranked actions: Critical = act now, High = act soon, Medium = consider." /></h3>
          <p className="section-description">
            Priority-ranked actions to improve your portfolio's risk profile.
          </p>
          <ActionableRecommendations />
        </section>
      ) : null}

      {/* Risk Metrics Grid */}
      <div className="risk-metrics-grid">
        {/* Correlation Matrix */}
        <section className="risk-section glass-panel">
          <h3>Correlation Matrix <InfoTooltipBadge text="Shows how asset prices move together. High positive = less diversification benefit." /></h3>
          {isLoadingRisk && !riskMetrics ? (
            <div className="skeleton-block" style={{ height: '200px', borderRadius: '8px' }} />
          ) : (
            <>
              <p className="section-description">
                Asset correlation over 30 days. High correlation = less diversification.
              </p>
              <CorrelationMatrix />
            </>
          )}
        </section>

        {/* Volatility Metrics */}
        <section className="risk-section glass-panel">
          <h3>Volatility & Drawdown <InfoTooltipBadge text="How much prices swing, annualized. Higher = larger potential gains and losses." /></h3>
          {isLoadingRisk && !riskMetrics ? (
            <div className="skeleton-block" style={{ height: '200px', borderRadius: '8px' }} />
          ) : (
            <>
              <p className="section-description">
                Annualized volatility and maximum drawdown by asset.
              </p>
              <VolatilityMetrics />
            </>
          )}
        </section>
      </div>

      {/* Risk Scenarios */}
      {isLoadingScenarios && scenarios.length === 0 ? (
        <section className="risk-section glass-panel">
          <h3>Stress Test Scenarios <InfoTooltipBadge text="Simulated market events showing portfolio impact under extreme conditions." /></h3>
          <div style={{ display: 'flex', gap: '8px', marginBottom: '12px' }}>
            <div className="skeleton-block" style={{ width: '120px', height: '50px', borderRadius: '8px' }} />
            <div className="skeleton-block" style={{ width: '120px', height: '50px', borderRadius: '8px' }} />
            <div className="skeleton-block" style={{ width: '120px', height: '50px', borderRadius: '8px' }} />
          </div>
          <div className="skeleton-block" style={{ height: '150px', borderRadius: '12px' }} />
        </section>
      ) : scenarios.length > 0 ? (
        <section className="risk-section glass-panel">
          <h3>Stress Test Scenarios <InfoTooltipBadge text="Simulated market events showing portfolio impact under extreme conditions." /></h3>
          <p className="section-description">
            Simulate how your portfolio would perform under different market conditions.
          </p>
          <RiskScenarios />
        </section>
      ) : null}

      {/* Exposure Summary Cards */}
      <div className="exposure-summary-cards">
        <div className="summary-card glass-panel">
          <span className="card-label">Total Portfolio</span>
          <span className="card-value">{formatCurrency(totalValue)}</span>
        </div>
        <div className="summary-card glass-panel">
          <span className="card-label">Categories</span>
          <span className="card-value">{exposure.underlying_exposure.length}</span>
        </div>
        <div className="summary-card glass-panel">
          <span className="card-label">Chains</span>
          <span className="card-value">{exposure.chain_exposure.length}</span>
        </div>
      </div>

      {/* Donut Chart + Underlying Exposure Table */}
      <section className="exposure-section glass-panel">
        <h3>Underlying Asset Exposure</h3>
        <p className="section-description">
          Your portfolio reclassified into core asset categories with confidence indicators.
        </p>

        <div className="exposure-main-content">
          <ExposureDonutChart
            data={exposure.underlying_exposure}
            onCategoryClick={setSelectedCategory}
          />

          <div className="exposure-table-container">
            <table className="exposure-table">
              <thead>
                <tr>
                  <th>Category</th>
                  <th className="text-right">USD Value</th>
                  <th className="text-right">% Portfolio</th>
                  <th>Confidence</th>
                </tr>
              </thead>
              <tbody>
                {exposure.underlying_exposure
                  .sort((a, b) => b.percentage - a.percentage)
                  .map((entry) => (
                    <tr
                      key={entry.category}
                      className="clickable-row"
                      onClick={() => setSelectedCategory(entry.category)}
                    >
                      <td className="category-cell">
                        <span className={`category-badge category-${entry.category.toLowerCase().replace(/ /g, '-')}`}>
                          {entry.category}
                        </span>
                      </td>
                      <td className="text-right">{formatCurrency(entry.value_usd)}</td>
                      <td className="text-right">
                        <div className="percentage-cell">
                          <div
                            className="percentage-bar"
                            style={{ width: `${Math.min(entry.percentage, 100)}%` }}
                          />
                          <span>{entry.percentage.toFixed(1)}%</span>
                        </div>
                      </td>
                      <td>
                        <div className="confidence-cell">
                          {entry.confidence_breakdown && entry.confidence_breakdown.length > 1 ? (
                            <div className="confidence-breakdown-inline">
                              {entry.confidence_breakdown.map((cb) => (
                                <span
                                  key={cb.level}
                                  className={`confidence-mini ${getConfidenceClass(cb.level)}`}
                                  title={getConfidenceTooltip(cb.level)}
                                >
                                  {cb.level.charAt(0)} {cb.percentage.toFixed(0)}%
                                </span>
                              ))}
                            </div>
                          ) : (
                            <span
                              className={`confidence-badge ${getConfidenceClass(entry.confidence)}`}
                              title={getConfidenceTooltip(entry.confidence)}
                            >
                              {entry.confidence}
                            </span>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* Chain Exposure */}
      <section className="exposure-section glass-panel">
        <h3>Chain Exposure</h3>
        <p className="section-description">
          Distribution across blockchains with L1/L2 classification.
        </p>

        <div className="exposure-table-container">
          <table className="exposure-table">
            <thead>
              <tr>
                <th>Chain</th>
                <th>Type</th>
                <th className="text-right">USD Value</th>
                <th className="text-right">% Portfolio</th>
              </tr>
            </thead>
            <tbody>
              {exposure.chain_exposure
                .sort((a, b) => b.percentage - a.percentage)
                .map((chain) => (
                  <tr key={chain.chain}>
                    <td>
                      <span className="chain-badge">{chain.chain}</span>
                    </td>
                    <td>
                      <span className={`chain-type-badge ${(chain.chain_type || 'L1').toLowerCase()}`}>
                        {chain.chain_type || 'L1'}
                      </span>
                    </td>
                    <td className="text-right">{formatCurrency(chain.value_usd)}</td>
                    <td className="text-right">
                      <div className="percentage-cell">
                        <div
                          className="percentage-bar chain-bar"
                          style={{ width: `${Math.min(chain.percentage, 100)}%` }}
                        />
                        <span>{chain.percentage.toFixed(1)}%</span>
                      </div>
                    </td>
                  </tr>
                ))}
            </tbody>
          </table>
        </div>
      </section>

      <div className="exposure-disclaimer">
        <p>
          <strong>Note:</strong> Risk metrics use 30-day historical data from DeFi Llama.
          Correlation and volatility calculations may be limited for newer or low-liquidity assets.
        </p>
      </div>

      {/* Side Panel */}
      <ExposureSidePanel
        category={selectedCategory}
        exposure={selectedExposure}
        positions={positions}
        onClose={() => setSelectedCategory(null)}
      />
    </div>
  );
}

export default ExposureTab;
