import { useState } from 'react';
import usePortfolioStore from '../store/portfolio';

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(value);
}

function RiskScenarios() {
  const { scenarios } = usePortfolioStore();
  const [selectedScenarioId, setSelectedScenarioId] = useState<string | null>(null);

  if (!scenarios.length) {
    return (
      <div className="scenarios-empty">
        <p>No scenarios available.</p>
      </div>
    );
  }

  const selectedScenario = scenarios.find(s => s.id === selectedScenarioId);

  return (
    <div className="risk-scenarios">
      <div className="scenario-tabs">
        {scenarios.map((scenario) => (
          <button
            key={scenario.id}
            className={`scenario-tab ${selectedScenarioId === scenario.id ? 'active' : ''}`}
            onClick={() => setSelectedScenarioId(
              selectedScenarioId === scenario.id ? null : scenario.id
            )}
          >
            <span className="scenario-name">{scenario.name}</span>
            <span className={`scenario-impact ${scenario.portfolio_impact >= 0 ? 'positive' : 'negative'}`}>
              {scenario.portfolio_impact >= 0 ? '+' : ''}{scenario.portfolio_impact.toFixed(0)}%
            </span>
          </button>
        ))}
      </div>

      {selectedScenario && (
        <div className="scenario-details">
          <div className="scenario-header">
            <h4>{selectedScenario.name}</h4>
            <p className="scenario-description">{selectedScenario.description}</p>
          </div>

          <div className="scenario-summary">
            <div className="scenario-stat">
              <span className="stat-label">Current Value</span>
              <span className="stat-value">
                {formatCurrency(selectedScenario.affected_assets.reduce((sum, a) => sum + a.current_value, 0))}
              </span>
            </div>
            <div className="scenario-stat">
              <span className="stat-label">Scenario Value</span>
              <span className={`stat-value ${selectedScenario.portfolio_impact >= 0 ? 'positive' : 'negative'}`}>
                {formatCurrency(selectedScenario.affected_assets.reduce((sum, a) => sum + a.scenario_value, 0))}
              </span>
            </div>
            <div className="scenario-stat">
              <span className="stat-label">Impact</span>
              <span className={`stat-value ${selectedScenario.portfolio_impact >= 0 ? 'positive' : 'negative'}`}>
                {selectedScenario.portfolio_impact >= 0 ? '+' : ''}{selectedScenario.portfolio_impact.toFixed(1)}%
              </span>
            </div>
          </div>

          <div className="scenario-assets">
            <h5>Asset Impact</h5>
            <div className="assets-list">
              {selectedScenario.affected_assets
                .filter(a => a.percent_change !== 0)
                .sort((a, b) => Math.abs(b.percent_change) - Math.abs(a.percent_change))
                .map((asset) => (
                  <div key={asset.symbol} className="asset-impact-row">
                    <span className="asset-symbol">{asset.symbol}</span>
                    <span className="asset-current">{formatCurrency(asset.current_value)}</span>
                    <span className="asset-arrow">→</span>
                    <span className={`asset-scenario ${asset.percent_change >= 0 ? 'positive' : 'negative'}`}>
                      {formatCurrency(asset.scenario_value)}
                    </span>
                    <span className={`asset-change ${asset.percent_change >= 0 ? 'positive' : 'negative'}`}>
                      ({asset.percent_change >= 0 ? '+' : ''}{asset.percent_change.toFixed(0)}%)
                    </span>
                  </div>
                ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default RiskScenarios;
