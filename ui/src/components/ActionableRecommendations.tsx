import usePortfolioStore from '../store/portfolio';

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(value);
}

function getPriorityClass(priority: string): string {
  switch (priority) {
    case 'critical': return 'priority-critical';
    case 'high': return 'priority-high';
    case 'medium': return 'priority-medium';
    case 'low': return 'priority-low';
    default: return '';
  }
}

function getCategoryIcon(category: string): string {
  switch (category) {
    case 'risk': return '⚠️';
    case 'opportunity': return '💡';
    case 'rebalance': return '⚖️';
    case 'alert': return '🔔';
    default: return '📊';
  }
}

function ActionableRecommendations() {
  const { recommendations, riskMetrics } = usePortfolioStore();

  if (recommendations.length === 0) {
    return (
      <div className="recommendations-empty">
        <p>No recommendations at this time. Your portfolio looks balanced.</p>
      </div>
    );
  }

  return (
    <div className="actionable-recommendations">
      {riskMetrics && (
        <div className="risk-score-header">
          <span className="risk-score-label">Portfolio Risk Score</span>
          <span className={`risk-score-value risk-${riskMetrics.risk_score > 70 ? 'high' : riskMetrics.risk_score > 40 ? 'medium' : 'low'}`}>
            {riskMetrics.risk_score}/100
          </span>
        </div>
      )}

      <div className="recommendations-list">
        {recommendations.map((rec) => (
          <div key={rec.id} className={`recommendation-card ${getPriorityClass(rec.priority)}`}>
            <div className="rec-header">
              <span className="rec-icon">{getCategoryIcon(rec.category)}</span>
              <span className={`rec-priority-badge ${getPriorityClass(rec.priority)}`}>
                {rec.priority.toUpperCase()}
              </span>
            </div>
            <h4 className="rec-title">{rec.title}</h4>
            <p className="rec-description">{rec.description}</p>
            <p className="rec-impact">{rec.impact}</p>
            {rec.action && (
              <div className="rec-action">
                <span className="action-label">{rec.action.label}</span>
                {rec.action.from_asset && rec.action.to_asset && (
                  <span className="action-details">
                    {rec.action.from_asset} → {rec.action.to_asset}
                  </span>
                )}
                {rec.action.estimated_value && (
                  <span className="action-value">
                    ~{formatCurrency(rec.action.estimated_value)}
                  </span>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

export default ActionableRecommendations;
