import { useState, useCallback, useEffect, useRef } from "react"
import type { TokenPrice, CoinGeckoPriceResponse } from "../types/price"
import { COINGECKO_IDS, CHAIN_NATIVE_TOKENS } from "../types/price"

const COINGECKO_API = "https://api.coingecko.com/api/v3"
const REFRESH_INTERVAL_MS = 60_000 // 1 minute

interface UseTokenPriceResult {
  prices: Record<number, TokenPrice>
  isLoading: boolean
  error: string | null
  refresh: () => Promise<void>
  getPrice: (chainId: number) => TokenPrice | null
}

interface UseTokenPriceOptions {
  autoRefresh?: boolean
  refreshInterval?: number
}

export function useTokenPrice(
  chainIds?: number[],
  options: UseTokenPriceOptions = {}
): UseTokenPriceResult {
  const { autoRefresh = true, refreshInterval = REFRESH_INTERVAL_MS } = options
  const [prices, setPrices] = useState<Record<number, TokenPrice>>({})
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const targetChains = chainIds ?? CHAIN_NATIVE_TOKENS.map(t => t.chainId)

  const fetchPrices = useCallback(async () => {
    const ids = targetChains
      .map(cid => COINGECKO_IDS[cid])
      .filter(Boolean)

    if (ids.length === 0) return

    setIsLoading(true)
    setError(null)

    try {
      const idsParam = [...new Set(ids)].join(",")
      const url = `${COINGECKO_API}/simple/price?ids=${idsParam}&vs_currencies=usd&include_24hr_change=true&include_market_cap=true&include_24hr_vol=true`

      const response = await fetch(url)
      if (!response.ok) {
        throw new Error(`CoinGecko API error: ${response.status}`)
      }

      const data: CoinGeckoPriceResponse = await response.json()

      const newPrices: Record<number, TokenPrice> = {}
      for (const chainId of targetChains) {
        const coingeckoId = COINGECKO_IDS[chainId]
        if (!coingeckoId || !data[coingeckoId]) continue
        const tokenMeta = CHAIN_NATIVE_TOKENS.find(t => t.chainId === chainId)!
        const marketData = data[coingeckoId]
        newPrices[chainId] = {
          symbol: tokenMeta.symbol,
          name: tokenMeta.name,
          price: marketData.usd,
          priceChange24h: marketData.usd_24h_change,
          marketCap: marketData.usd_market_cap,
          volume24h: marketData.usd_24h_vol,
          lastUpdated: (marketData.last_updated_at ?? Date.now() / 1000) * 1000,
          chainId,
        }
      }
      setPrices(prev => ({ ...prev, ...newPrices }))
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [targetChains])

  const getPrice = useCallback((chainId: number): TokenPrice | null => {
    return prices[chainId] ?? null
  }, [prices])

  useEffect(() => {
    fetchPrices()
    if (autoRefresh) {
      intervalRef.current = setInterval(fetchPrices, refreshInterval)
    }
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current)
    }
  }, [fetchPrices, autoRefresh, refreshInterval])

  return { prices, isLoading, error, refresh: fetchPrices, getPrice }
}

// Standalone price hook for a single chain
export function useSingleChainPrice(chainId: number): TokenPrice | null {
  const { prices } = useTokenPrice([chainId])
  return prices[chainId] ?? null
}

