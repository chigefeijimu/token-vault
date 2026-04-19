import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import type { Wallet, Chain, Transaction, DAppConnection, BalanceInfo } from '../types/wallet';

// Default supported chains
const DEFAULT_CHAINS: Chain[] = [
  {
    id: 1,
    name: 'Ethereum',
    symbol: 'ETH',
    rpcUrl: 'https://eth.llamarpc.com',
    blockExplorer: 'https://etherscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    isSupported: true,
  },
  {
    id: 56,
    name: 'BNB Chain',
    symbol: 'BNB',
    rpcUrl: 'https://bsc-rpc.publicnode.com',
    blockExplorer: 'https://bscscan.com',
    nativeCurrency: { name: 'BNB', symbol: 'BNB', decimals: 18 },
    isSupported: true,
  },
  {
    id: 137,
    name: 'Polygon',
    symbol: 'MATIC',
    rpcUrl: 'https://polygon-rpc.com',
    blockExplorer: 'https://polygonscan.com',
    nativeCurrency: { name: 'MATIC', symbol: 'MATIC', decimals: 18 },
    isSupported: true,
  },
  {
    id: 42161,
    name: 'Arbitrum',
    symbol: 'ETH',
    rpcUrl: 'https://arb1.arbitrum.io/rpc',
    blockExplorer: 'https://arbiscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    isSupported: true,
  },
  {
    id: 10,
    name: 'Optimism',
    symbol: 'ETH',
    rpcUrl: 'https://mainnet.optimism.io',
    blockExplorer: 'https://optimistic.etherscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    isSupported: true,
  },
  {
    id: 43114,
    name: 'Avalanche',
    symbol: 'AVAX',
    rpcUrl: 'https://api.avax.network/ext/bc/C/rpc',
    blockExplorer: 'https://snowtrace.io',
    nativeCurrency: { name: 'Avalanche', symbol: 'AVAX', decimals: 18 },
    isSupported: true,
  },
];

interface WalletState {
  // Wallet state
  wallets: Wallet[];
  activeWallet: Wallet | null;
  isUnlocked: boolean;
  isLoading: boolean;
  
  // Chain state
  chains: Chain[];
  activeChain: Chain;
  
  // Balance state
  balances: Map<string, BalanceInfo>; // key: `${address}-${chainId}`
  
  // Transaction state
  pendingTransactions: Transaction[];
  transactionHistory: Transaction[];
  
  // DApp connections
  dappConnections: DAppConnection[];
  
  // Actions
  addWallet: (wallet: Wallet) => void;
  setActiveWallet: (walletId: string) => void;
  removeWallet: (walletId: string) => void;
  setUnlocked: (unlocked: boolean) => void;
  setIsLoading: (loading: boolean) => void;
  setActiveChain: (chain: Chain) => void;
  setBalance: (address: string, chainId: number, balance: BalanceInfo) => void;
  addPendingTransaction: (tx: Transaction) => void;
  updateTransactionStatus: (hash: string, status: Transaction['status']) => void;
  addDappConnection: (conn: DAppConnection) => void;
  removeDappConnection: (dappUrl: string) => void;
  fetchBalance: (address: string, chainId: number) => Promise<BalanceInfo | null>;
  loadWalletsFromBackend: () => Promise<void>;
  
