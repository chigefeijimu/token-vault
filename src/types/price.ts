export interface TokenPrice {
  symbol: string
  name: string
  price: number
  priceChange24h?: number
  marketCap?: number
  volume24h?: number
  lastUpdated: number
  imageUrl?: string
  contractAddress?: string
  chainId?: number
}

export interface PriceData {
  [symbol: string]: TokenPrice
}

export interface CoinGeckoMarketData {
  id: string
  symbol: string
  name: string
  current_price: number
  price_change_percentage_24h?: number
  market_cap?: number
  total_volume?: number
  image?: string
  last_updated?: string
}

export interface CoinGeckoPriceResponse {
  [id: string]: {
    usd: number
    usd_24h_change?: number
    usd_market_cap?: number
    usd_24h_vol?: number
    last_updated_at?: number
  }
}

export const COINGECKO_IDS: Record<number, string> = {
  1:     "ethereum",
  56:    "binancecoin",
  137:   "matic-network",
  42161: "ethereum",
  10:    "ethereum",
  43114: "avalanche-2",
}

export const CHAIN_NATIVE_TOKENS: Array<{ chainId: number; symbol: string; name: string; coingeckoId: string }> = [
  { chainId: 1,     symbol: "ETH",  name: "Ethereum",     coingeckoId: "ethereum" },
  { chainId: 56,    symbol: "BNB",  name: "BNB",          coingeckoId: "binancecoin" },
  { chainId: 137,  symbol: "MATIC",name: "Polygon",      coingeckoId: "matic-network" },
  { chainId: 42161, symbol: "ETH", name: "Arbitrum",      coingeckoId: "ethereum" },
  { chainId: 10,   symbol: "ETH",  name: "Optimism",     coingeckoId: "ethereum" },
  { chainId: 43114, symbol: "AVAX", name: "Avalanche",    coingeckoId: "avalanche-2" },
]

