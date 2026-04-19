import { useState } from "react"
import { TransactionHistory } from "../components/TransactionHistory"
import { TokenPriceDisplay } from "../components/TokenPriceDisplay"
import { CHAINS } from "../types/wallet"
import { useTokenPrice } from "../hooks/useTokenPrice"

export function Dashboard() {
  const [selectedChainId, setSelectedChainId] = useState(1)
  // const chain = CHAINS.find(c => c.id === selectedChainId) ?? CHAINS[0]

  // Token prices for the header
  const { prices, isLoading: priceLoading } = useTokenPrice(
    [1, 56, 137, 42161, 43114],
    { autoRefresh: true, refreshInterval: 60_000 }
  )

  return (
    <div className="p-4 sm:p-6 max-w-6xl mx-auto space-y-6">
      {/* Page header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
        <div>
          <h1 className="text-2xl font-bold text-vault-text">Dashboard</h1>
          <p className="text-sm text-gray-500 mt-0.5">
            Token prices, market data, and transaction history
          </p>
        </div>
        {/* Chain selector */}
        <div className="flex items-center gap-2">
          <span className="text-sm text-gray-500">Chain:</span>
          <select
            value={selectedChainId}
            onChange={e => setSelectedChainId(Number(e.target.value))}
            className="bg-vault-bg border border-vault-border rounded-lg px-3 py-1.5 text-vault-text text-sm focus:border-transparent focus:ring-2 focus:ring-vault-gradient/50"
          >
            {CHAINS.map(c => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Token Price Section */}
      <div>
        <TokenPriceDisplay chainIds={[1, 56, 137, 42161, 43114]} />
      </div>

      {/* Price summary bar for top chains */}
      <div className="bg-vault-card rounded-xl p-4 border border-vault-border">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-base font-semibold text-vault-text">Live Prices</h2>
          {priceLoading && (
            <div className="flex items-center gap-1.5 text-xs text-gray-500">
              <div className="h-3 w-3 border-2 border-vault-gradient border-t-transparent rounded-full animate-spin" />
              Updating
            </div>
          )}
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
          {[1, 56, 137, 42161, 43114].map(chainId => {
            const price = prices[chainId]
            const chainInfo = CHAINS.find(c => c.id === chainId)
            if (!price) {
              return (
                <div key={chainId} className="text-center py-2">
                  <p className="text-xs text-gray-500">{chainInfo?.name}</p>
                  <p className="text-sm text-gray-600 mt-1">--</p>
                </div>
              )
            }
            const change = price.priceChange24h ?? 0
            const changeClass = change >= 0 ? "text-green-400" : "text-red-400"
            return (
              <div key={chainId} className="text-center py-2">
                <p className="text-xs text-gray-500">{chainInfo?.name}</p>
                <p className="text-base font-bold text-vault-text mt-1">
                  ${price.price >= 1 ? price.price.toFixed(2) : price.price.toFixed(4)}
                </p>
                {price.priceChange24h !== undefined && (
                  <p className={`text-xs ${changeClass} mt-0.5`}>
                    {change >= 0 ? "+" : ""}{change.toFixed(2)}%
                  </p>
                )}
              </div>
            )
          })}
        </div>
      </div>

      {/* Transaction History Section */}
      <div>
        <TransactionHistory chainId={selectedChainId} />
      </div>
    </div>
  )
}

export default Dashboard