  // Transaction history methods
  addToTransactionHistory: (tx: Transaction) => void;
  getTransactionHistory: () => Transaction[];
  getWalletTransactions: (address: string) => Transaction[];
  getChainTransactions: (chainId: number) => Transaction[];
  clearTransactionHistory: () => void;
  confirmTransaction: (hash: string) => void;
  failTransaction: (hash: string) => void;
  removePendingTransaction: (hash: string) => void;
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set, get) => ({
      wallets: [],
      activeWallet: null,
      isUnlocked: false,
      isLoading: false,
      chains: DEFAULT_CHAINS,
      activeChain: DEFAULT_CHAINS[0],
      balances: new Map(),
      pendingTransactions: [],
      transactionHistory: [],
      dappConnections: [],
      
      addWallet: (wallet) => set((state) => ({
        wallets: [...state.wallets, wallet],
      })),
      
      setActiveWallet: (walletId) => set((state) => ({
        activeWallet: state.wallets.find(w => w.id === walletId) || null,
      })),
      
      removeWallet: (walletId) => set((state) => ({
        wallets: state.wallets.filter(w => w.id !== walletId),
        activeWallet: state.activeWallet?.id === walletId 
          ? state.wallets.find(w => w.id !== walletId) || null 
          : state.activeWallet,
      })),
      
      setUnlocked: (unlocked) => set({ isUnlocked: unlocked }),

      setIsLoading: (loading) => set({ isLoading: loading }),

      setActiveChain: (chain) => set({ activeChain: chain }),
      
      setBalance: (address, chainId, balance) => set((state) => {
        const newBalances = new Map(state.balances);
        newBalances.set(`${address}-${chainId}`, balance);
        return { balances: newBalances };
      }),
      
      addPendingTransaction: (tx) => set((state) => ({
        pendingTransactions: [...state.pendingTransactions, tx],
      })),
      
      updateTransactionStatus: (hash, status) => set((state) => {
        const pendingIndex = state.pendingTransactions.findIndex(tx => tx.hash === hash);
        if (pendingIndex !== -1) {
          const updatedPending = [...state.pendingTransactions];
          updatedPending[pendingIndex] = { ...updatedPending[pendingIndex], status };
          
          // If confirmed or failed, move to history
          if (status === 'confirmed' || status === 'failed') {
            const tx = updatedPending.splice(pendingIndex, 1)[0];
            return {
              pendingTransactions: updatedPending,
              transactionHistory: [tx, ...state.transactionHistory].slice(0, 100),
            };
          }
          
          return { pendingTransactions: updatedPending };
        }
        
        // Also update in history if present
        const historyIndex = state.transactionHistory.findIndex(tx => tx.hash === hash);
        if (historyIndex !== -1) {
          const updatedHistory = [...state.transactionHistory];
          updatedHistory[historyIndex] = { ...updatedHistory[historyIndex], status };
          return { transactionHistory: updatedHistory };
        }
        
        return state;
      }),
      
      addDappConnection: (conn) => set((state) => {
        const filtered = state.dappConnections.filter(
          c => !(c.dappUrl === conn.dappUrl && c.walletAddress === conn.walletAddress)
        );
        return { dappConnections: [...filtered, conn] };
      }),
      
      removeDappConnection: (dappUrl) => set((state) => ({
        dappConnections: state.dappConnections.filter(c => c.dappUrl !== dappUrl),
      })),
      
      fetchBalance: async (address, chainId) => {
        try {
          const balance = await invoke<BalanceInfo>('get_balance', {
            address,
            chainId,
          });
          get().setBalance(address, chainId, balance);
          return balance;
        } catch (error) {
          console.error('Failed to fetch balance:', error);
          return null;
        }
      },
      
      loadWalletsFromBackend: async () => {
        try {
          set({ isLoading: true });
          const backendWallets = await invoke<Wallet[]>('get_wallets');
          const wallets = backendWallets.map((w: any) => ({
            id: w.id,
            name: w.name,
            address: w.address,
            createdAt: w.created_at,
            isActive: false,
          }));
          set({ wallets });
        } catch (error) {
          console.error('Failed to load wallets from backend:', error);
        } finally {
          set({ isLoading: false });
        }
      },
      
      // Transaction history methods
      addToTransactionHistory: (tx) => set((state) => ({
        transactionHistory: [tx, ...state.transactionHistory].slice(0, 100),
      })),
      
      getTransactionHistory: () => {
        return get().transactionHistory;
      },
      
      getWalletTransactions: (address) => {
        return get().transactionHistory.filter(
          tx => tx.from.toLowerCase() === address.toLowerCase() || 
                tx.to.toLowerCase() === address.toLowerCase()
        );
      },
      
      getChainTransactions: (chainId) => {
        return get().transactionHistory.filter(tx => tx.chainId === chainId);
      },
      
      clearTransactionHistory: () => set({ transactionHistory: [] }),
      
      confirmTransaction: (hash) => {
        const state = get();
        const pendingTx = state.pendingTransactions.find(tx => tx.hash === hash);
        if (pendingTx) {
          const confirmedTx = { ...pendingTx, status: 'confirmed' as const };
          set({
            pendingTransactions: state.pendingTransactions.filter(tx => tx.hash !== hash),
            transactionHistory: [confirmedTx, ...state.transactionHistory].slice(0, 100),
          });
        } else {
          // Update in history if already there
          set({
            transactionHistory: state.transactionHistory.map(tx =>
              tx.hash === hash ? { ...tx, status: 'confirmed' as const } : tx
            ),
          });
        }
      },
      
      failTransaction: (hash) => {
        const state = get();
        const pendingTx = state.pendingTransactions.find(tx => tx.hash === hash);
        if (pendingTx) {
          const failedTx = { ...pendingTx, status: 'failed' as const };
          set({
            pendingTransactions: state.pendingTransactions.filter(tx => tx.hash !== hash),
            transactionHistory: [failedTx, ...state.transactionHistory].slice(0, 100),
          });
        } else {
          set({
            transactionHistory: state.transactionHistory.map(tx =>
              tx.hash === hash ? { ...tx, status: 'failed' as const } : tx
            ),
          });
        }
      },
      
      removePendingTransaction: (hash) => set((state) => ({
        pendingTransactions: state.pendingTransactions.filter(tx => tx.hash !== hash),
      })),
    }),
    {
      name: 'token-vault-wallet',
      partialize: (state) => ({
        wallets: state.wallets,
        activeWallet: state.activeWallet,
        chains: state.chains,
        activeChain: state.activeChain,
        transactionHistory: state.transactionHistory,
        dappConnections: state.dappConnections,
        // Don't persist pending transactions as they should be re-fetched
      }),
    }
  )
);
