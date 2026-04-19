import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { Chain, CHAINS } from '../types/wallet';

export interface CustomChain {
  id: number;
  name: string;
  symbol: string;
  rpcUrl: string;
  explorerUrl: string;
}

interface ChainContextType {
  currentChain: Chain;
  chains: Chain[];
  testChains: Chain[];
  customChains: CustomChain[];
  isCustomRpc: boolean;
  setCurrentChain: (chain: Chain) => void;
  addCustomChain: (chain: CustomChain) => void;
  removeCustomChain: (id: number) => void;
  switchToMainnet: (chainId: number) => void;
  switchToTestnet: (chainId: number) => void;
  setCustomRpc: (rpcUrl: string, chainId: number, name: string, symbol: string, explorerUrl: string) => void;
  clearCustomRpc: () => void;
}

const DEFAULT_TEST_CHAINS: Chain[] = [
  { id: 11155111, name: 'Sepolia', symbol: 'ETH', rpcUrl: 'https://rpc.sepolia.org', explorerUrl: 'https://sepolia.etherscan.io' },
  { id: 80001, name: 'Mumbai', symbol: 'MATIC', rpcUrl: 'https://rpc-mumbai.maticvigil.com', explorerUrl: 'https://mumbai.polygonscan.com' },
  { id: 97, name: 'BSC Testnet', symbol: 'BNB', rpcUrl: 'https://data-seed-prebsc-1-s1.binance.org:8545', explorerUrl: 'https://testnet.bscscan.com' },
  { id: 421613, name: 'Arbitrum Goerli', symbol: 'ETH', rpcUrl: 'https://goerli-rollup.arbitrum.io/rpc', explorerUrl: 'https://goerli.arbiscan.io' },
  { id: 420, name: 'Optimism Goerli', symbol: 'ETH', rpcUrl: 'https://goerli.optimism.io', explorerUrl: 'https://goerli-optimism.etherscan.io' },
  { id: 43113, name: 'Avalanche Fuji', symbol: 'AVAX', rpcUrl: 'https://api.avax-test.network/ext/bc/C/rpc', explorerUrl: 'https://testnet.snowtrace.io' },
];

const ChainContext = createContext<ChainContextType | undefined>(undefined);

export const ChainProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [currentChain, setCurrentChain] = useState<Chain>(CHAINS[0]);
  const [customChains, setCustomChains] = useState<CustomChain[]>([]);
  const [isCustomRpc, setIsCustomRpc] = useState(false);

  const _allChains = [...CHAINS, ...customChains];
  void _allChains; // suppress unused warning

  const switchToMainnet = useCallback((chainId: number) => {
    const chain = CHAINS.find(c => c.id === chainId);
    if (chain) {
      setCurrentChain(chain);
      setIsCustomRpc(false);
    }
  }, []);

  const switchToTestnet = useCallback((chainId: number) => {
    const chain = DEFAULT_TEST_CHAINS.find(c => c.id === chainId);
    if (chain) {
      setCurrentChain(chain);
      setIsCustomRpc(false);
    }
  }, []);

  const addCustomChain = useCallback((chain: CustomChain) => {
    setCustomChains(prev => {
      const exists = prev.some(c => c.id === chain.id);
      if (exists) {
        return prev.map(c => c.id === chain.id ? chain : c);
      }
      return [...prev, chain];
    });
  }, []);

  const removeCustomChain = useCallback((id: number) => {
    setCustomChains(prev => prev.filter(c => c.id !== id));
    setCurrentChain(prev => {
      if (prev.id === id) {
        return CHAINS[0];
      }
      return prev;
    });
  }, []);

  const setCustomRpc = useCallback((rpcUrl: string, chainId: number, name: string, symbol: string, explorerUrl: string) => {
    const customChain: CustomChain = { id: chainId, name, symbol, rpcUrl, explorerUrl };
    addCustomChain(customChain);
    setCurrentChain(customChain as Chain);
    setIsCustomRpc(true);
  }, [addCustomChain]);

  const clearCustomRpc = useCallback(() => {
    setIsCustomRpc(false);
  }, []);

  return (
    <ChainContext.Provider
      value={{
        currentChain,
        chains: CHAINS,
        testChains: DEFAULT_TEST_CHAINS,
        customChains,
        isCustomRpc,
        setCurrentChain,
        addCustomChain,
        removeCustomChain,
        switchToMainnet,
        switchToTestnet,
        setCustomRpc,
        clearCustomRpc,
      }}
    >
      {children}
    </ChainContext.Provider>
  );
};

export const useChainContext = (): ChainContextType => {
  const context = useContext(ChainContext);
  if (!context) {
    throw new Error('useChainContext must be used within a ChainProvider');
  }
  return context;
};