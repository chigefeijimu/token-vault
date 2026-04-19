import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export interface PendingTx {
  hash: string
  status: 'pending' | 'confirmed' | 'failed'
  chainId: number
  timestamp: number
  from: string
  to: string
  amount: string
}

interface PendingTxStore {
  pendingTxs: Record<string, PendingTx>

  addPendingTx: (hash: string, chainId: number, from: string, to: string, amount: string) => void
  updateTxStatus: (hash: string, status: 'pending' | 'confirmed' | 'failed') => void
  removePendingTx: (hash: string) => void
  getPendingTxs: () => PendingTx[]
  getPendingTxsByChain: (chainId: number) => PendingTx[]
  clearResolved: () => void
}

export const usePendingTxStore = create<PendingTxStore>()(
  persist(
    (set, get) => ({
      pendingTxs: {},

      addPendingTx: (hash, chainId, from, to, amount) => {
        set((state) => ({
          pendingTxs: {
            ...state.pendingTxs,
            [hash]: {
              hash,
              status: 'pending',
              chainId,
              timestamp: Date.now(),
              from,
              to,
              amount,
            },
          },
        }))
      },

      updateTxStatus: (hash, status) => {
        set((state) => {
          const tx = state.pendingTxs[hash]
          if (!tx) return state
          return {
            pendingTxs: {
              ...state.pendingTxs,
              [hash]: { ...tx, status },
            },
          }
        })
      },

      removePendingTx: (hash) => {
        set((state) => {
          const { [hash]: _, ...rest } = state.pendingTxs
          return { pendingTxs: rest }
        })
      },

      getPendingTxs: () => {
        return Object.values(get().pendingTxs)
      },

      getPendingTxsByChain: (chainId) => {
        return Object.values(get().pendingTxs).filter((tx) => tx.chainId === chainId)
      },

      clearResolved: () => {
        const now = Date.now()
        const SIXTY_SECONDS = 60_000
        set((state) => {
          const next: Record<string, PendingTx> = {}
          for (const [hash, tx] of Object.entries(state.pendingTxs)) {
            if (tx.status === 'pending') {
              next[hash] = tx
            } else if (now - tx.timestamp < SIXTY_SECONDS) {
              // Keep confirmed/failed txs for 60 seconds before pruning
              next[hash] = tx
            }
          }
          return { pendingTxs: next }
        })
      },
    }),
    {
      name: 'token-vault-pending-txs',
    }
  )
)
