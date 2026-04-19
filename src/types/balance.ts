export interface TokenBalance {
  address: string
  symbol: string
  name: string
  decimals: number
  balance: string
  balanceFormatted: string
  logoUrl?: string
}

export interface NativeBalance {
  symbol: string
  balance: string
  balanceFormatted: string
  decimals: number
}

export interface WalletBalances {
  address: string
  chainId: number
  nativeBalance: NativeBalance
  erc20Balances: TokenBalance[]
  totalUsdValue?: number
}