import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useWalletStore } from '../stores/walletStore'

export function CreateWallet() {
  const navigate = useNavigate()
  const { createWallet } = useWalletStore()
  const [name, setName] = useState('')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState('')
  const [step, setStep] = useState<'form' | 'mnemonic'>('form')
  const [generatedMnemonic, setGeneratedMnemonic] = useState('')
  const [isCreating, setIsCreating] = useState(false)

  const handleCreate = async () => {
    if (!name.trim()) {
      setError('Please enter a wallet name')
      return
    }
    if (password.length < 8) {
      setError('Password must be at least 8 characters')
      return
    }
    if (password !== confirmPassword) {
      setError('Passwords do not match')
      return
    }

    setIsCreating(true)
    const result = await createWallet(name, password)
    setIsCreating(false)

    if (result.success) {
      setGeneratedMnemonic(result.mnemonic || '')
      setStep('mnemonic')
    } else {
      setError(result.error || 'Failed to create wallet')
    }
  }

  const handleDone = () => {
    navigate('/')
  }

  return (
    <div className="p-6 max-w-lg mx-auto">
      <button
        onClick={() => navigate('/')}
        className="mb-4 text-gray-400 hover:text-white flex items-center gap-2"
      >
        ← Back
      </button>

      <h1 className="text-2xl font-bold mb-6">Create New Wallet</h1>

      {step === 'form' && (
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
            <label className="block text-sm text-gray-400 mb-1">Confirm Password</label>
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="Confirm password"
              className="w-full px-4 py-2 bg-gray-800 rounded-lg border border-gray-700 focus:border-blue-500 focus:outline-none"
            />
          </div>

          {error && (
            <p className="text-red-500 text-sm">{error}</p>
          )}

          <button
            onClick={handleCreate}
            disabled={isCreating}
            className="w-full py-3 bg-blue-600 rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
          >
            {isCreating ? 'Creating...' : 'Create Wallet'}
          </button>
        </div>
      )}

      {step === 'mnemonic' && (
        <div>
          <div className="bg-yellow-900/30 border border-yellow-600 rounded-lg p-4 mb-4">
            <p className="text-yellow-500 font-medium mb-2">⚠️ Save Your Recovery Phrase</p>
            <p className="text-sm text-gray-400 mb-3">
              Write down these 12 words and keep them safe. This is the only way to recover your wallet.
            </p>
            <div className="bg-gray-900 p-4 rounded-lg">
              <p className="font-mono text-sm leading-relaxed break-all">
                {generatedMnemonic}
              </p>
            </div>
          </div>

          <button
            onClick={handleDone}
            className="w-full py-3 bg-green-600 rounded-lg hover:bg-green-700 transition"
          >
            I've Saved My Recovery Phrase
          </button>
        </div>
      )}
    </div>
  )
}
