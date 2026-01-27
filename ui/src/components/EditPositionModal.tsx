import { useState, useEffect } from 'react';
import usePortfolioStore from '../store/portfolio';
import { AddPositionRequest } from '../types/Portfolio';

const CHAINS = ['Ethereum', 'Arbitrum', 'Optimism', 'Polygon', 'Base', 'Avalanche', 'BNB Chain', 'Solana', 'Bitcoin'];

function EditPositionModal() {
  const {
    positions,
    editingPositionId,
    setEditingPositionId,
    updatePosition,
    isLoading,
  } = usePortfolioStore();

  const position = positions.find((p) => p.position.id === editingPositionId)?.position;

  const [formData, setFormData] = useState<Partial<AddPositionRequest>>({
    token_symbol: '',
    token_name: '',
    chain: 'Ethereum',
    quantity: '',
    entry_price_usd: '',
    entry_date: '',
    user_note: '',
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  // Populate form when position changes
  useEffect(() => {
    if (position) {
      setFormData({
        token_symbol: position.token_symbol,
        token_name: position.token_name,
        chain: position.chain,
        quantity: position.quantity,
        entry_price_usd: position.entry_price_usd,
        entry_date: position.entry_date || '',
        user_note: position.user_note || '',
      });
    }
  }, [position]);

  if (!position) {
    return null;
  }

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.token_symbol?.trim()) {
      newErrors.token_symbol = 'Symbol is required';
    }
    if (!formData.token_name?.trim()) {
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

    const success = await updatePosition(editingPositionId!, formData);
    if (success) {
      setEditingPositionId(null);
    }
  };

  const handleClose = () => {
    setEditingPositionId(null);
  };

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Edit Position</h2>
          <button className="modal-close" onClick={handleClose}>×</button>
        </div>

        <form onSubmit={handleSubmit} className="modal-form">
          <div className="form-row">
            <div className="form-group">
              <label>Symbol *</label>
              <input
                type="text"
                value={formData.token_symbol || ''}
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
                value={formData.token_name || ''}
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
              value={formData.chain || 'Ethereum'}
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
                value={formData.quantity || ''}
                onChange={(e) => setFormData({ ...formData, quantity: e.target.value })}
                placeholder="1.5"
                className={errors.quantity ? 'error' : ''}
              />
              {errors.quantity && <span className="error-text">{errors.quantity}</span>}
            </div>

            <div className="form-group">
              <label>Entry Price (USD) *</label>
              <input
                type="number"
                step="any"
                value={formData.entry_price_usd || ''}
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
              {isLoading ? 'Saving...' : 'Save Changes'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default EditPositionModal;
