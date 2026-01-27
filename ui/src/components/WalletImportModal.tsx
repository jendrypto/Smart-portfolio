import { useState } from 'react';
import usePortfolioStore from '../store/portfolio';
import { WalletToken, ScanResult } from '../types/Portfolio';

const EVM_CHAINS = ['Ethereum', 'Arbitrum', 'Optimism', 'Base', 'Polygon', 'Avalanche', 'BNB Chain'];

type Step = 'input' | 'scanning' | 'results';

function WalletImportModal() {
  const {
    setWalletModalOpen,
    scanWallet,
    importWalletTokens,
    isWalletScanning,
    isLoading,
  } = usePortfolioStore();

  const [step, setStep] = useState<Step>('input');
  const [address, setAddress] = useState('');
  const [selectedChains, setSelectedChains] = useState<string[]>([...EVM_CHAINS]);
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [tokens, setTokens] = useState<WalletToken[]>([]);
  const [showDust, setShowDust] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [scanningChain, setScanningChain] = useState('');

  const toggleChain = (chain: string) => {
    setSelectedChains(prev =>
      prev.includes(chain)
        ? prev.filter(c => c !== chain)
        : [...prev, chain]
    );
  };

  const toggleAllChains = () => {
    if (selectedChains.length === EVM_CHAINS.length) {
      setSelectedChains([]);
    } else {
      setSelectedChains([...EVM_CHAINS]);
    }
  };

  const isValidAddress = (addr: string) => {
    return /^0x[a-fA-F0-9]{40}$/.test(addr);
  };

  const handleScan = async () => {
    setError(null);

    if (!isValidAddress(address)) {
      setError('Invalid wallet address. Expected 0x + 40 hex characters.');
      return;
    }

    if (selectedChains.length === 0) {
      setError('Select at least one chain to scan.');
      return;
    }

    setStep('scanning');
    setScanningChain(`Scanning ${selectedChains.length} chains...`);

    const result = await scanWallet(address, selectedChains);

    if (result) {
      setScanResult(result);
      setTokens(result.tokens.map(t => ({ ...t, selected: true })));
      setStep('results');
    } else {
      setStep('input');
      const storeError = usePortfolioStore.getState().error;
      setError(storeError || 'Scan failed. Please try again.');
    }
  };

  const toggleToken = (index: number) => {
    setTokens(prev => prev.map((t, i) =>
      i === index ? { ...t, selected: !t.selected } : t
    ));
  };

  const toggleAllTokens = () => {
    const allSelected = tokens.every(t => t.selected);
    setTokens(prev => prev.map(t => ({ ...t, selected: !allSelected })));
  };

  const handleImport = async () => {
    const selectedTokens = tokens.filter(t => t.selected);
    if (selectedTokens.length === 0) return;

    const success = await importWalletTokens(selectedTokens);
    if (success) {
      setWalletModalOpen(false);
    }
  };

  const handleClose = () => {
    setWalletModalOpen(false);
  };

  const formatValue = (value: string) => {
    const num = parseFloat(value);
    if (isNaN(num)) return '$0.00';
    return `$${num.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  };

  const formatBalance = (balance: string) => {
    const num = parseFloat(balance);
    if (isNaN(num)) return '0';
    if (num >= 1000) return num.toLocaleString(undefined, { maximumFractionDigits: 2 });
    if (num >= 1) return num.toLocaleString(undefined, { maximumFractionDigits: 4 });
    return num.toLocaleString(undefined, { maximumFractionDigits: 8 });
  };

  const selectedCount = tokens.filter(t => t.selected).length;
  const selectedValue = tokens
    .filter(t => t.selected)
    .reduce((sum, t) => sum + parseFloat(t.value_usd || '0'), 0);

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className={`modal ${step === 'results' ? 'modal-wide' : ''}`} onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>
            {step === 'input' && 'Import Wallet'}
            {step === 'scanning' && 'Scanning Wallet'}
            {step === 'results' && 'Scan Results'}
          </h2>
          <button className="modal-close" onClick={handleClose}>×</button>
        </div>

        <div className="modal-form">
          {/* Step 1: Input */}
          {step === 'input' && (
            <>
              <div className="form-group">
                <label>Wallet Address</label>
                <input
                  type="text"
                  value={address}
                  onChange={(e) => setAddress(e.target.value.trim())}
                  placeholder="0x1234...abcd"
                  autoFocus
                  className={error && !isValidAddress(address) ? 'error' : ''}
                  style={{ fontFamily: 'monospace' }}
                />
              </div>

              <div className="form-group">
                <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span>Chains to Scan</span>
                  <button
                    type="button"
                    onClick={toggleAllChains}
                    style={{
                      background: 'none',
                      border: 'none',
                      color: 'var(--primary)',
                      cursor: 'pointer',
                      fontSize: '0.75rem',
                      padding: 0,
                    }}
                  >
                    {selectedChains.length === EVM_CHAINS.length ? 'Deselect All' : 'Select All'}
                  </button>
                </label>
                <div className="wallet-chain-grid" style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(2, 1fr)',
                  gap: '0.5rem',
                  marginTop: '0.5rem',
                }}>
                  {EVM_CHAINS.map(chain => (
                    <label
                      key={chain}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '0.5rem',
                        cursor: 'pointer',
                        padding: '0.4rem 0.6rem',
                        borderRadius: '0.375rem',
                        background: selectedChains.includes(chain)
                          ? 'rgba(217, 253, 101, 0.1)'
                          : 'rgba(255, 255, 255, 0.03)',
                        border: `1px solid ${selectedChains.includes(chain) ? 'rgba(217, 253, 101, 0.3)' : 'var(--border-color)'}`,
                        fontSize: '0.8rem',
                        transition: 'all 0.15s ease',
                      }}
                    >
                      <input
                        type="checkbox"
                        checked={selectedChains.includes(chain)}
                        onChange={() => toggleChain(chain)}
                        style={{ accentColor: 'var(--primary)' }}
                      />
                      {chain}
                    </label>
                  ))}
                </div>
              </div>

              {error && (
                <div style={{
                  color: 'var(--negative)',
                  fontSize: '0.8rem',
                  padding: '0.5rem 0.75rem',
                  background: 'rgba(255, 75, 75, 0.1)',
                  borderRadius: '0.375rem',
                  marginBottom: '1rem',
                }}>
                  {error}
                </div>
              )}

              <div className="form-actions">
                <button type="button" className="btn btn-secondary" onClick={handleClose}>
                  Cancel
                </button>
                <button
                  type="button"
                  className="btn btn-primary"
                  onClick={handleScan}
                  disabled={!address || selectedChains.length === 0}
                >
                  Scan Wallet
                </button>
              </div>
            </>
          )}

          {/* Step 2: Scanning */}
          {step === 'scanning' && (
            <div style={{ textAlign: 'center', padding: '2rem 1rem' }}>
              <div className="loading-spinner" style={{ margin: '0 auto 1rem' }} />
              <p style={{ color: 'var(--text-secondary)', fontSize: '0.9rem' }}>
                {scanningChain}
              </p>
              <p style={{ color: 'var(--text-muted)', fontSize: '0.75rem', marginTop: '0.5rem' }}>
                This may take a moment...
              </p>
            </div>
          )}

          {/* Step 3: Results */}
          {step === 'results' && scanResult && (
            <>
              <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                flexWrap: 'wrap',
                gap: '0.25rem',
                marginBottom: '0.75rem',
                fontSize: '0.8rem',
                color: 'var(--text-secondary)',
              }}>
                <span>
                  Found {tokens.length} token{tokens.length !== 1 ? 's' : ''}
                  {scanResult.dust_filtered > 0 && !showDust && (
                    <> ({scanResult.dust_filtered} small filtered)</>
                  )}
                </span>
                <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                  {scanResult.dust_filtered > 0 && (
                    <button
                      type="button"
                      onClick={() => setShowDust(!showDust)}
                      style={{
                        background: 'none',
                        border: 'none',
                        color: 'var(--primary)',
                        cursor: 'pointer',
                        fontSize: '0.75rem',
                        padding: 0,
                      }}
                    >
                      {showDust ? 'Hide dust' : 'Show all'}
                    </button>
                  )}
                  <button
                    type="button"
                    onClick={toggleAllTokens}
                    style={{
                      background: 'none',
                      border: 'none',
                      color: 'var(--primary)',
                      cursor: 'pointer',
                      fontSize: '0.75rem',
                      padding: 0,
                    }}
                  >
                    {tokens.every(t => t.selected) ? 'Deselect All' : 'Select All'}
                  </button>
                </div>
              </div>

              <div style={{
                maxHeight: '350px',
                overflowY: 'auto',
                border: '1px solid var(--border-color)',
                borderRadius: '0.5rem',
              }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.8rem' }}>
                  <thead>
                    <tr style={{
                      borderBottom: '1px solid var(--border-color)',
                      position: 'sticky',
                      top: 0,
                      background: 'var(--background-dark)',
                      zIndex: 1,
                    }}>
                      <th style={{ padding: '0.5rem', textAlign: 'left', width: '30px' }}></th>
                      <th style={{ padding: '0.5rem', textAlign: 'left' }}>Token</th>
                      <th className="wallet-col-hide-mobile" style={{ padding: '0.5rem', textAlign: 'left' }}>Chain</th>
                      <th className="wallet-col-hide-mobile" style={{ padding: '0.5rem', textAlign: 'right' }}>Balance</th>
                      <th style={{ padding: '0.5rem', textAlign: 'right' }}>Value</th>
                    </tr>
                  </thead>
                  <tbody>
                    {tokens.map((token, i) => (
                      <tr
                        key={`${token.symbol}-${token.chain}-${i}`}
                        onClick={() => toggleToken(i)}
                        style={{
                          borderBottom: '1px solid var(--border-color-light)',
                          cursor: 'pointer',
                          opacity: token.selected ? 1 : 0.4,
                          transition: 'opacity 0.15s ease',
                        }}
                      >
                        <td style={{ padding: '0.4rem 0.5rem' }}>
                          <input
                            type="checkbox"
                            checked={token.selected ?? true}
                            onChange={() => toggleToken(i)}
                            onClick={e => e.stopPropagation()}
                            style={{ accentColor: 'var(--primary)' }}
                          />
                        </td>
                        <td style={{ padding: '0.4rem 0.5rem' }}>
                          <div style={{ fontWeight: 600 }}>{token.symbol}</div>
                          <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
                            {token.name}
                            <span className="wallet-chain-inline"> · {token.chain}</span>
                          </div>
                        </td>
                        <td className="wallet-col-hide-mobile" style={{ padding: '0.4rem 0.5rem', color: 'var(--text-secondary)' }}>
                          {token.chain}
                        </td>
                        <td className="wallet-col-hide-mobile" style={{ padding: '0.4rem 0.5rem', textAlign: 'right', fontFamily: 'monospace' }}>
                          {formatBalance(token.balance)}
                        </td>
                        <td style={{ padding: '0.4rem 0.5rem', textAlign: 'right', fontWeight: 600 }}>
                          {formatValue(token.value_usd)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginTop: '0.75rem',
                fontSize: '0.8rem',
                color: 'var(--text-secondary)',
              }}>
                <span>
                  {selectedCount} selected
                </span>
                <span style={{ fontWeight: 600, color: 'white' }}>
                  Total: {formatValue(selectedValue.toString())}
                </span>
              </div>

              <div className="form-actions" style={{ marginTop: '1rem' }}>
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => { setStep('input'); setError(null); }}
                >
                  Back
                </button>
                <button
                  type="button"
                  className="btn btn-primary"
                  onClick={handleImport}
                  disabled={selectedCount === 0 || isLoading}
                >
                  {isLoading ? 'Importing...' : `Import ${selectedCount} Token${selectedCount !== 1 ? 's' : ''}`}
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

export default WalletImportModal;
