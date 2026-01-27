import { ExposureEntry } from '../types/Portfolio';

interface Props {
  data: ExposureEntry[];
  onCategoryClick: (category: string) => void;
}

const CATEGORY_COLORS: Record<string, string> = {
  'ETH': '#627EEA',
  'BTC': '#F7931A',
  'Stablecoins': '#26A17B',
  'Alt L1s': '#E84142',
  'Protocol Tokens': '#8B5CF6',
  'Other': '#6B7280',
};

function ExposureDonutChart({ data, onCategoryClick }: Props) {
  const total = data.reduce((sum, d) => sum + d.value_usd, 0);
  const sortedData = [...data].sort((a, b) => b.value_usd - a.value_usd);

  // Calculate SVG arc paths
  let currentAngle = -90; // Start at top
  const arcs = sortedData.map((entry) => {
    const angle = (entry.value_usd / total) * 360;
    const startAngle = currentAngle;
    const endAngle = currentAngle + angle;
    currentAngle = endAngle;

    const startRad = (startAngle * Math.PI) / 180;
    const endRad = (endAngle * Math.PI) / 180;
    const largeArc = angle > 180 ? 1 : 0;

    const outerR = 100;
    const innerR = 60;

    const x1 = 120 + outerR * Math.cos(startRad);
    const y1 = 120 + outerR * Math.sin(startRad);
    const x2 = 120 + outerR * Math.cos(endRad);
    const y2 = 120 + outerR * Math.sin(endRad);
    const x3 = 120 + innerR * Math.cos(endRad);
    const y3 = 120 + innerR * Math.sin(endRad);
    const x4 = 120 + innerR * Math.cos(startRad);
    const y4 = 120 + innerR * Math.sin(startRad);

    const path = `M ${x1} ${y1} A ${outerR} ${outerR} 0 ${largeArc} 1 ${x2} ${y2} L ${x3} ${y3} A ${innerR} ${innerR} 0 ${largeArc} 0 ${x4} ${y4} Z`;

    return {
      path,
      color: CATEGORY_COLORS[entry.category] || CATEGORY_COLORS['Other'],
      category: entry.category,
      percentage: entry.percentage,
    };
  });

  return (
    <div className="donut-chart-container">
      <svg viewBox="0 0 240 240" className="donut-chart">
        {arcs.map((arc) => (
          <path
            key={arc.category}
            d={arc.path}
            fill={arc.color}
            className="donut-segment"
            onClick={() => onCategoryClick(arc.category)}
          />
        ))}
        <text x="120" y="115" textAnchor="middle" className="donut-total-label">
          Total Value
        </text>
        <text x="120" y="135" textAnchor="middle" className="donut-total-value">
          ${(total / 1000).toFixed(1)}K
        </text>
      </svg>
      <div className="donut-legend">
        {sortedData.map((entry) => (
          <div
            key={entry.category}
            className="legend-item"
            onClick={() => onCategoryClick(entry.category)}
          >
            <span
              className="legend-color"
              style={{ backgroundColor: CATEGORY_COLORS[entry.category] || CATEGORY_COLORS['Other'] }}
            />
            <span className="legend-label">{entry.category}</span>
            <span className="legend-value">{entry.percentage.toFixed(1)}%</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export default ExposureDonutChart;
