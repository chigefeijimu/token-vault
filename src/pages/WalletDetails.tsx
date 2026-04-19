import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useWalletStore } from '../stores/walletStore'
import { CHAINS } from '../types/wallet'

export function WalletDetails() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { getWallet, selectedChain, setSelectedChain, fetchBalance, deleteWallet, balances } = useWalletStore()
  const [showPrivateKey, setShowPrivateKey] = useState(false)
  const [privateKey, setPrivateKey] = useState<string | null>(null)
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [isDeleting, setIsDeleting] = useState(false)

  const wallet = id ? getWallet(id) : null

  useEffect(() => {
    if (wallet) {
      fetchBalance(wallet.address)
    }
  }, [wallet, selectedChain])

  const handleExportPrivateKey = async () => {
    if (!password) {
      setError('Please enter your password')
      return
    }
    const result = await window.__TAURI__.core.invoke<string>('export_private_key', {
      id: wallet!.id,
      password,
    })
    if (result) {
      setPrivateKey(result)
      setShowPrivateKey(true)
      setError('')
    } else {
      setError('Invalid password')
    }
  }

  const handleDelete = async () => {
    if (!wallet) return
    if (!confirm(`Delete wallet "${wallet.name}"? This cannot be undone.`)) return

    setIsDeleting(true)
    await deleteWallet(wallet.id)
    navigate('/')
  }

  if (!wallet) {
    return (
      <div className="p-6 text-center">
        <p className="text-gray-500">Wallet not found</p>
        <button onClick={() => navigate('/')} className="mt-4 text-blue-500">Back to Dashboard</button>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <button onClick={() => navigate('/')} className="mb-4 text-gray-400 hover:text-white flex items-center gap-2">
        ← Back
      </button>

      <div className="bg-gray-800 rounded-xl p-6 mb-6 border border-gray-700">
        <div className="flex justify-between items-start mb-4">
          <div>
            <h1 className="text-2xl font-bold">{wallet.name}</h1>
            <p className="text-gray-400 text-sm font-mono mt-1">{wallet.address}</p>
          </div>
          <button
            onClick={() => navigate(`/send/${wallet.id}`)}
            className="px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 transition"
          >
            Send
          </button>
        </div>

        <div className="mb-4">
          <label className="block text-sm text-gray-400 mb-2">Select Chain</label>
          <div className="flex gap-2 flex-wrap">
            {CHAINS.map(chain => (
              <button
                key={chain.id}
                onClick={() => setSelectedChain(chain.id)}
                className={`px-3 py-1 rounded-full text-sm transition ${
                  selectedChain === chain.id
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
                }`}
              >
                {chain.name}
              </button>
            ))}
          </div>
        </div>

        <div className="bg-gray-900 rounded-lg p-4">
          <p className="text-sm text-gray-400 mb-1">
            {CHAINS.find(c => c.id === selectedChain)?.name} Balance
          </p>
          <p className="text-3xl font-bold">
            {balances[wallet.address]?.[selectedChain] ?? '—'}
          </p>
        </div>
      </div>

      <div className="bg-gray-800 rounded-xl p-6 border border-gray-700">
        <h2 className="text-lg font-semibold mb-4">Export Private Key</h2>
        {!showPrivateKey ? (
          <div className="space-y-4">
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter password to export"
              className="w-full px-4 py-2 bg-gray-900 rounded-lg border border-gray-700 focus:border-blue-500 focus:outline-none"
            />
            {error && <p className="text-red-500 text-sm">{error}</p>}
            <button
              onClick={handleExportPrivateKey}
              className="w-full py-2 bg-gray-700 rounded-lg hover:bg-gray-600 transition"
            >
              Show Private Key
            </button>
          </div>
        ) : (
          <div>
            <div className="bg-red-900/30 border border-red-600 rounded-lg p-4 mb-4">
              <p className="text-red-500 text-sm font-medium mb-2">⚠️ Never share your private key!</p>
              <div className="bg-gray-900 p-4 rounded-lg">
                <p className="font-mono text-sm break-all">{privateKey}</p>
              </div>
            </div>
            <button
              onClick={() => { setShowPrivateKey(false); setPrivateKey(null); setPassword(''); }}
              className="w-full py-2 bg-gray-700 rounded-lg hover:bg-gray-600 transition"
            >
              Hide
            </button>
          </div>
        )}
      </div>

      <button
        onClick={handleDelete}
        disabled={isDeleting}
        className="w-full mt-6 py-3 bg-red-900/50 border border-red-700 rounded-lg hover:bg-red-900 transition text-red-400 disabled:opacity-50"
      >
        {isDeleting ? 'Deleting...' : 'Delete Wallet'}
      </button>
    </div>
  )
}
