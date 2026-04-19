import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import type { Wallet, Chain, Transaction, DAppConnection, BalanceInfo, TokenBalance } from '../types/wallet';

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
  erc20Balances: Map<string, TokenBalance[]>; // key: `${address}-${chainId}`
  
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
  setErc20Balances: (address: string, chainId: number, tokens: TokenBalance[]) => void;
  addPendingTransaction: (tx: Transaction) => void;
  updateTransactionStatus: (hash: string, status: Transaction['status']) => void;
  addDappConnection: (conn: DAppConnection) => void;
  removeDappConnection: (dappUrl: string) => void;
  fetchBalance: (address: string, chainId: number) => Promise<BalanceInfo | null>;
  fetchErc20Balances: (address: string, chainId: number, tokenAddresses: string[]) => Promise<TokenBalance[]>;
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
      erc20Balances: new Map(),
      pendingTransactions: [],
      transactionHistory: [],
      dappConnections: [],
      
      addWallet: (wallet) => set((state) => ({
        wallets: [...state.wallets, wallet],
      })),
      
      setActiveWallet: (walletId) => set((state) => ({
        activeWallet: state.wallets.find(w => w.id === walletId) || null,
      })),
      
      removeWallet: (walletId) => set((state) => {
        const newWallets = state.wallets.filter(w => w.id !== walletId);
        return {
          wallets: newWallets,
          activeWallet: state.activeWallet?.id === walletId ? (newWallets[0] || null) : state.activeWallet,
        };
      }),
      
      setUnlocked: (unlocked) => set({ isUnlocked: unlocked }),
      
      setIsLoading: (loading) => set({ isLoading: loading }),
      
      setActiveChain: (chain) => set({ activeChain: chain }),
      
      setBalance: (address, chainId, balance) => set((state) => {
        const key = `${address}-${chainId}`;
        const newBalances = new Map(state.balances);
        newBalances.set(key, balance);
        return { balances: newBalances };
      }),
      
      setErc20Balances: (address, chainId, tokens) => set((state) => {
        const key = `${address}-${chainId}`;
        const newErc20Balances = new Map(state.erc20Balances);
        newErc20Balances.set(key, tokens);
        return { erc20Balances: newErc20Balances };
      }),
      
      addPendingTransaction: (tx) => set((state) => ({
        pendingTransactions: [...state.pendingTransactions, tx],
      })),
      
      updateTransactionStatus: (hash, status) => set((state) => ({
        pendingTransactions: state.pendingTransactions.map(tx =>
          tx.hash === hash ? { ...tx, status } : tx
        ),
      })),
      
      addDappConnection: (conn) => set((state) => ({
        dappConnections: [...state.dappConnections, conn],
      })),
      
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
      
      fetchErc20Balances: async (address, chainId, tokenAddresses) => {
        try {
          set({ isLoading: true });
          const tokens = await invoke<TokenBalance[]>('get_erc20_balances', {
            address,
            chainId,
            tokenAddresses,
          });
          get().setErc20Balances(address, chainId, tokens);
          return tokens;
        } catch (error) {
          console.error('Failed to fetch ERC20 balances:', error);
          return [];
        } finally {
          set({ isLoading: false });
        }
      },
      
      loadWalletsFromBackend: async () => {
        try {
          set({ isLoading: true });
          const backendWallets = await invoke<{ id: string; name: string; address: string; created_at: string }[]>('get_wallets');
          const wallets: Wallet[] = backendWallets.map(w => ({
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
      
      addToTransactionHistory: (tx) => set((state) => ({
        transactionHistory: [tx, ...state.transactionHistory].slice(0, 100),
      })),
      
      getTransactionHistory: () => get().transactionHistory,
      
      getWalletTransactions: (address) =>
        get().transactionHistory.filter(tx => tx.from.toLowerCase() === address.toLowerCase()),
      
      getChainTransactions: (chainId) =>
        get().transactionHistory.filter(tx => tx.chainId === chainId),
      
      clearTransactionHistory: () => set({ transactionHistory: [] }),
      
      confirmTransaction: (hash) => {
        const state = get();
        const tx = state.pendingTransactions.find(t => t.hash === hash);
        if (tx) {
          state.addToTransactionHistory({ ...tx, status: 'confirmed' });
          state.removePendingTransaction(hash);
        }
      },
      
      failTransaction: (hash) => {
        const state = get();
        const tx = state.pendingTransactions.find(t => t.hash === hash);
        if (tx) {
          state.addToTransactionHistory({ ...tx, status: 'failed' });
          state.removePendingTransaction(hash);
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
      }),
    }
  )
);