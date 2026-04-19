import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useWalletStore } from '../stores/walletStore'

export function ImportWallet() {
  const navigate = useNavigate()
  const { importWallet } = useWalletStore()
  const [name, setName] = useState('')
  const [password, setPassword] = useState('')
  const [importType, setImportType] = useState<'mnemonic' | 'privateKey'>('mnemonic')
  const [mnemonic, setMnemonic] = useState('')
  const [privateKey, setPrivateKey] = useState('')
  const [error, setError] = useState('')
  const [isImporting, setIsImporting] = useState(false)

  const handleImport = async () => {
    if (!name.trim()) {
      setError('Please enter a wallet name')
      return
    }
    if (password.length < 8) {
      setError('Password must be at least 8 characters')
      return
    }
    if (importType === 'mnemonic' && mnemonic.trim().split(/\s+/).length !== 12) {
      setError('Please enter exactly 12 words')
      return
    }
    if (importType === 'privateKey' && !privateKey.startsWith('0x')) {
      setError('Private key must start with 0x')
      return
    }

    setIsImporting(true)
    const result = await importWallet(
      name,
      password,
      importType === 'mnemonic' ? mnemonic : privateKey,
      importType
    )
    setIsImporting(false)

    if (result.success) {
      navigate('/')
    } else {
      setError(result.error || 'Failed to import wallet')
    }
  }

  return (
    <div className="p-6 max-w-lg mx-auto">
      <button
        onClick={() => navigate('/')}
        className="mb-4 text-gray-400 hover:text-white flex items-center gap-2"
      >
        ← Back
      </button>

      <h1 className="text-2xl font-bold mb-6">Import Wallet</h1>

      <div className="space-y-4">
        <div>
          <label className="block text-sm text-gray-400 mb-1">Wallet Name</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My Wallet"
            className="w-full px-4 py-2 bg-gray-800 rounded-lg border border-gray-700 focus:border-blue-500 focus:outline-none"
          />
        </div>

        <div>
          <label className="block text-sm text-gray-400 mb-1">Password</label>
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Min 8 characters"
            className="w-full px-4 py-2 bg-gray-800 rounded-lg border border-gray-700 focus:border-blue-500 focus:outline-none"
          />
        </div>

        <div>
          <label className="block text-sm text-gray-400 mb-2">Import Method</label>
          <div className="flex gap-4">
            <button
              type="button"
              onClick={() => setImportType('mnemonic')}
              className={`flex-1 py-2 rounded-lg transition ${
                importType === 'mnemonic'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
              }`}
            >
              Recovery Phrase
            </button>
            <button
              type="button"
              onClick={() => setImportType('privateKey')}
              className={`flex-1 py-2 rounded-lg transition ${
                importType === 'privateKey'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
              }`}
            >
              Private Key
            </button>
          </div>
        </div>

        {importType === 'mnemonic' ? (
          <div>
            <label className="block text-sm text-gray-400 mb-1">Recovery Phrase (12 words)</label>
            <textarea
              value={mnemonic}
              onChange={(e) => setMnemonic(e.target.value)}
              placeholder="word1 word2 word3 ..."
              rows={3}
              className="w-full px-4 py-2 bg-gray-800 rounded-lg border border-gray-700 focus:border-blue-500 focus:outline-none font-mono text-sm resize-none"
            />
          </div>
        ) : (
          <div>
            <label className="block text-sm text-gray-400 mb-1">Private Key</label>
            <input
              type="text"
              value={privateKey}
              onChange={(e) => setPrivateKey(e.target.value)}
              placeholder="0x..."
              className="w-full px-4 py-2 bg-gray-800 rounded-lg border border-gray-700 focus:border-blue-500 focus:outline-none font-mono"
            />
          </div>
        )}

        {error && (
          <p className="text-red-500 text-sm">{error}</p>
        )}

        <button
          onClick={handleImport}
          disabled={isImporting}
          className="w-full py-3 bg-green-600 rounded-lg hover:bg-green-700 transition disabled:opacity-50"
        >
          {isImporting ? 'Importing...' : 'Import Wallet'}
        </button>
      </div>
    </div>
  )
}
