import { useState } from "react"
import { useNavigate } from "react-router-dom"
import { TransactionHistory } from "../components/TransactionHistory"
import { TokenPriceDisplay } from "../components/TokenPriceDisplay"
import { NFTGallery } from "../components/NFTGallery"
import { CHAINS } from "../types/wallet"
import { useWalletStore } from "../stores/walletStore"
import { useTokenPrice } from "../hooks/useTokenPrice"
import { usePendingTxPolling } from "../hooks/usePendingTxPolling"

type Tab = "tokens" | "transactions" | "nfts"

export function Dashboard() {
  const [selectedChainId, setSelectedChainId] = useState(1)
  const [activeTab, setActiveTab] = useState<Tab>("tokens")
  const [selectedWalletId, setSelectedWalletId] = useState<string>("")
  const navigate = useNavigate()

  // Activate transaction status polling
  usePendingTxPolling()

  const { wallets, selectedChain } = useWalletStore()
  const selectedWallet = wallets.find(w => w.id === selectedWalletId)

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
        {/* Chain selector + Settings */}
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
          <button
            onClick={() => navigate("/send")}
            className="px-3 py-1.5 bg-vault-gradient text-white rounded-lg text-sm font-medium hover:opacity-90 transition"
          >
            Send
          </button>
          <button
            onClick={() => navigate("/settings")}
            className="px-3 py-1.5 bg-vault-border text-vault-text rounded-lg text-sm hover:bg-vault-bg transition"
          >
            Settings
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex items-center gap-1 border-b border-vault-border">
        {(["tokens", "transactions", "nfts"] as Tab[]).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm font-medium transition border-b-2 -mb-px ${
              activeTab === tab
                ? "border-vault-gradient text-vault-text"
                : "border-transparent text-gray-500 hover:text-gray-300"
            }`}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      {activeTab === "tokens" && (
        <>
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
        </>
      )}

      {activeTab === "transactions" && (
        <div>
          <TransactionHistory chainId={selectedChainId} />
        </div>
      )}

      {activeTab === "nfts" && (
        <>
          {wallets.length === 0 ? (
            <div className="bg-vault-card rounded-xl p-8 border border-vault-border text-center">
              <p className="text-gray-400 mb-4">No wallet found. Import or create a wallet to view NFTs.</p>
              <div className="flex justify-center gap-3">
                <button
                  onClick={() => navigate("/import")}
                  className="px-4 py-2 bg-vault-gradient text-white rounded-lg text-sm font-medium hover:opacity-90 transition"
                >
                  Import Wallet
                </button>
                <button
                  onClick={() => navigate("/create")}
                  className="px-4 py-2 bg-vault-border text-vault-text rounded-lg text-sm font-medium hover:bg-vault-bg transition"
                >
                  Create Wallet
                </button>
              </div>
            </div>
          ) : (
            <div className="space-y-3">
              <div className="flex items-center gap-2">
                <span className="text-sm text-gray-500">Wallet:</span>
                <select
                  value={selectedWalletId}
                  onChange={e => setSelectedWalletId(e.target.value)}
                  className="bg-vault-bg border border-vault-border rounded-lg px-3 py-1.5 text-vault-text text-sm focus:border-transparent focus:ring-2 focus:ring-vault-gradient/50 max-w-xs"
                >
                  <option value="">Select a wallet</option>
                  {wallets.map(w => (
                    <option key={w.id} value={w.id}>{w.name} ({w.address})</option>
                  ))}
                </select>
              </div>
              {selectedWallet ? (
                <NFTGallery
                  address={selectedWallet.address}
                  chainId={selectedChain}
                />
              ) : (
                <div className="bg-vault-card rounded-xl p-8 border border-vault-border text-center text-gray-500 text-sm">
                  Select a wallet above to view NFTs.
                </div>
              )}
            </div>
          )}
        </>
      )}
    </div>
  )
}

export default Dashboard

