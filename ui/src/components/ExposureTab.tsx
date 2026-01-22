import usePortfolioStore from '../store/portfolio';
import { ConfidenceLevel } from '../types/Portfolio';

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
      return 'Direct holding with clear backing';
    case 'Medium':
      return 'Wrapped or bridged asset with known backing';
    case 'Low':
      return 'Complex or synthetic asset';
    default:
      return '';
  }
}

function ExposureTab() {
  const { exposure, isLoading, positions } = usePortfolioStore();

  if (isLoading && !exposure) {
    return <div className="loading">Loading exposure data...</div>;
  }

  if (positions.length === 0) {
    return (
      <div className="empty-state">
        <h2>No positions yet</h2>
        <p>Add positions to see your exposure analysis.</p>
      </div>
    );
  }

  if (!exposure) {
    return (
      <div className="empty-state">
        <h2>Exposure data unavailable</h2>
        <p>Try refreshing the page.</p>
      </div>
    );
  }

  return (
    <div className="exposure-tab">
      <section className="exposure-section glass-panel">
        <h3>Underlying Exposure</h3>
        <p className="section-description">
          Reinterpretation of your holdings into understandable asset categories.
        </p>

        <div className="exposure-table-container">
          <table className="exposure-table">
            <thead>
              <tr>
                <th>Category</th>
                <th>USD Value</th>
                <th>% of Portfolio</th>
                <th>Confidence</th>
                <th>Notes</th>
              </tr>
            </thead>
            <tbody>
              {exposure.underlying_exposure
                .sort((a, b) => b.percentage - a.percentage)
                .map((entry) => (
                  <tr key={entry.category}>
                    <td className="category-cell">
                      <span className={`category-badge category-${entry.category.toLowerCase()}`}>
                        {entry.category}
                      </span>
                    </td>
                    <td>{formatCurrency(entry.value_usd)}</td>
                    <td>
                      <div className="percentage-cell">
                        <div
                          className="percentage-bar"
                          style={{ width: `${Math.min(entry.percentage, 100)}%` }}
                        />
                        <span>{entry.percentage.toFixed(1)}%</span>
                      </div>
                    </td>
                    <td>
                      <span
                        className={`confidence-badge ${getConfidenceClass(entry.confidence)}`}
                        title={getConfidenceTooltip(entry.confidence)}
                      >
                        {entry.confidence}
                      </span>
                    </td>
                    <td className="notes-cell">{entry.notes || '-'}</td>
                  </tr>
                ))}
            </tbody>
          </table>
        </div>

        <div className="exposure-bars">
          {exposure.underlying_exposure
            .sort((a, b) => b.percentage - a.percentage)
            .map((entry) => (
              <div key={entry.category} className="exposure-bar-item">
                <div className="bar-label">
                  <span className={`category-badge category-${entry.category.toLowerCase()}`}>
                    {entry.category}
                  </span>
                  <span>{entry.percentage.toFixed(1)}%</span>
                </div>
                <div className="bar-track">
                  <div
                    className={`bar-fill category-${entry.category.toLowerCase()}`}
                    style={{ width: `${Math.min(entry.percentage, 100)}%` }}
                  />
                </div>
              </div>
            ))}
        </div>
      </section>

      <section className="exposure-section glass-panel">
        <h3>Chain Exposure</h3>
        <p className="section-description">
          Distribution of your holdings across different blockchains.
        </p>

        <div className="exposure-table-container">
          <table className="exposure-table">
            <thead>
              <tr>
                <th>Chain</th>
                <th>USD Value</th>
                <th>% of Portfolio</th>
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
                    <td>{formatCurrency(chain.value_usd)}</td>
                    <td>
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
          <strong>Note:</strong> Exposure analysis is based on known asset mappings.
          Complex DeFi positions, derivatives, or less common assets may be classified as "Other".
          Always verify your actual exposure independently.
        </p>
      </div>
    </div>
  );
}

export default ExposureTab;
