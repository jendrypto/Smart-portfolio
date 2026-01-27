import { useMemo, useState } from 'react';
import usePortfolioStore from '../store/portfolio';
import { SortField } from '../types/Portfolio';

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
}

function formatPercent(value: number): string {
  const sign = value >= 0 ? '+' : '';
  return `${sign}${value.toFixed(2)}%`;
}

function formatQuantity(value: string): string {
  const num = parseFloat(value);
  if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(2)}K`;
  if (num < 0.0001) return num.toExponential(2);
  return num.toFixed(4);
}

function getTokenIcon(symbol: string): { class: string; text: string } {
  const sym = symbol.toUpperCase();
  if (sym === 'BTC' || sym === 'BITCOIN' || sym === 'WBTC') {
    return { class: 'btc', text: '₿' };
  }
  if (sym === 'ETH' || sym === 'ETHEREUM' || sym === 'WETH' || sym === 'STETH') {
    return { class: 'eth', text: '◆' };
  }
  return { class: 'other', text: sym.charAt(0) };
}

function PositionsTable() {
  const {
    positions,
    sortField,
    sortDirection,
    setSorting,
    deletePosition,
    setEditingPositionId,
    isLoading,
    summary,
  } = usePortfolioStore();

  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);
  const [activeRowId, setActiveRowId] = useState<string | null>(null);
  const [editMode, setEditMode] = useState(false);

  const toggleRowActions = (id: string) => {
    if (activeRowId === id) {
      setActiveRowId(null);
      setConfirmDelete(null);
    } else {
      setActiveRowId(id);
      setConfirmDelete(null);
    }
  };

  const sortedPositions = useMemo(() => {
    const sorted = [...positions].sort((a, b) => {
      let aVal: number, bVal: number;
      switch (sortField) {
        case 'value':
          aVal = a.position_value_usd;
          bVal = b.position_value_usd;
          break;
        case 'pnl':
          aVal = a.unrealized_pnl_usd;
          bVal = b.unrealized_pnl_usd;
          break;
        case 'allocation':
          aVal = a.allocation_percent;
          bVal = b.allocation_percent;
          break;
        default:
          aVal = a.position_value_usd;
          bVal = b.position_value_usd;
      }
      return sortDirection === 'desc' ? bVal - aVal : aVal - bVal;
    });
    return sorted;
  }, [positions, sortField, sortDirection]);

  const handleSort = (field: SortField) => {
    if (field === sortField) {
      setSorting(field, sortDirection === 'desc' ? 'asc' : 'desc');
    } else {
      setSorting(field, 'desc');
    }
  };

  const handleDelete = async (id: string) => {
    if (confirmDelete === id) {
      await deletePosition(id);
      setConfirmDelete(null);
      setActiveRowId(null);
    } else {
      setConfirmDelete(id);
    }
  };

  const handleEdit = (id: string) => {
    setEditingPositionId(id);
    setActiveRowId(null);
  };

  const getSortIcon = (field: SortField) => {
    if (field !== sortField) return '↕';
    return sortDirection === 'desc' ? '↓' : '↑';
  };

  const lastUpdate = summary?.last_price_update
    ? new Date(summary.last_price_update * 1000).toLocaleTimeString()
    : 'Never';

  return (
    <div className="positions-panel glass-panel">
      <div className="positions-header">
        <h2 className="positions-title">
          Positions
          <span className="positions-count">{positions.length}</span>
        </h2>
        <div className="positions-meta">
          <button
            className={`btn btn-small ${editMode ? 'btn-edit-active' : 'btn-edit'}`}
            onClick={() => {
              if (editMode) {
                setActiveRowId(null);
                setConfirmDelete(null);
              }
              setEditMode(!editMode);
            }}
          >
            {editMode ? 'Done' : 'Edit'}
          </button>
          Last price update: <span>{lastUpdate}</span>
        </div>
      </div>

      <div className="positions-table-container">
        <table className="positions-table">
          <thead>
            <tr>
              {editMode && <th style={{ width: '32px' }}></th>}
              <th>Token</th>
              <th>Chain</th>
              <th className="text-right">Quantity</th>
              <th className="text-right">Price</th>
              <th className="text-right sortable" onClick={() => handleSort('value')}>
                Value {getSortIcon('value')}
              </th>
              <th className="text-right sortable" onClick={() => handleSort('pnl')}>
                P&L {getSortIcon('pnl')}
              </th>
              {editMode && <th className="text-center"></th>}
            </tr>
          </thead>
          <tbody>
            {sortedPositions.map((p) => {
              const icon = getTokenIcon(p.position.token_symbol);
              const pnlIsPositive = p.unrealized_pnl_usd >= 0;
              const isActive = activeRowId === p.position.id;

              return (
                <tr key={p.position.id} className={isActive ? 'row-active' : ''}>
                  {editMode && (
                    <td className="text-center">
                      <button
                        className={`pencil-btn ${isActive ? 'active' : ''}`}
                        onClick={() => toggleRowActions(p.position.id)}
                        title="Edit options"
                      >
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/>
                        </svg>
                      </button>
                    </td>
                  )}
                  <td>
                    <div className="token-cell">
                      <div className={`token-icon ${icon.class}`}>{icon.text}</div>
                      <div className="token-info">
                        <span className="token-name">{p.position.token_name}</span>
                        <span className="token-symbol">{p.position.token_symbol}</span>
                      </div>
                    </div>
                  </td>
                  <td>
                    <span className="chain-badge">{p.position.chain}</span>
                  </td>
                  <td className="text-right">
                    <span className="table-value">{formatQuantity(p.position.quantity)}</span>
                  </td>
                  <td className="text-right">
                    <span className="table-value white">{formatCurrency(p.current_price_usd)}</span>
                  </td>
                  <td className="text-right">
                    <span className="table-value primary">{formatCurrency(p.position_value_usd)}</span>
                  </td>
                  <td className="text-right">
                    <span className={`table-value ${pnlIsPositive ? 'positive' : 'negative'}`}>
                      {formatCurrency(p.unrealized_pnl_usd)}
                    </span>
                  </td>
                  {editMode && (
                    <td className="text-center">
                      {isActive && (
                        <div className="actions-cell">
                          <button
                            className="btn btn-small btn-edit"
                            onClick={() => handleEdit(p.position.id)}
                            disabled={isLoading}
                          >
                            Edit
                          </button>
                          <button
                            className={`btn btn-small ${confirmDelete === p.position.id ? 'btn-danger-confirm' : 'btn-danger'}`}
                            onClick={() => handleDelete(p.position.id)}
                            disabled={isLoading}
                          >
                            {confirmDelete === p.position.id ? 'Confirm?' : 'Delete'}
                          </button>
                        </div>
                      )}
                    </td>
                  )}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div className="positions-footer">
        <div className="positions-footer-info">
          Showing <span>{positions.length}</span> assets
        </div>
        <div className="pagination-btns">
          <button className="pagination-btn" disabled>‹</button>
          <button className="pagination-btn" disabled>›</button>
        </div>
      </div>
    </div>
  );
}

export default PositionsTable;
