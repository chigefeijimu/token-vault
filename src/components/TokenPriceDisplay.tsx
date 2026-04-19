import { useState } from "react"
import { useTokenPrice } from "../hooks/useTokenPrice"
import { CHAINS } from "../types/wallet"
import type { TokenPrice } from "../types/price"

function formatPrice(price: number): string {
  if (price >= 1000) return `$${price.toLocaleString("en-US", { maximumFractionDigits: 2 })}`
  if (price >= 1) return `$${price.toFixed(2)}`
  if (price >= 0.01) return `$${price.toFixed(4)}`
  return `$${price.toFixed(6)}`
}

function formatChange(change?: number): string {
  if (change === undefined) return ""
  const sign = change >= 0 ? "+" : ""
  return `${sign}${change.toFixed(2)}%`
}

function PriceCard({ price, chainId }: { price: TokenPrice; chainId: number }) {
  const chain = CHAINS.find(c => c.id === chainId)
  const changeClass = (price.priceChange24h ?? 0) >= 0 ? "text-green-400" : "text-red-400"

  return (
    <div className="bg-vault-bg rounded-lg p-3 border border-vault-border/50 hover:border-vault-border transition flex flex-col gap-1">
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold text-vault-text">{price.symbol}</span>
          <span className="text-xs text-gray-500">{chain?.name}</span>
        </div>
        {price.priceChange24h !== undefined && (
          <span className={`text-xs font-medium ${changeClass}`}>
            {formatChange(price.priceChange24h)}
          </span>
        )}
      </div>
      <div className="flex justify-between items-end">
        <span className="text-base font-bold text-vault-text">
          {formatPrice(price.price)}
        </span>
      </div>
    </div>
  )
}

interface TokenPriceDisplayProps {
  chainIds?: number[]
  compact?: boolean
}

export function TokenPriceDisplay({ chainIds, compact = false }: TokenPriceDisplayProps) {
  const targetChains = chainIds ?? [1, 56, 137, 42161, 10, 43114]
  const { prices, isLoading, error, refresh } = useTokenPrice(targetChains)
  const [showAll, setShowAll] = useState(!compact)

  if (error) {
    return (
      <div className="text-red-400 text-sm p-2">
        Failed to load prices: {error}
        <button onClick={refresh} className="ml-2 underline">Retry</button>
      </div>
    )
  }

  const visibleChains = showAll ? targetChains : targetChains.slice(0, 3)

  return (
    <div className="space-y-2">
      <div className="flex justify-between items-center">
        <h3 className="text-sm font-semibold text-vault-text">Token Prices</h3>
        <button
          onClick={refresh}
          disabled={isLoading}
          className="text-xs text-gray-400 hover:text-vault-text disabled:opacity-50"
        >
          {isLoading ? "Loading..." : "Refresh"}
        </button>
      </div>
      <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
        {visibleChains.map(chainId => {
          const price = prices[chainId]
          if (!price) return null
          return <PriceCard key={chainId} price={price} chainId={chainId} />
        })}
      </div>
      {targetChains.length > 3 && (
        <button
          onClick={() => setShowAll(!showAll)}
          className="text-xs text-vault-accent hover:underline"
        >
          {showAll ? "Show less" : `+${targetChains.length - 3} more`}
        </button>
      )}
    </div>
  )
}
