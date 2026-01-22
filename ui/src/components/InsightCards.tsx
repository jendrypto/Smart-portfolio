import { useEffect } from 'react';
import usePortfolioStore from '../store/portfolio';
import { Insight } from '../types/Portfolio';

function getColorClass(color: string): string {
  switch (color) {
    case 'lime':
      return 'lime';
    case 'blue':
      return 'blue';
    case 'red':
      return 'red';
    case 'yellow':
      return 'yellow';
    default:
      return 'lime';
  }
}

function InsightCard({ insight, index }: { insight: Insight; index: number }) {
  return (
    <div className="insight-card glass-card">
      <div className={`insight-icon ${getColorClass(insight.color)}`}>
        {insight.icon}
      </div>
      <div className="insight-content">
        <h4>Insight {String(index + 1).padStart(2, '0')}</h4>
        <p>{insight.description}</p>
      </div>
    </div>
  );
}

function InsightCards() {
  const { insights, fetchInsights, positions } = usePortfolioStore();

  // Fetch insights when positions change
  useEffect(() => {
    fetchInsights();
  }, [positions.length, fetchInsights]);

  // Default insights when empty
  const defaultInsights = [
    {
      icon: '💡',
      color: 'lime',
      description: 'Add positions to see portfolio insights.',
    },
    {
      icon: '📊',
      color: 'blue',
      description: 'Track your holdings across multiple chains.',
    },
    {
      icon: '🎯',
      color: 'lime',
      description: 'Monitor your real asset exposure over time.',
    },
  ];

  // Use fetched insights or defaults
  const displayInsights = insights.length > 0 ? insights.slice(0, 3) : [];

  if (displayInsights.length === 0) {
    return (
      <div className="insight-cards">
        {defaultInsights.map((insight, index) => (
          <div key={index} className="insight-card glass-card">
            <div className={`insight-icon ${insight.color}`}>{insight.icon}</div>
            <div className="insight-content">
              <h4>Insight {String(index + 1).padStart(2, '0')}</h4>
              <p>{insight.description}</p>
            </div>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="insight-cards">
      {displayInsights.map((insight, index) => (
        <InsightCard key={insight.id} insight={insight} index={index} />
      ))}
    </div>
  );
}

export default InsightCards;
