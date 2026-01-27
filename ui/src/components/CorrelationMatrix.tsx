import usePortfolioStore from '../store/portfolio';

function getCorrelationColor(value: number): string {
  // -1 to 1 range: blue (negative) to white (0) to red (positive)
  if (value > 0.7) return 'corr-high-positive';
  if (value > 0.3) return 'corr-positive';
  if (value > -0.3) return 'corr-neutral';
  if (value > -0.7) return 'corr-negative';
  return 'corr-high-negative';
}

function CorrelationMatrix() {
  const { riskMetrics } = usePortfolioStore();

  if (!riskMetrics || !riskMetrics.correlation_matrix.assets.length) {
    return (
      <div className="correlation-empty">
        <p>Not enough assets to calculate correlation.</p>
      </div>
    );
  }

  const { assets, matrix } = riskMetrics.correlation_matrix;

  // Show max 6 assets for readability
  const displayAssets = assets.slice(0, 6);
  const displayMatrix = matrix.slice(0, 6).map(row => row.slice(0, 6));

  return (
    <div className="correlation-matrix">
      <div className="matrix-grid" style={{
        gridTemplateColumns: `auto repeat(${displayAssets.length}, 1fr)`
      }}>
        {/* Header row */}
        <div className="matrix-cell header-corner"></div>
        {displayAssets.map((asset) => (
          <div key={`header-${asset}`} className="matrix-cell header-cell">
            {asset.slice(0, 4).toUpperCase()}
          </div>
        ))}

        {/* Data rows */}
        {displayAssets.map((rowAsset, i) => (
          <>
            <div key={`row-${rowAsset}`} className="matrix-cell row-header">
              {rowAsset.slice(0, 4).toUpperCase()}
            </div>
            {displayMatrix[i].map((value, j) => (
              <div
                key={`${i}-${j}`}
                className={`matrix-cell data-cell ${getCorrelationColor(value)}`}
                title={`${displayAssets[i]} vs ${displayAssets[j]}: ${(value * 100).toFixed(0)}%`}
              >
                {i === j ? '—' : (value * 100).toFixed(0)}
              </div>
            ))}
          </>
        ))}
      </div>

      <div className="correlation-legend">
        <span className="legend-item">
          <span className="legend-color corr-high-positive"></span>
          High +
        </span>
        <span className="legend-item">
          <span className="legend-color corr-neutral"></span>
          Neutral
        </span>
        <span className="legend-item">
          <span className="legend-color corr-high-negative"></span>
          High −
        </span>
      </div>
    </div>
  );
}

export default CorrelationMatrix;
