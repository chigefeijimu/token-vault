import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { TransactionRecord } from "../types/transaction";

interface CachedHistory {
  transactions: TransactionRecord[];
  fetchedAt: number;
}

interface TransactionStore {
  historyCache: Record<string, CachedHistory>;
  pendingTransactions: TransactionRecord[];
  isFetching: boolean;
  error: string | null;

  addPendingTransaction: (tx: TransactionRecord) => void;
  updateTransactionStatus: (hash: string, status: TransactionRecord["status"]) => void;
  setHistory: (address: string, chainId: number, transactions: TransactionRecord[]) => void;
  appendTransactions: (address: string, chainId: number, transactions: TransactionRecord[]) => void;
  setFetching: (loading: boolean) => void;
  setError: (error: string | null) => void;
  getCacheKey: (address: string, chainId: number) => string;
  getHistory: (address: string, chainId: number) => TransactionRecord[] | null;
  clearHistory: (address: string, chainId: number) => void;
}

export const useTransactionStore = create<TransactionStore>()(
  persist(
    (set, get) => ({
      historyCache: {},
      pendingTransactions: [],
      isFetching: false,
      error: null,

      getCacheKey: (address: string, chainId: number) => {
        return `${address.toLowerCase()}-${chainId}`;
      },

      addPendingTransaction: (tx: TransactionRecord) => {
        set((state) => ({
          pendingTransactions: [
            ...state.pendingTransactions.filter((t) => t.hash !== tx.hash),
            tx,
          ],
        }));
      },

      updateTransactionStatus: (hash: string, status: TransactionRecord["status"]) => {
        set((state) => ({
          pendingTransactions: state.pendingTransactions.map((t) =>
            t.hash === hash ? { ...t, status } : t
          ),
          historyCache: Object.fromEntries(
            Object.entries(state.historyCache).map(([key, val]) => [
              key,
              {
                ...val,
                transactions: val.transactions.map((t) =>
                  t.hash === hash ? { ...t, status } : t
                ),
              },
            ])
          ),
        }));
      },

      setHistory: (address: string, chainId: number, transactions: TransactionRecord[]) => {
        const key = get().getCacheKey(address, chainId);
        set((state) => ({
          historyCache: {
            ...state.historyCache,
            [key]: { transactions, fetchedAt: Date.now() },
          },
        }));
      },

      appendTransactions: (address: string, chainId: number, transactions: TransactionRecord[]) => {
        const key = get().getCacheKey(address, chainId);
        set((state) => {
          const existing = state.historyCache[key]?.transactions ?? [];
          const seen = new Set(existing.map((t) => t.hash));
          const merged = [...existing, ...transactions.filter((t) => !seen.has(t.hash))];
          return {
            historyCache: {
              ...state.historyCache,
              [key]: { transactions: merged, fetchedAt: Date.now() },
            },
          };
        });
      },

      setFetching: (loading: boolean) => set({ isFetching: loading }),
      setError: (error: string | null) => set({ error }),

      getHistory: (address: string, chainId: number) => {
        const key = get().getCacheKey(address, chainId);
        return get().historyCache[key]?.transactions ?? null;
      },

      clearHistory: (address: string, chainId: number) => {
        const key = get().getCacheKey(address, chainId);
        set((state) => {
          const { [key]: _, ...rest } = state.historyCache;
          return { historyCache: rest };
        });
      },
    }),
    {
      name: "token-vault-transactions",
      partialize: (state) => ({
        historyCache: state.historyCache,
        pendingTransactions: state.pendingTransactions,
      }),
    }
  )
);
