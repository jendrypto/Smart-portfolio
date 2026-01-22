import { useState, useEffect, useCallback } from 'react';
import usePortfolioStore from '../store/portfolio';
import { AddPositionRequest, TokenSearchResult } from '../types/Portfolio';

const CHAINS = ['Ethereum', 'Arbitrum', 'Optimism', 'Polygon', 'Base', 'Avalanche', 'BNB Chain', 'Solana', 'Bitcoin'];

function AddPositionModal() {
  const {
    setAddModalOpen,
    addPosition,
    searchTokens,
    tokenSearchResults,
    clearTokenSearch,
    isLoading,
    fetchTokenPrice,
  } = usePortfolioStore();

  const [formData, setFormData] = useState<AddPositionRequest>({
    token_symbol: '',
    token_name: '',
    chain: 'Ethereum',
    quantity: '',
    entry_price_usd: '',
    entry_date: new Date().toISOString().split('T')[0],
  });

  const [searchQuery, setSearchQuery] = useState('');
  const [showResults, setShowResults] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isFetchingPrice, setIsFetchingPrice] = useState(false);

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(() => {
      if (searchQuery.length >= 1) {
        searchTokens(searchQuery);
        setShowResults(true);
      } else {
        clearTokenSearch();
        setShowResults(false);
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [searchQuery, searchTokens, clearTokenSearch]);

  const handleSelectToken = async (token: TokenSearchResult) => {
    setFormData({
      ...formData,
      token_identifier: token.id,
      token_symbol: token.symbol.toUpperCase(),
      token_name: token.name,
    });
    setSearchQuery(token.name);
    setShowResults(false);
    clearTokenSearch();

    // Fetch and auto-populate current price
    setIsFetchingPrice(true);
    const price = await fetchTokenPrice(token.id);
    setIsFetchingPrice(false);

    if (price !== null) {
      setFormData(prev => ({
        ...prev,
        entry_price_usd: price.toString(),
      }));
    }
  };

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.token_symbol.trim()) {
      newErrors.token_symbol = 'Symbol is required';
    }
    if (!formData.token_name.trim()) {
      newErrors.token_name = 'Name is required';
    }
    if (!formData.quantity || parseFloat(formData.quantity) <= 0) {
      newErrors.quantity = 'Quantity must be greater than 0';
    }
    if (!formData.entry_price_usd || parseFloat(formData.entry_price_usd) < 0) {
      newErrors.entry_price_usd = 'Price must be 0 or greater';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;

    const success = await addPosition(formData);
    if (success) {
      setAddModalOpen(false);
    }
  };

  const handleClose = () => {
    clearTokenSearch();
    setAddModalOpen(false);
  };

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Add Position</h2>
          <button className="modal-close" onClick={handleClose}>×</button>
        </div>

        <form onSubmit={handleSubmit} className="modal-form">
          <div className="form-group">
            <label>Search Token</label>
            <div className="search-container">
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search by name or symbol..."
                autoFocus
              />
              {showResults && tokenSearchResults.length > 0 && (
                <div className="search-results">
                  {tokenSearchResults.map((token) => (
                    <div
                      key={token.id}
                      className="search-result"
                      onClick={() => handleSelectToken(token)}
                    >
                      <span className="result-name">{token.name}</span>
                      <span className="result-symbol">{token.symbol.toUpperCase()}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>Symbol *</label>
              <input
                type="text"
                value={formData.token_symbol}
                onChange={(e) => setFormData({ ...formData, token_symbol: e.target.value.toUpperCase() })}
                placeholder="ETH"
                className={errors.token_symbol ? 'error' : ''}
              />
              {errors.token_symbol && <span className="error-text">{errors.token_symbol}</span>}
            </div>

            <div className="form-group">
              <label>Name *</label>
              <input
                type="text"
                value={formData.token_name}
                onChange={(e) => setFormData({ ...formData, token_name: e.target.value })}
                placeholder="Ethereum"
                className={errors.token_name ? 'error' : ''}
              />
              {errors.token_name && <span className="error-text">{errors.token_name}</span>}
            </div>
          </div>

          <div className="form-group">
            <label>Chain</label>
            <select
              value={formData.chain}
              onChange={(e) => setFormData({ ...formData, chain: e.target.value })}
            >
              {CHAINS.map((chain) => (
                <option key={chain} value={chain}>{chain}</option>
              ))}
            </select>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>Quantity *</label>
              <input
                type="number"
                step="any"
                value={formData.quantity}
                onChange={(e) => setFormData({ ...formData, quantity: e.target.value })}
                placeholder="1.5"
                className={errors.quantity ? 'error' : ''}
              />
              {errors.quantity && <span className="error-text">{errors.quantity}</span>}
            </div>

            <div className="form-group">
              <label>Entry Price (USD) * {isFetchingPrice && <span className="fetching-price">(fetching...)</span>}</label>
              <input
                type="number"
                step="any"
                value={formData.entry_price_usd}
                onChange={(e) => setFormData({ ...formData, entry_price_usd: e.target.value })}
                placeholder="3000.00"
                className={errors.entry_price_usd ? 'error' : ''}
              />
              {errors.entry_price_usd && <span className="error-text">{errors.entry_price_usd}</span>}
            </div>
          </div>

          <div className="form-group">
            <label>Entry Date</label>
            <input
              type="date"
              value={formData.entry_date || ''}
              onChange={(e) => setFormData({ ...formData, entry_date: e.target.value })}
            />
          </div>

          <div className="form-group">
            <label>Note (optional)</label>
            <textarea
              value={formData.user_note || ''}
              onChange={(e) => setFormData({ ...formData, user_note: e.target.value })}
              placeholder="Any notes about this position..."
              rows={2}
            />
          </div>

          <div className="form-actions">
            <button type="button" className="btn btn-secondary" onClick={handleClose}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary" disabled={isLoading}>
              {isLoading ? 'Adding...' : 'Add Position'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default AddPositionModal;
