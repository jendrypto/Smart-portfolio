import { useMemo } from 'react';
import { DailySnapshot } from '../types/Portfolio';

interface Props {
  snapshots: DailySnapshot[];
}

function PortfolioChart({ snapshots }: Props) {
  const chartData = useMemo(() => {
    if (snapshots.length === 0) return null;

    const sortedSnapshots = [...snapshots].sort((a, b) =>
      new Date(a.date).getTime() - new Date(b.date).getTime()
    );

    const values = sortedSnapshots.map(s => s.total_value_usd);
    const minValue = Math.min(...values);
    const maxValue = Math.max(...values);
    const range = maxValue - minValue || 1;

    // SVG dimensions
    const width = 1000;
    const height = 200;
    const padding = { top: 10, right: 10, bottom: 10, left: 10 };
    const chartWidth = width - padding.left - padding.right;
    const chartHeight = height - padding.top - padding.bottom;

    // Generate path
    const points = sortedSnapshots.map((snapshot, i) => {
      const x = padding.left + (i / (sortedSnapshots.length - 1 || 1)) * chartWidth;
      const y = padding.top + chartHeight - ((snapshot.total_value_usd - minValue) / range) * chartHeight;
      return { x, y, snapshot };
    });

    // Create smooth curve using quadratic bezier
    let pathD = '';
    let areaD = '';

    if (points.length === 1) {
      // Single point - draw a horizontal line
      pathD = `M ${points[0].x} ${points[0].y} L ${width - padding.right} ${points[0].y}`;
      areaD = `M ${points[0].x} ${height - padding.bottom} L ${points[0].x} ${points[0].y} L ${width - padding.right} ${points[0].y} L ${width - padding.right} ${height - padding.bottom} Z`;
    } else {
      // Multiple points - create smooth curve
      pathD = `M ${points[0].x} ${points[0].y}`;

      for (let i = 0; i < points.length - 1; i++) {
        const current = points[i];
        const next = points[i + 1];
        const midX = (current.x + next.x) / 2;
        const midY = (current.y + next.y) / 2;

        if (i === 0) {
          pathD += ` Q ${current.x} ${current.y}, ${midX} ${midY}`;
        } else {
          pathD += ` T ${midX} ${midY}`;
        }
      }

      // End at the last point
      pathD += ` T ${points[points.length - 1].x} ${points[points.length - 1].y}`;

      // Area path (for gradient fill)
      areaD = pathD + ` L ${points[points.length - 1].x} ${height - padding.bottom} L ${points[0].x} ${height - padding.bottom} Z`;
    }

    return { points, pathD, areaD, minValue, maxValue, width, height, padding };
  }, [snapshots]);

  if (!chartData || snapshots.length < 1) {
    return (
      <div className="chart-placeholder">
        <p>Not enough data to display chart. Add positions and refresh prices to start tracking.</p>
      </div>
    );
  }

  const formatValue = (v: number) => {
    if (v >= 1000000) return `$${(v / 1000000).toFixed(1)}M`;
    if (v >= 1000) return `$${(v / 1000).toFixed(1)}K`;
    return `$${v.toFixed(0)}`;
  };

  return (
    <div className="portfolio-chart">
      <svg viewBox={`0 0 ${chartData.width} ${chartData.height}`} className="chart-svg" preserveAspectRatio="none">
        <defs>
          <linearGradient id="lineGradient" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#0F52FF" />
            <stop offset="50%" stopColor="#7C3AED" />
            <stop offset="100%" stopColor="#D9FD65" />
          </linearGradient>
          <linearGradient id="fillGradient" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stopColor="#D9FD65" stopOpacity="0.15" />
            <stop offset="100%" stopColor="#D9FD65" stopOpacity="0" />
          </linearGradient>
        </defs>

        {/* Area fill */}
        <path d={chartData.areaD} fill="url(#fillGradient)" />

        {/* Glow effect (blurred duplicate) */}
        <path
          d={chartData.pathD}
          fill="none"
          stroke="url(#lineGradient)"
          strokeWidth="8"
          strokeLinecap="round"
          opacity="0.5"
          style={{ filter: 'blur(4px)' }}
        />

        {/* Main line */}
        <path
          d={chartData.pathD}
          fill="none"
          stroke="url(#lineGradient)"
          strokeWidth="4"
          strokeLinecap="round"
          className="chart-path"
          style={{ filter: 'drop-shadow(0 0 4px rgba(255, 255, 255, 0.5))' }}
        />

        {/* Data points (only show a few key points) */}
        {chartData.points.length <= 10 && chartData.points.map((point, i) => (
          <g key={i} className="data-point-group">
            <circle
              cx={point.x}
              cy={point.y}
              r={4}
              className="data-point"
              style={{ filter: 'drop-shadow(0 0 8px rgba(217, 253, 101, 0.5))' }}
            />
            <title>
              {point.snapshot.date}: {formatValue(point.snapshot.total_value_usd)}
            </title>
          </g>
        ))}
      </svg>
    </div>
  );
}

export default PortfolioChart;
