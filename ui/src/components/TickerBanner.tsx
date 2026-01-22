import usePortfolioStore from '../store/portfolio';
import { useEffect } from 'react';

function formatPrice(price: number): string {
  if (price >= 1000) return `$${price.toLocaleString('en-US', { maximumFractionDigits: 0 })}`;
  if (price >= 1) return `$${price.toFixed(2)}`;
  return `$${price.toFixed(4)}`;
}

function TickerBanner() {
  const { topTokens, fetchTopTokens } = usePortfolioStore();

  useEffect(() => {
    fetchTopTokens();
    const interval = setInterval(fetchTopTokens, 60000);
    return () => clearInterval(interval);
  }, [fetchTopTokens]);

  if (topTokens.length === 0) return null;

  const displayTokens = [...topTokens, ...topTokens];

  return (
    <div className="ticker-banner">
      <div className="ticker-track">
        {displayTokens.map((token, index) => (
          <div key={`${token.id}-${index}`} className="ticker-item">
            <span className="ticker-symbol">{token.symbol}</span>
            <span className="ticker-price">{formatPrice(token.current_price)}</span>
            <span className={`ticker-change ${token.price_change_percentage_24h >= 0 ? 'positive' : 'negative'}`}>
              {token.price_change_percentage_24h >= 0 ? '+' : ''}{token.price_change_percentage_24h.toFixed(2)}%
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

export default TickerBanner;
