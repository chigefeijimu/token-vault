import React, { useState, useRef, useEffect } from 'react';
import { useChainContext } from '../contexts/ChainContext';
import styles from './NetworkSelector.module.css';

interface NetworkSelectorProps {
  compact?: boolean;
  onNetworkChange?: (chainId: number) => void;
}

export const NetworkSelector: React.FC<NetworkSelectorProps> = ({ compact = false, onNetworkChange }) => {
  const {
    currentChain,
    chains,
    testChains,
    customChains,
    isCustomRpc,
    setCurrentChain,
    switchToMainnet,
    switchToTestnet,
    removeCustomChain,
  } = useChainContext();

  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<'mainnet' | 'testnet' | 'custom'>('mainnet');
  const [customRpcUrl, setCustomRpcUrl] = useState('');
  const [customChainId, setCustomChainId] = useState('');
  const [customName, setCustomName] = useState('');
  const [customSymbol, setCustomSymbol] = useState('');
  const [customExplorer, setCustomExplorer] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleChainSelect = (chain: typeof currentChain) => {
    setCurrentChain(chain);
    setIsOpen(false);
    onNetworkChange?.(chain.id);
  };

  const handleAddCustomRpc = () => {
    if (!customRpcUrl || !customChainId) return;
    
    const chainId = parseInt(customChainId);
    const newChain = {
      id: chainId,
      name: customName || `Custom Network ${chainId}`,
      symbol: customSymbol || 'CUSTOM',
      rpcUrl: customRpcUrl,
      explorerUrl: customExplorer || '',
    };
    setCurrentChain(newChain);
    setIsOpen(false);
    onNetworkChange?.(chainId);
    
    // Reset form
    setCustomRpcUrl('');
    setCustomChainId('');
    setCustomName('');
    setCustomSymbol('');
    setCustomExplorer('');
  };

  const handleRemoveCustom = (e: React.MouseEvent, chainId: number) => {
    e.stopPropagation();
    removeCustomChain(chainId);
  };

  const getNetworkIcon = (chainId: number) => {
    const chain = [...chains, ...testChains, ...customChains].find(c => c.id === chainId);
    if (!chain) return null;
    
    return (
      <div className={styles.chainIcon}>
        <span className={styles.chainSymbol}>{chain.symbol.charAt(0)}</span>
      </div>
    );
  };

  if (compact) {
    return (
      <div className={styles.compactSelector} ref={dropdownRef}>
        <button className={styles.compactButton} onClick={() => setIsOpen(!isOpen)}>
          {getNetworkIcon(currentChain.id)}
          <span>{currentChain.name}</span>
          <span className={`${styles.chainId} ${isCustomRpc ? styles.custom : ''}`}>
            {currentChain.id}
          </span>
          <span className={styles.arrow}>▼</span>
        </button>
        
        {isOpen && (
          <div className={styles.compactDropdown}>
            <div className={styles.compactTabs}>
              <button
                className={`${styles.tabButton} ${activeTab === 'mainnet' ? styles.active : ''}`}
                onClick={() => setActiveTab('mainnet')}
              >
                Mainnet
              </button>
              <button
                className={`${styles.tabButton} ${activeTab === 'testnet' ? styles.active : ''}`}
                onClick={() => setActiveTab('testnet')}
              >
                Testnet
              </button>
              <button
                className={`${styles.tabButton} ${activeTab === 'custom' ? styles.active : ''}`}
                onClick={() => setActiveTab('custom')}
              >
                Custom
              </button>
            </div>
            
            <div className={styles.compactContent}>
              {activeTab === 'mainnet' && (
                <div className={styles.chainList}>
                  {chains.map(chain => (
                    <button
                      key={chain.id}
                      className={`${styles.chainItem} ${currentChain.id === chain.id ? styles.selected : ''}`}
                      onClick={() => {
                        switchToMainnet(chain.id);
                        onNetworkChange?.(chain.id);
                      }}
                    >
                      {getNetworkIcon(chain.id)}
                      <span className={styles.chainName}>{chain.name}</span>
                      <span className={styles.chainIdBadge}>{chain.id}</span>
                    </button>
                  ))}
                </div>
              )}
              
              {activeTab === 'testnet' && (
                <div className={styles.chainList}>
                  {testChains.map(chain => (
                    <button
                      key={chain.id}
                      className={`${styles.chainItem} ${currentChain.id === chain.id ? styles.selected : ''}`}
                      onClick={() => {
                        switchToTestnet(chain.id);
                        onNetworkChange?.(chain.id);
                      }}
                    >
                      {getNetworkIcon(chain.id)}
                      <span className={styles.chainName}>{chain.name}</span>
                      <span className={styles.chainIdBadge}>{chain.id}</span>
                    </button>
                  ))}
                </div>
              )}
              
              {activeTab === 'custom' && (
                <div className={styles.customForm}>
                  {customChains.length > 0 && (
                    <div className={styles.customChainsList}>
                      <p className={styles.sectionLabel}>Saved Custom Networks</p>
                      {customChains.map(chain => (
                        <button
                          key={chain.id}
                          className={`${styles.chainItem} ${currentChain.id === chain.id ? styles.selected : ''}`}
                          onClick={() => handleChainSelect(chain)}
                        >
                          {getNetworkIcon(chain.id)}
                          <span className={styles.chainName}>{chain.name}</span>
                          <span className={styles.chainIdBadge}>{chain.id}</span>
                          <button
                            className={styles.removeButton}
                            onClick={(e) => handleRemoveCustom(e, chain.id)}
                            title="Remove"
                          >
                            ×
                          </button>
                        </button>
                      ))}
                    </div>
                  )}
                  
                  <p className={styles.sectionLabel}>Add Custom RPC</p>
                  <input
                    type="text"
                    placeholder="RPC URL (https://...)"
                    value={customRpcUrl}
                    onChange={(e) => setCustomRpcUrl(e.target.value)}
                    className={styles.input}
                  />
                  <input
                    type="number"
                    placeholder="Chain ID"
                    value={customChainId}
                    onChange={(e) => setCustomChainId(e.target.value)}
                    className={styles.input}
                  />
                  <input
                    type="text"
                    placeholder="Network Name (optional)"
                    value={customName}
                    onChange={(e) => setCustomName(e.target.value)}
                    className={styles.input}
                  />
                  <input
                    type="text"
                    placeholder="Symbol (optional)"
                    value={customSymbol}
                    onChange={(e) => setCustomSymbol(e.target.value)}
                    className={styles.input}
                  />
                  <input
                    type="text"
                    placeholder="Explorer URL (optional)"
                    value={customExplorer}
                    onChange={(e) => setCustomExplorer(e.target.value)}
                    className={styles.input}
                  />
                  <button
                    className={styles.addButton}
                    onClick={handleAddCustomRpc}
                    disabled={!customRpcUrl || !customChainId}
                  >
                    Add Network
                  </button>
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className={styles.container} ref={dropdownRef}>
      <div className={styles.header}>
        <h3 className={styles.title}>Network</h3>
        <div className={styles.currentNetwork}>
          {getNetworkIcon(currentChain.id)}
          <span>{currentChain.name}</span>
          {isCustomRpc && <span className={styles.customBadge}>Custom</span>}
        </div>
      </div>
      
      <div className={styles.tabs}>
        <button
          className={`${styles.tab} ${activeTab === 'mainnet' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('mainnet')}
        >
          Mainnet
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'testnet' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('testnet')}
        >
          Testnet
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'custom' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('custom')}
        >
          Custom RPC
        </button>
      </div>
      
      <div className={styles.content}>
        {activeTab === 'mainnet' && (
          <div className={styles.networkGrid}>
            {chains.map(chain => (
              <button
                key={chain.id}
                className={`${styles.networkCard} ${currentChain.id === chain.id ? styles.active : ''}`}
                onClick={() => handleChainSelect(chain)}
              >
                <div className={styles.networkIconLarge}>
                  <span>{chain.symbol.charAt(0)}</span>
                </div>
                <span className={styles.networkName}>{chain.name}</span>
                <span className={styles.networkSymbol}>{chain.symbol}</span>
                <span className={styles.networkId}>Chain ID: {chain.id}</span>
              </button>
            ))}
          </div>
        )}
        
        {activeTab === 'testnet' && (
          <div className={styles.networkGrid}>
            {testChains.map(chain => (
              <button
                key={chain.id}
                className={`${styles.networkCard} ${currentChain.id === chain.id ? styles.active : ''}`}
                onClick={() => handleChainSelect(chain)}
              >
                <div className={`${styles.networkIconLarge} ${styles.testnet}`}>
                  <span>{chain.symbol.charAt(0)}</span>
                </div>
                <span className={styles.networkName}>{chain.name}</span>
                <span className={styles.networkSymbol}>{chain.symbol}</span>
                <span className={styles.networkId}>Chain ID: {chain.id}</span>
              </button>
            ))}
          </div>
        )}
        
        {activeTab === 'custom' && (
          <div className={styles.customSection}>
            {customChains.length > 0 && (
              <div className={styles.savedNetworks}>
                <h4>Saved Custom Networks</h4>
                <div className={styles.networkList}>
                  {customChains.map(chain => (
                    <button
                      key={chain.id}
                      className={`${styles.customNetworkItem} ${currentChain.id === chain.id ? styles.active : ''}`}
                      onClick={() => handleChainSelect(chain)}
                    >
                      <span className={styles.customNetworkName}>{chain.name}</span>
                      <span className={styles.customNetworkId}>ID: {chain.id}</span>
                      <button
                        className={styles.deleteButton}
                        onClick={(e) => handleRemoveCustom(e, chain.id)}
                      >
                        Delete
                      </button>
                    </button>
                  ))}
                </div>
              </div>
            )}
            
            <div className={styles.customFormSection}>
              <h4>Add Custom RPC Endpoint</h4>
              <div className={styles.formGroup}>
                <label>RPC URL *</label>
                <input
                  type="url"
                  placeholder="https://your-rpc-endpoint.com"
                  value={customRpcUrl}
                  onChange={(e) => setCustomRpcUrl(e.target.value)}
                  className={styles.input}
                />
              </div>
              <div className={styles.formGroup}>
                <label>Chain ID *</label>
                <input
                  type="number"
                  placeholder="1"
                  value={customChainId}
                  onChange={(e) => setCustomChainId(e.target.value)}
                  className={styles.input}
                />
              </div>
              <div className={styles.formRow}>
                <div className={styles.formGroup}>
                  <label>Network Name</label>
                  <input
                    type="text"
                    placeholder="My Custom Network"
                    value={customName}
                    onChange={(e) => setCustomName(e.target.value)}
                    className={styles.input}
                  />
                </div>
                <div className={styles.formGroup}>
                  <label>Symbol</label>
                  <input
                    type="text"
                    placeholder="ETH"
                    value={customSymbol}
                    onChange={(e) => setCustomSymbol(e.target.value)}
                    className={styles.input}
                  />
                </div>
              </div>
              <div className={styles.formGroup}>
                <label>Block Explorer URL</label>
                <input
                  type="url"
                  placeholder="https://explorer.example.com"
                  value={customExplorer}
                  onChange={(e) => setCustomExplorer(e.target.value)}
                  className={styles.input}
                />
              </div>
              <button
                className={styles.submitButton}
                onClick={handleAddCustomRpc}
                disabled={!customRpcUrl || !customChainId}
              >
                Add Custom Network
              </button>
            </div>
          </div>
        )}
      </div>
      
      <div className={styles.footer}>
        <span>Current: {currentChain.name} (ID: {currentChain.id})</span>
      </div>
    </div>
  );
};

export default NetworkSelector;