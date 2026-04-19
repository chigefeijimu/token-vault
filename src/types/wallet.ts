export interface Chain {
  id: number
  name: string
  symbol: string
  rpcUrl: string
  explorerUrl: string
}

export const CHAINS: Chain[] = [
  { id: 1, name: 'Ethereum', symbol: 'ETH', rpcUrl: 'https://eth.llamarpc.com', explorerUrl: 'https://etherscan.io' },
  { id: 56, name: 'BNB Chain', symbol: 'BNB', rpcUrl: 'https://bsc-dataseed.binance.org', explorerUrl: 'https://bscscan.com' },
  { id: 137, name: 'Polygon', symbol: 'MATIC', rpcUrl: 'https://polygon-rpc.com', explorerUrl: 'https://polygonscan.com' },
  { id: 42161, name: 'Arbitrum', symbol: 'ETH', rpcUrl: 'https://arb1.arbitrum.io/rpc', explorerUrl: 'https://arbiscan.io' },
  { id: 10, name: 'Optimism', symbol: 'ETH', rpcUrl: 'https://mainnet.optimism.io', explorerUrl: 'https://optimistic.etherscan.io' },
  { id: 43114, name: 'Avalanche', symbol: 'AVAX', rpcUrl: 'https://api.avax.network/ext/bc/C/rpc', explorerUrl: 'https://snowtrace.io' },
]

export interface ChainConfig {
  id: number
  name: string
  symbol: string
  rpcUrl: string
  explorerUrl: string
  decimals: number
  type: 'evm'
}

export interface BalanceInfo {
  balance: string
  balanceFormatted: string
  symbol: string
  decimals: number
}

export interface Transaction {
  hash: string
  from: string
  to: string
  value: string
  timestamp: number
  blockNumber: string
  blockHash: string
  chainId: number
  status: 'pending' | 'confirmed' | 'failed'
}

export interface WalletData {
  id: string
  name: string
  address: string
  createdAt: number
  encryptedMnemonic?: string
  encryptedPrivateKey?: string
}

// Saved address types
export interface SavedAddress {
  id: string
  name: string
  address: string
  chainId: number
  isFavorite: boolean
  createdAt: number
  updatedAt: number
}

export interface SavedAddressInput {
  name: string
  address: string
  chainId: number
  isFavorite?: boolean
}

export interface SavedAddressUpdate {
  id: string
  name?: string
  address?: string
  chainId?: number
  isFavorite?: boolean
}

export interface AddressValidationResult {
  isValid: boolean
  address: string
  chainId?: number
  error?: string
}

// Custom ERC20 token
export interface CustomToken {
  address: string
  name: string
  symbol: string
  decimals: number
  chainId: number
  logoUrl?: string
}
