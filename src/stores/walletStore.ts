import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { WalletData, BalanceInfo, Transaction } from '../types/wallet'

declare global {
  interface Window {
    __TAURI__: typeof import('@tauri-apps/api')
  }
}

interface WalletStore {
  wallets: WalletData[]
  selectedChain: number
  balances: Record<string, Record<number, string>>
  transactions: Record<string, Transaction[]>
  isLoading: boolean

  setSelectedChain: (chainId: number) => void
  createWallet: (name: string, password: string) => Promise<{ success: boolean; mnemonic?: string; error?: string }>
  importWallet: (name: string, password: string, value: string, type: 'mnemonic' | 'privateKey') => Promise<{ success: boolean; error?: string }>
  deleteWallet: (id: string) => Promise<void>
  getWallet: (id: string) => WalletData | undefined
  fetchBalance: (address: string) => Promise<void>
  sendTransaction: (walletId: string, to: string, amount: string, chainId: number) => Promise<{ success: boolean; txHash?: string; error?: string }>
  exportPrivateKey: (id: string, password: string) => Promise<string | null>
}

export const useWalletStore = create<WalletStore>()(
  persist(
    (set, get) => ({
      wallets: [],
      selectedChain: 1,
      balances: {},
      transactions: {},
      isLoading: false,

      setSelectedChain: (chainId) => set({ selectedChain: chainId }),

      createWallet: async (name, password) => {
        try {
          const result = await window.__TAURI__.core.invoke<{ id: string; mnemonic: string }>('create_wallet', {
            name,
            password,
          })
          const walletData: WalletData = {
            id: result.id,
            name,
            address: '',
            createdAt: Date.now(),
            encryptedMnemonic: undefined,
          }
          const wallets = get().wallets
          set({ wallets: [...wallets, walletData] })
          return { success: true, mnemonic: result.mnemonic }
        } catch (e) {
          return { success: false, error: String(e) }
        }
      },

      importWallet: async (name, password, value, type) => {
        try {
          const result = await window.__TAURI__.core.invoke<{ id: string }>('import_wallet', {
            name,
            password,
            mnemonic: type === 'mnemonic' ? value : null,
            privateKey: type === 'privateKey' ? value : null,
          })
          const walletData: WalletData = {
            id: result.id,
            name,
            address: '',
            createdAt: Date.now(),
          }
          const wallets = get().wallets
          set({ wallets: [...wallets, walletData] })
          return { success: true }
        } catch (e) {
          return { success: false, error: String(e) }
        }
      },

      deleteWallet: async (id) => {
        await window.__TAURI__.core.invoke('delete_wallet', { id })
        const wallets = get().wallets.filter(w => w.id !== id)
        set({ wallets })
      },

      getWallet: (id) => {
        return get().wallets.find(w => w.id === id)
      },

      fetchBalance: async (address) => {
        const chainId = get().selectedChain
        try {
          const info = await window.__TAURI__.core.invoke<BalanceInfo>('get_balance', {
            address,
            chainId,
          })
          set(state => ({
            balances: {
              ...state.balances,
              [address]: {
                ...state.balances[address],
                [chainId]: info.balanceFormatted,
              },
            },
          }))
        } catch (e) {
          console.error('Failed to fetch balance:', e)
        }
      },

      sendTransaction: async (walletId, to, amount, chainId) => {
        try {
          set({ isLoading: true })
          const result = await window.__TAURI__.core.invoke<{ txHash: string }>('send_transaction', {
            walletId,
            to,
            amount,
            chainId,
          })
          set({ isLoading: false })
          return { success: true, txHash: result.txHash }
        } catch (e) {
          set({ isLoading: false })
          return { success: false, error: String(e) }
        }
      },

      exportPrivateKey: async (id, password) => {
        try {
          const result = await window.__TAURI__.core.invoke<string>('export_private_key', {
            id,
            password,
          })
          return result
        } catch {
          return null
        }
      },
    }),
    {
      name: 'token-vault-storage',
      partialize: (state) => ({
        wallets: state.wallets,
        selectedChain: state.selectedChain,
      }),
    }
  )
)
