import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useWalletStore } from '../stores/walletStore'
import { CHAINS } from '../types/wallet'

export function WalletDashboard() {
  const navigate = useNavigate()
  const { wallets, selectedChain, setSelectedChain, fetchBalance } = useWalletStore()
  const [refreshing, setRefreshing] = useState(false)

  const handleRefresh = async () => {
    setRefreshing(true)
    for (const wallet of wallets) {
      await fetchBalance(wallet.address)
    }
    setRefreshing(false)
  }

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">My Wallets</h1>
        <div className="flex gap-3">
          <button
            onClick={() => navigate('/create')}
            className="px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 transition"
          >
            + Create
          </button>
          <button
            onClick={() => navigate('/import')}
            className="px-4 py-2 bg-green-600 rounded-lg hover:bg-green-700 transition"
          >
            + Import
          </button>
          <button
            onClick={handleRefresh}
            disabled={refreshing}
            className="px-4 py-2 bg-gray-700 rounded-lg hover:bg-gray-600 transition disabled:opacity-50"
          >
            {refreshing ? 'Refreshing...' : '↻ Refresh'}
          </button>
        </div>
      </div>

      <div className="mb-4 flex gap-2 flex-wrap">
        {CHAINS.map(chain => (
          <button
            key={chain.id}
            onClick={() => setSelectedChain(chain.id)}
            className={`px-3 py-1 rounded-full text-sm transition ${
              selectedChain === chain.id
                ? 'bg-blue-600 text-white'
                : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
            }`}
          >
            {chain.name}
          </button>
        ))}
      </div>

      {wallets.length === 0 ? (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg mb-4">No wallets yet</p>
          <p className="text-sm">Create or import a wallet to get started</p>
        </div>
      ) : (
        <div className="grid gap-4">
          {wallets.map(wallet => (
            <div
              key={wallet.id}
              onClick={() => navigate(`/wallet/${wallet.id}`)}
              className="bg-gray-800 rounded-lg p-4 cursor-pointer hover:bg-gray-750 transition border border-gray-700"
            >
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="font-medium text-lg">{wallet.name}</h3>
                  <p className="text-gray-400 text-sm font-mono mt-1">
                    {wallet.address}
                  </p>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    navigate(`/send/${wallet.id}`)
                  }}
                  className="px-3 py-1 bg-blue-600 rounded text-sm hover:bg-blue-700"
                >
                  Send
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
