import { useState, useEffect } from 'react';
import { ExposureEntry, NewsItem, PositionWithDerived } from '../types/Portfolio';
import { API_BASE } from '../utils/api';

interface Props {
  category: string | null;
  exposure: ExposureEntry | null;
  positions: PositionWithDerived[];
  onClose: () => void;
}

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
}

const CATEGORY_CURRENCIES: Record<string, string> = {
  'ETH': 'ETH',
  'BTC': 'BTC',
  'Stablecoins': 'USDC,USDT,DAI',
  'Alt L1s': 'SOL,AVAX,DOT,ATOM,NEAR,ADA,APT,SUI',
  'Protocol Tokens': 'UNI,AAVE,MKR,LINK,ARB,OP',
};

function formatNewsDate(dateStr: string): string {
  if (!dateStr) return '';
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffH = Math.floor(diffMs / (1000 * 60 * 60));
    if (diffH < 1) return 'Just now';
    if (diffH < 24) return `${diffH}h ago`;
    const diffD = Math.floor(diffH / 24);
    if (diffD < 7) return `${diffD}d ago`;
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  } catch {
    return '';
  }
}

function ExposureSidePanel({ category, exposure, positions, onClose }: Props) {
  const [news, setNews] = useState<NewsItem[]>([]);
  const [newsLoading, setNewsLoading] = useState(false);

  useEffect(() => {
    if (!category) {
      setNews([]);
      return;
    }

    const currencies = CATEGORY_CURRENCIES[category] || '';
    if (!currencies) {
      setNews([]);
      return;
    }

    setNewsLoading(true);
    fetch(`${API_BASE}/api/news?currencies=${encodeURIComponent(currencies)}`)
      .then(async (r) => {
        if (!r.ok) throw new Error('Failed to fetch news');
        const data = await r.json();
        setNews(data.news || []);
      })
      .catch((e) => {
        console.error('News fetch error:', e);
        setNews([]);
      })
      .finally(() => setNewsLoading(false));
  }, [category]);

  if (!category || !exposure) return null;

  // Filter positions that contribute to this category
  const relevantPositions = positions.filter(p => {
    const sym = p.position.token_symbol.toUpperCase();
    switch (category) {
      case 'ETH':
        return ['ETH', 'WETH', 'STETH', 'RETH', 'CBETH', 'WSTETH', 'FRXETH', 'SWETH'].includes(sym);
      case 'BTC':
        return ['BTC', 'WBTC', 'TBTC', 'CBBTC', 'RENBTC', 'HBTC'].includes(sym);
      case 'Stablecoins':
        return ['USDC', 'USDT', 'DAI', 'FRAX', 'LUSD', 'GUSD', 'BUSD', 'TUSD', 'USDP', 'PYUSD', 'SUSD', 'MIM', 'CRVUSD', 'GHO'].includes(sym);
      case 'Alt L1s':
        return ['SOL', 'AVAX', 'MATIC', 'DOT', 'ATOM', 'NEAR', 'ADA', 'FTM', 'ALGO', 'XLM', 'ICP', 'APT', 'SUI', 'SEI', 'INJ'].includes(sym);
      case 'Protocol Tokens':
        return ['UNI', 'AAVE', 'MKR', 'CRV', 'COMP', 'SNX', 'LDO', 'RPL', 'GMX', 'DYDX', 'LINK', 'GRT', 'ENS', 'OP', 'ARB', 'PENDLE', 'ENA', 'EIGEN'].includes(sym);
      default:
        // For "Other", include positions not in any other category
        const allKnown = [
          'ETH', 'WETH', 'STETH', 'RETH', 'CBETH', 'WSTETH', 'FRXETH', 'SWETH',
          'BTC', 'WBTC', 'TBTC', 'CBBTC', 'RENBTC', 'HBTC',
          'USDC', 'USDT', 'DAI', 'FRAX', 'LUSD', 'GUSD', 'BUSD', 'TUSD', 'USDP', 'PYUSD', 'SUSD', 'MIM', 'CRVUSD', 'GHO',
          'SOL', 'AVAX', 'MATIC', 'DOT', 'ATOM', 'NEAR', 'ADA', 'FTM', 'ALGO', 'XLM', 'ICP', 'APT', 'SUI', 'SEI', 'INJ',
          'UNI', 'AAVE', 'MKR', 'CRV', 'COMP', 'SNX', 'LDO', 'RPL', 'GMX', 'DYDX', 'LINK', 'GRT', 'ENS', 'OP', 'ARB', 'PENDLE', 'ENA', 'EIGEN'
        ];
        return !allKnown.includes(sym);
    }
  });

  return (
    <div className="side-panel-overlay" onClick={onClose}>
      <div className="side-panel" onClick={(e) => e.stopPropagation()}>
        <div className="side-panel-header">
          <h3>{category} Exposure</h3>
          <button className="side-panel-close" onClick={onClose}>&times;</button>
        </div>

        <div className="side-panel-summary">
          <div className="summary-stat">
            <span className="stat-label">Total Value</span>
            <span className="stat-value">{formatCurrency(exposure.value_usd)}</span>
          </div>
          <div className="summary-stat">
            <span className="stat-label">Portfolio %</span>
            <span className="stat-value">{exposure.percentage.toFixed(1)}%</span>
          </div>
        </div>

        <div className="side-panel-section">
          <h4>Confidence Breakdown</h4>
          <div className="confidence-breakdown">
            {exposure.confidence_breakdown && exposure.confidence_breakdown.map((cb) => (
              <div key={cb.level} className="breakdown-row">
                <span className={`confidence-badge confidence-${cb.level.toLowerCase()}`}>
                  {cb.level}
                </span>
                <span className="breakdown-value">{formatCurrency(cb.value_usd)}</span>
                <span className="breakdown-percent">{cb.percentage.toFixed(0)}%</span>
              </div>
            ))}
          </div>
        </div>

        <div className="side-panel-section">
          <h4>Contributing Positions</h4>
          <div className="positions-list">
            {relevantPositions.length > 0 ? (
              relevantPositions.map((p) => (
                <div key={p.position.id} className="position-row">
                  <div className="position-info">
                    <span className="position-symbol">{p.position.token_symbol}</span>
                    <span className="position-chain">{p.position.chain}</span>
                  </div>
                  <span className="position-value">{formatCurrency(p.position_value_usd)}</span>
                </div>
              ))
            ) : (
              <p className="no-positions">No positions found for this category.</p>
            )}
          </div>
        </div>

        {exposure.notes && (
          <div className="side-panel-section">
            <h4>Notes</h4>
            <p className="notes-text">{exposure.notes}</p>
          </div>
        )}

        {/* Latest News Section */}
        <div className="side-panel-section">
          <h4>Latest News</h4>
          {newsLoading ? (
            <div className="news-list">
              <div className="skeleton-block" style={{ height: '60px', borderRadius: '8px', marginBottom: '8px' }} />
              <div className="skeleton-block" style={{ height: '60px', borderRadius: '8px' }} />
            </div>
          ) : news.length > 0 ? (
            <div className="news-list">
              {news.slice(0, 5).map((item, i) => (
                <a
                  key={i}
                  href={item.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="news-item"
                >
                  <span className="news-title">{item.title}</span>
                  <span className="news-meta">
                    <span>{item.source}</span>
                    <span>{formatNewsDate(item.published_at)}</span>
                    {(item.positive_votes > 0 || item.negative_votes > 0) && (
                      <span className="news-votes">
                        +{item.positive_votes} / -{item.negative_votes}
                      </span>
                    )}
                  </span>
                </a>
              ))}
            </div>
          ) : (
            <p className="no-positions">No recent news available.</p>
          )}
        </div>
      </div>
    </div>
  );
}

export default ExposureSidePanel;
